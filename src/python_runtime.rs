//! Blue Python task and event responsibilities, executed by Bevy's task pool.
#![allow(non_snake_case)]

use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, TaskPool, block_on, poll_once};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use std::cell::Cell;
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

thread_local! {
    static TASK_ID: Cell<u64> = const { Cell::new(0) };
    static BLOCK_TRAP: Cell<usize> = const { Cell::new(0) };
}

pub struct BlueTask {
    pub id: u64,
    done: AtomicBool,
}
impl BlueTask {
    pub fn alive(&self) -> bool {
        !self.done.load(Ordering::Acquire)
    }
}

struct TaskContext(u64);
impl TaskContext {
    fn enter(id: u64) -> Self {
        Self(TASK_ID.with(|slot| slot.replace(id)))
    }
}
impl Drop for TaskContext {
    fn drop(&mut self) {
        TASK_ID.with(|slot| slot.set(self.0));
    }
}
struct BlockTrap;
impl BlockTrap {
    fn enter() -> Self {
        BLOCK_TRAP.with(|n| n.set(n.get() + 1));
        Self
    }
}
impl Drop for BlockTrap {
    fn drop(&mut self) {
        BLOCK_TRAP.with(|n| n.set(n.get() - 1));
    }
}
struct TaskCompletion(Arc<BlueTask>);
impl Drop for TaskCompletion {
    fn drop(&mut self) {
        self.0.done.store(true, Ordering::Release);
    }
}

struct YieldWaiter {
    ready: Mutex<bool>,
    wake: Condvar,
}

/// Source Synchro owns its yielders; native condition variables replace channels
/// and stack switching. Remaining time/sleeper responsibilities stay pending.
#[derive(Default)]
pub struct Synchro {
    mYielders: Mutex<Vec<Arc<YieldWaiter>>>,
    mShutdown: AtomicBool,
}
impl Synchro {
    pub fn Yield(&self, py: Python<'_>) -> PyResult<()> {
        if TASK_ID.with(Cell::get) == 0 {
            return Err(PyRuntimeError::new_err("Cannot yield the main task"));
        }
        if BLOCK_TRAP.with(Cell::get) != 0 {
            return Err(PyRuntimeError::new_err(
                "Scheduling is forbidden in a synchronous Blue callback",
            ));
        }
        py.detach(|| {
            if self.mShutdown.load(Ordering::Acquire) {
                return;
            }
            let waiter = Arc::new(YieldWaiter {
                ready: Mutex::new(false),
                wake: Condvar::new(),
            });
            self.mYielders
                .lock()
                .expect("Blue yield queue poisoned")
                .push(waiter.clone());
            let ready = waiter.ready.lock().expect("Blue yield state poisoned");
            drop(
                waiter
                    .wake
                    .wait_while(ready, |ready| !*ready)
                    .expect("Blue yield state poisoned"),
            );
        });
        Ok(())
    }
    fn resume_yielders(&self) {
        let waiters =
            std::mem::take(&mut *self.mYielders.lock().expect("Blue yield queue poisoned"));
        for waiter in waiters {
            *waiter.ready.lock().expect("Blue yield state poisoned") = true;
            waiter.wake.notify_one();
        }
    }
}

struct Submission {
    start: Arc<YieldWaiter>,
    completed: async_channel::Receiver<()>,
    worker: std::thread::JoinHandle<()>,
}

/// Initial task/event portion of BluePyOS. PyO3 supplies interpreter attachment
/// and ownership; Bevy owns submissions, scheduling and retained task handles.
pub struct BluePyOS {
    pub mSynchro: Arc<Synchro>,
    next_task: AtomicU64,
    pending: Mutex<Vec<Submission>>,
    running: Mutex<Vec<Task<()>>>,
}
impl Default for BluePyOS {
    fn default() -> Self {
        Self {
            mSynchro: Arc::new(Synchro::default()),
            next_task: AtomicU64::new(1),
            pending: Mutex::new(Vec::new()),
            running: Mutex::new(Vec::new()),
        }
    }
}
impl BluePyOS {
    pub fn CreateTaskletImpl(
        &self,
        callable: Py<PyAny>,
        args: Py<PyTuple>,
        kwargs: Option<Py<PyDict>>,
    ) -> PyResult<Arc<BlueTask>> {
        let task = Arc::new(BlueTask {
            id: self.next_task.fetch_add(1, Ordering::Relaxed),
            done: AtomicBool::new(false),
        });
        let signal = task.clone();
        let start = Arc::new(YieldWaiter {
            ready: Mutex::new(false),
            wake: Condvar::new(),
        });
        let worker_start = start.clone();
        let (complete, completed) = async_channel::bounded(1);
        // A synchronous Python stack may suspend inside Yield. Give that stack
        // an OS thread so it cannot exhaust Bevy's bounded async worker pool.
        // Bevy still owns admission and completion; async-channel bridges the
        // blocking Python stack without another general-purpose executor.
        let worker = std::thread::Builder::new()
            .name(format!("blue-python-{}", task.id))
            .spawn(move || {
                let _completion = TaskCompletion(signal.clone());
                let ready = worker_start
                    .ready
                    .lock()
                    .expect("Blue task admission poisoned");
                drop(
                    worker_start
                        .wake
                        .wait_while(ready, |ready| !*ready)
                        .expect("Blue task admission poisoned"),
                );
                let _context = TaskContext::enter(signal.id);
                Python::attach(|py| {
                    if let Err(error) = callable
                        .bind(py)
                        .call(args.bind(py), kwargs.as_ref().map(|kw| kw.bind(py)))
                    {
                        error.print(py);
                    }
                });
                let _ = complete.try_send(());
            })
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        self.pending
            .lock()
            .expect("Blue task queue poisoned")
            .push(Submission {
                start,
                completed,
                worker,
            });
        Ok(task)
    }
    pub fn SendEvent(
        &self,
        py: Python<'_>,
        target: &Bound<'_, PyAny>,
        eventName: &str,
        args: Py<PyTuple>,
    ) -> bool {
        let Ok(callable) = target.getattr(eventName) else {
            return true;
        };
        let _trap = BlockTrap::enter();
        match callable.call(args.bind(py), None) {
            Ok(_) => true,
            Err(error) => {
                error.print(py);
                false
            }
        }
    }
    pub fn PostEvent(
        &self,
        py: Python<'_>,
        target: &Bound<'_, PyAny>,
        eventName: &str,
        args: Py<PyTuple>,
    ) -> bool {
        let Ok(callable) = target.getattr(eventName) else {
            return true;
        };
        let callable = callable.unbind();
        match py.detach(|| self.CreateTaskletImpl(callable, args, None)) {
            Ok(_) => true,
            Err(error) => {
                error.print(py);
                false
            }
        }
    }
    fn pump(&self) {
        self.mSynchro.resume_yielders();
        let submissions =
            std::mem::take(&mut *self.pending.lock().expect("Blue task queue poisoned"));
        let pool = AsyncComputeTaskPool::get_or_init(TaskPool::new);
        let mut running = self.running.lock().expect("Blue running tasks poisoned");
        for submission in submissions {
            *submission
                .start
                .ready
                .lock()
                .expect("Blue task admission poisoned") = true;
            submission.start.wake.notify_one();
            running.push(pool.spawn(async move {
                let _ = submission.completed.recv().await;
                // Completion is sent after Python work is over. Joining here
                // never waits for a suspended Python task to make progress.
                let _ = submission.worker.join();
            }));
        }
        running.retain_mut(|task| block_on(poll_once(task)).is_none());
    }
}

#[derive(Resource, Clone)]
pub(crate) struct BluePythonResource(pub Arc<BluePyOS>);
pub(crate) fn pump_python(runtime: Res<BluePythonResource>) {
    runtime.0.pump();
}
