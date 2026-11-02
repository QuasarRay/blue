#![allow(non_snake_case)]
use super::wrapper::BlueWrapper;
use crate::python_runtime::{BluePyOS, BlueTask, Synchro};
use crate::{BlueRuntime, IRoot, ObjectHandle};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule, PyTuple};
use std::sync::Arc;

/// Original `src-cpp/BlueTestHelpers.h::BlueTestEvents` has no own state.
pub struct BlueTestEvents;
impl IRoot for BlueTestEvents {}
impl BlueTestEvents {
    pub fn PySendEvent(
        py: Python<'_>,
        target: &Bound<'_, PyAny>,
        first: i32,
        second: f64,
    ) -> PyResult<bool> {
        let runtime = py.detach(|| BlueRuntime::shared().python());
        Ok(runtime.SendEvent(
            py,
            target,
            "DoSendEvent",
            (first, second).into_pyobject(py)?.unbind(),
        ))
    }
    pub fn PyPostEvent(
        py: Python<'_>,
        target: &Bound<'_, PyAny>,
        first: i32,
        second: f64,
    ) -> PyResult<bool> {
        let runtime = py.detach(|| BlueRuntime::shared().python());
        Ok(runtime.PostEvent(
            py,
            target,
            "OnPostEvent",
            (first, second).into_pyobject(py)?.unbind(),
        ))
    }
}

#[pyclass(name="BlueTestEvents", module="blue", extends=BlueWrapper, frozen)]
pub(super) struct PyBlueTestEvents {
    _handle: ObjectHandle<BlueTestEvents>,
}
#[pymethods]
impl PyBlueTestEvents {
    #[new]
    pub(super) fn new(py: Python<'_>) -> PyClassInitializer<Self> {
        PyClassInitializer::from(BlueWrapper::new("blue.BlueTestEvents", &[])).add_subclass(Self {
            _handle: py
                .detach(|| BlueRuntime::shared().create("blue.BlueTestEvents", BlueTestEvents)),
        })
    }
    fn SendEvent(slf: &Bound<'_, Self>, first: i32, second: f64) -> PyResult<bool> {
        BlueTestEvents::PySendEvent(slf.py(), slf.as_any(), first, second)
    }
    fn PostEvent(slf: &Bound<'_, Self>, first: i32, second: f64) -> PyResult<bool> {
        BlueTestEvents::PyPostEvent(slf.py(), slf.as_any(), first, second)
    }
}

#[pyclass(name = "BlueTask", module = "blue", frozen)]
struct PyBlueTask {
    task: Arc<BlueTask>,
}
#[pymethods]
impl PyBlueTask {
    #[getter]
    fn alive(&self) -> bool {
        self.task.alive()
    }
    #[getter]
    fn task_id(&self) -> u64 {
        self.task.id
    }
}

#[pyclass(name = "BluePyOS", module = "blue", frozen)]
struct PyBluePyOS {
    runtime: Arc<BluePyOS>,
}
#[pymethods]
impl PyBluePyOS {
    #[pyo3(signature=(callable, args=None, kwargs=None))]
    fn CreateTasklet(
        &self,
        py: Python<'_>,
        callable: Py<PyAny>,
        args: Option<Py<PyTuple>>,
        kwargs: Option<Py<PyDict>>,
    ) -> PyResult<PyBlueTask> {
        let args = args.unwrap_or_else(|| PyTuple::empty(py).unbind());
        let task = py.detach(|| self.runtime.CreateTaskletImpl(callable, args, kwargs))?;
        Ok(PyBlueTask { task })
    }
}

#[pyclass(name = "Synchro", module = "blue", frozen)]
struct PySynchro {
    synchro: Arc<Synchro>,
}
#[pymethods]
impl PySynchro {
    fn Yield(&self, py: Python<'_>) -> PyResult<()> {
        self.synchro.Yield(py)
    }
}

#[pyclass(name = "BlueOS", module = "blue", frozen)]
struct PyBlueOS {
    os: Arc<crate::BlueOS>,
}
#[pymethods]
impl PyBlueOS {
    fn Pump(&self, py: Python<'_>) {
        py.detach(|| self.os.PumpOS());
    }
}

pub(super) fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    let runtime = py.detach(|| BlueRuntime::shared().python());
    m.add_class::<PyBlueTestEvents>()?;
    m.add(
        "pyos",
        Py::new(
            py,
            PyBluePyOS {
                runtime: runtime.clone(),
            },
        )?,
    )?;
    m.add(
        "synchro",
        Py::new(
            py,
            PySynchro {
                synchro: runtime.mSynchro.clone(),
            },
        )?,
    )?;
    let os = py.detach(|| BlueRuntime::shared().os());
    m.add("os", Py::new(py, PyBlueOS { os })?)?;
    Ok(())
}
