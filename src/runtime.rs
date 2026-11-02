//! Bevy owns object membership, registrations and schedules. Python handles
//! retrieve the same canonical object state from the world.
#![allow(non_snake_case)]

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
fn advance_pump(mut count: ResMut<PumpCount>) {
    count.0 += 1;
}

pub struct BluePlugin;
impl Plugin for BluePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlueClasses>()
            .init_resource::<PumpCount>()
            .register_type::<ClassRegistration>()
            .add_systems(BluePump, advance_pump);
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
        ObjectHandle(
            Arc::new(ObjectLease {
                runtime: self.clone(),
                entity,
            }),
            std::marker::PhantomData,
        )
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
}

struct ObjectLease {
    runtime: Arc<BlueRuntime>,
    entity: Entity,
}
impl Drop for ObjectLease {
    fn drop(&mut self) {
        let mut world = self.runtime.world.lock().expect("Blue world poisoned");
        let class = world
            .get::<ObjectClass>(self.entity)
            .expect("missing Blue object")
            .0
            .clone();
        world
            .resource_mut::<BlueClasses>()
            .UpdateObjectCount(&class, false);
        world.despawn(self.entity);
    }
}

/// Rust ownership adapter; the source object's nominal type remains separate.
pub struct ObjectHandle<T: Send + Sync + 'static>(
    Arc<ObjectLease>,
    std::marker::PhantomData<fn() -> T>,
);
impl<T: Send + Sync + 'static> Clone for ObjectHandle<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), std::marker::PhantomData)
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
