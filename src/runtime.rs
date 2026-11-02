//! Bevy owns object membership, registrations and schedules. Python handles
//! retrieve the same canonical object state from the world.
#![allow(non_snake_case)]

use crate::python_runtime::{BluePyOS, BluePythonResource, pump_python};
use bevy::app::{App, Plugin};
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Clone, Debug, Reflect)]
pub struct ClassRegistration {
    pub name: String,
    pub interfaces: Vec<String>,
}

/// Initial registry responsibilities. Full `IBlueClasses` implementation is
/// pending and remains recorded as partial in the correspondence record.
#[derive(Resource, Default)]
pub struct BlueClasses {
    m_classesByName: BTreeMap<String, ClassRegistration>,
    live_count: BTreeMap<String, usize>,
}

impl BlueClasses {
    /// Python construction policy at the original declaring owner's boundary.
    /// PyO3's registered native constructor creates the real canonical object.
    pub fn PyCreateInstance(
        py: pyo3::Python<'_>,
        name: &str,
    ) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        use pyo3::prelude::*;
        let known = py.detach(|| {
            BlueRuntime::shared()
                .world
                .lock()
                .expect("Blue world poisoned")
                .resource::<BlueClasses>()
                .GetClassRegistration(name)
                .is_some()
        });
        if !known {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                "Unknown Blue class {name}"
            )));
        }
        let (module, class) = name
            .rsplit_once('.')
            .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("Class ID needs a module"))?;
        Ok(py.import(module)?.getattr(class)?.call0()?.unbind())
    }
    pub fn RegisterSingleClass(&mut self, registration: ClassRegistration) {
        self.live_count
            .entry(registration.name.clone())
            .or_default();
        self.m_classesByName
            .insert(registration.name.clone(), registration);
    }
    pub fn GetClassRegistration(&self, name: &str) -> Option<&ClassRegistration> {
        self.m_classesByName.get(name)
    }
    pub fn PyLiveCount(&self) -> BTreeMap<String, usize> {
        self.live_count.clone()
    }
    pub fn UpdateObjectCount(&mut self, name: &str, increment: bool) {
        let count = self.live_count.entry(name.to_owned()).or_default();
        if increment {
            *count += 1;
        } else {
            *count -= 1;
        }
    }
}

#[derive(Component)]
struct ObjectState<T: Send + Sync + 'static>(Arc<Mutex<T>>);
#[derive(Component)]
struct ObjectClass(String);
#[derive(ScheduleLabel, Hash, PartialEq, Eq, Clone, Debug)]
pub(crate) struct BluePump;
#[derive(Resource, Default)]
struct PumpCount(u64);
#[derive(Resource)]
struct BlueOSState(Arc<crate::BlueOS>);
fn advance_pump(mut count: ResMut<PumpCount>) {
    count.0 += 1;
}

pub struct BluePlugin;
impl Plugin for BluePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlueClasses>()
            .init_resource::<PumpCount>()
            .insert_resource(BlueOSState(Arc::new(crate::BlueOS::default())))
            .insert_resource(BluePythonResource(Arc::new(BluePyOS::default())))
            .register_type::<ClassRegistration>()
            .add_systems(BluePump, (advance_pump, pump_python));
        app.world_mut()
            .resource_mut::<BlueClasses>()
            .RegisterSingleClass(ClassRegistration {
                name: "blue.MemStream".into(),
                interfaces: [
                    "MemStream",
                    "IBlueMemStream",
                    "IBlueStream",
                    "ICcpStream",
                    "ICacheable",
                    "IRoot",
                ]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            });
        app.world_mut()
            .resource_mut::<BlueClasses>()
            .RegisterSingleClass(ClassRegistration {
                name: "blue.BlueTestEvents".into(),
                interfaces: vec!["BlueTestEvents".into(), "IRoot".into()],
            });
    }
}

pub struct BlueRuntime {
    world: Mutex<World>,
}

impl BlueRuntime {
    pub fn new() -> Arc<Self> {
        let mut app = App::new();
        app.add_plugins(BluePlugin);
        // Transfer the real headless world, including its schedules. App's
        // window/runner infrastructure is unnecessary on Python worker threads.
        Arc::new(Self {
            world: Mutex::new(std::mem::take(app.world_mut())),
        })
    }
    pub(crate) fn shared() -> &'static Arc<Self> {
        static RUNTIME: OnceLock<Arc<BlueRuntime>> = OnceLock::new();
        RUNTIME.get_or_init(Self::new)
    }
    pub fn create<T: Send + Sync + 'static>(
        self: &Arc<Self>,
        class: &str,
        value: T,
    ) -> ObjectHandle<T> {
        let mut world = self.world.lock().expect("Blue world poisoned");
        assert!(
            world
                .resource::<BlueClasses>()
                .GetClassRegistration(class)
                .is_some(),
            "unregistered Blue class"
        );
        let entity = world
            .spawn((
                ObjectState(Arc::new(Mutex::new(value))),
                ObjectClass(class.to_owned()),
            ))
            .id();
        world
            .resource_mut::<BlueClasses>()
            .UpdateObjectCount(class, true);
        ObjectHandle(Arc::new(ObjectLease {
            runtime: self.clone(),
            entity,
            marker: std::marker::PhantomData,
        }))
    }
    pub fn Pump(&self) {
        self.world
            .lock()
            .expect("Blue world poisoned")
            .run_schedule(BluePump);
    }
    pub fn LiveCount(&self) -> BTreeMap<String, usize> {
        self.world
            .lock()
            .expect("Blue world poisoned")
            .resource::<BlueClasses>()
            .PyLiveCount()
    }
    pub fn python(&self) -> Arc<BluePyOS> {
        self.world
            .lock()
            .expect("Blue world poisoned")
            .resource::<BluePythonResource>()
            .0
            .clone()
    }
    pub fn os(&self) -> Arc<crate::BlueOS> {
        self.world
            .lock()
            .expect("Blue world poisoned")
            .resource::<BlueOSState>()
            .0
            .clone()
    }
}

struct ObjectLease<T: Send + Sync + 'static> {
    runtime: Arc<BlueRuntime>,
    entity: Entity,
    marker: std::marker::PhantomData<fn() -> T>,
}
impl<T: Send + Sync + 'static> Drop for ObjectLease<T> {
    fn drop(&mut self) {
        let state = {
            let mut world = self.runtime.world.lock().expect("Blue world poisoned");
            let class = world
                .get::<ObjectClass>(self.entity)
                .expect("missing Blue object")
                .0
                .clone();
            world
                .resource_mut::<BlueClasses>()
                .UpdateObjectCount(&class, false);
            let state = world.entity_mut(self.entity).take::<ObjectState<T>>();
            world.despawn(self.entity);
            state
        };
        // A source object's field destruction may invoke Python finalizers or
        // other reentrant user code. Membership is removed before destruction,
        // and neither the world nor a registry lock is held across that code.
        drop(state);
    }
}

/// Rust ownership adapter; the source object's nominal type remains separate.
pub struct ObjectHandle<T: Send + Sync + 'static>(Arc<ObjectLease<T>>);
impl<T: Send + Sync + 'static> Clone for ObjectHandle<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: Send + Sync + 'static> ObjectHandle<T> {
    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let state = {
            let world = self.0.runtime.world.lock().expect("Blue world poisoned");
            world
                .get::<ObjectState<T>>(self.0.entity)
                .expect("missing Blue object state")
                .0
                .clone()
        };
        f(&mut state.lock().expect("Blue object poisoned"))
    }
    pub fn entity(&self) -> Entity {
        self.0.entity
    }
}
