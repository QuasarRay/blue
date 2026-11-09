//! Bevy-native counterpart of Carbon `IBlueClasses.h`.
//!
//! `World` is the object store, `Entity` replaces an `IRoot*` identity and
//! `ComponentId` replaces a class/interface pointer offset. Registrations,
//! copy configuration and counters are Bevy `Resource`s. No parallel class
//! registry, C++ object allocator, vtable, global singleton, or raw pointer
//! is introduced here.
//!
//! ## Contract boundary
//!
//! This module delegates the compatible subset to `blue_types.rs` rather
//! than reimplementing the class registry, graph copier, or deletion queue.
//! In particular `copy_to` duplicates repeatedly referenced children and
//! rejects cycles; `clone_to` preserves aliasing and cycles among `BlueLinks`.
//!
//! **Not C++ ABI-compatible.** Interface results are entities, not adjusted
//! pointers. `update_object_count` records explicit deltas; it does not
//! emulate the C++ object's intrusive reference count or resurrectable weak
//! references. `BlueMethodDefinition`, `IBlueRtti`, and Python wrappers are
//! not defined by the supplied sources and are deliberately not invented.

use crate::blue_types::{
    self, BlueCopier, BlueCopyError, BlueObject, BluePendingDeletes,
    BlueTypesRegistry, ClassInfo, ClassRegistration, Clsid, CopyOverrideCallback,
    CustomAssignmentCallback, Iid, InterfaceEntry, PostCopyCallback, VarEntry,
    BlueCreateError,
};
use bevy::{
    ecs::component::ComponentId,
    platform::collections::{HashMap, HashSet},
    prelude::{App, Entity, Plugin, Resource, World},
};

/// Mutable, Bevy-owned copier configuration; no independent object storage.
#[derive(Resource, Default)]
pub struct BlueCopyState(pub BlueCopier);

/// Class counters written by the legacy `UpdateObjectCount` operation.
///
/// This is explicit bookkeeping; ECS entity existence and `BlueObject`
/// insertion/removal do not automatically trigger these counters.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BlueClassCount {
    pub live: u32,
    pub locks: u32,
}

/// Per-class counter state. Not a replacement for C++ intrusive reference
/// counting or `ClassInfo::mLiveCount`/`mLockCount` atomic storage.
#[derive(Resource, Default, Debug)]
pub struct BlueClassCounts {
    counts: HashMap<Clsid, BlueClassCount>,
}

impl BlueClassCounts {
    #[must_use]
    pub fn get(&self, class: &Clsid) -> BlueClassCount {
        self.counts.get(class).copied().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlueCountError {
    MissingObject,
    MissingClassTag,
    Underflow,
    Overflow,
}

#[derive(Resource, Debug)]
pub struct BlueClassesConfig {
    /// Maximum number of pending objects processed by one explicit call.
    pub pending_delete_budget: usize,
}

impl Default for BlueClassesConfig {
    fn default() -> Self {
        Self {
            pending_delete_budget: 64,
        }
    }
}

/// Initializes `IBlueClasses`'s additional Bevy resources.
///
/// Install `BlueTypesPlugin` as well for the class registry, intern pool,
/// deferred deletion queue, and `Last` schedule handler. The two plugins are
/// independent so existing applications can install `BlueTypesPlugin` once.
pub struct BlueClassesPlugin;

impl Plugin for BlueClassesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlueCopyState>()
            .init_resource::<BlueClassCounts>()
            .init_resource::<BlueClassesConfig>();
    }
}

/// Native Bevy domain facade of the C++ `IBlueClasses` class manager.
///
/// Using an extension trait on `World` avoids a second lifecycle authority;
/// calls that need both a resource and `&mut World` use `resource_scope`.
/// No method exposes a C++ pointer or foreign object layout.
pub trait IBlueClasses {
    /// Equivalent to both `RegisterClasses` overloads: slices need no sentinel.
    fn register_blue_classes(&mut self, registrations: &[(ClassInfo, ClassRegistration)]);

    /// Unregisters complete class/factory pairs by ID.
    fn unregister_blue_classes(&mut self, ids: &[Clsid]);

    /// Native equivalent of `GetClassRegistration`.
    fn get_blue_class_registration(&self, id: &Clsid) -> Option<&ClassRegistration>;

    fn get_blue_class_info(&self, id: &Clsid) -> Option<&ClassInfo>;

    /// `FindClsid(module, name)`: returns a registered ID, not merely a new hash.
    fn find_blue_clsid(&self, module: &str, name: &str) -> Option<Clsid>;

    /// Output-parameter-compatible spelling: does not modify `out` on failure.
    fn find_blue_clsid_into(&self, out: &mut Clsid, module: &str, name: &str) -> bool;

    /// `FindClsid(className)`; name-only matches the C++ API shape.
    fn find_blue_clsid_by_name(&self, name: &str) -> Option<Clsid>;

    /// `CreateInstance`, returning the Bevy-owned entity.
    fn create_blue_instance(
        &mut self,
        class: &Clsid,
        requested_interface: &Iid,
    ) -> Result<Entity, BlueCreateError>;

    fn create_blue_instance_from_name(
        &mut self,
        name: &str,
        requested_interface: &Iid,
    ) -> Result<Entity, BlueCreateError>;

    /// `QueryThisInterface`, including inherited interface tables.
    /// `BLUEQIOPT` only controls diagnostics in the source; this pure query
    /// emits no diagnostics and therefore needs no silent flag.
    fn query_blue_interface(&self, entity: Entity, requested_interface: &Iid) -> Option<Entity>;

    /// `FindVariable` in the class and parent chain. Bevy reflection and
    /// component access replace the source's `ptrdiff_t` member offsets.
    fn find_blue_variable(&self, class: &Clsid, name: &str) -> Option<&VarEntry>;

    /// Native substitute for interface RTTI. Does **not** implement `IBlueRtti`.
    fn blue_interface_component_id(&self, class: &Clsid, iid: &Iid) -> Option<ComponentId>;

    /// Record explicitly supplied live-count/lock-count deltas for a class.
    /// Neither deletion nor entity creation is driven by these counters.
    fn update_blue_object_count(
        &mut self,
        entity: Entity,
        instance_delta: i32,
        lock_delta: i32,
    ) -> Result<BlueClassCount, BlueCountError>;

    /// `CopyTo`: duplicated repeated links, cycles rejected.
    fn copy_blue_to(
        &mut self,
        source: Entity,
        destination: Option<Entity>,
    ) -> Result<Entity, BlueCopyError>;

    /// `CloneTo`: shared links and cycles preserved.
    fn clone_blue_to(
        &mut self,
        source: Entity,
        destination: Option<Entity>,
    ) -> Result<Entity, BlueCopyError>;

    fn set_blue_copy_callbacks(
        &mut self,
        override_callback: Option<CopyOverrideCallback>,
        post_copy_callback: Option<PostCopyCallback>,
    );

    fn register_blue_custom_assignment(
        &mut self,
        class: Clsid,
        callback: CustomAssignmentCallback,
    );

    /// `ProcessPendingDeletes` with configurable operation budget.
    fn process_blue_pending_deletes(&mut self) -> usize;

    fn process_all_blue_pending_deletes(&mut self) -> usize;

    fn set_blue_pending_deletes_enabled(&mut self, enabled: bool);

    fn blue_pending_deletes_enabled(&self) -> bool;
}

fn checked_delta(value: u32, delta: i32) -> Result<u32, BlueCountError> {
    if delta >= 0 {
        value.checked_add(delta.unsigned_abs()).ok_or(BlueCountError::Overflow)
    } else {
        value.checked_sub(delta.unsigned_abs()).ok_or(BlueCountError::Underflow)
    }
}

impl IBlueClasses for World {
    fn register_blue_classes(&mut self, registrations: &[(ClassInfo, ClassRegistration)]) {
        self.init_resource::<BlueTypesRegistry>();
        self.resource_mut::<BlueTypesRegistry>().register_classes(registrations);
    }

    fn unregister_blue_classes(&mut self, ids: &[Clsid]) {
        if let Some(mut registry) = self.get_resource_mut::<BlueTypesRegistry>() {
            registry.unregister_classes(ids);
        }
    }

    fn get_blue_class_registration(&self, id: &Clsid) -> Option<&ClassRegistration> {
        self.get_resource::<BlueTypesRegistry>()?.factory(id)
    }

    fn get_blue_class_info(&self, id: &Clsid) -> Option<&ClassInfo> {
        self.get_resource::<BlueTypesRegistry>()?.class(id)
    }

    fn find_blue_clsid(&self, module: &str, name: &str) -> Option<Clsid> {
        self.get_resource::<BlueTypesRegistry>()?
            .find_clsid(module, name)
            .cloned()
    }

    fn find_blue_clsid_into(&self, out: &mut Clsid, module: &str, name: &str) -> bool {
        if let Some(id) = self.find_blue_clsid(module, name) {
            *out = id;
            true
        } else {
            false
        }
    }

    fn find_blue_clsid_by_name(&self, name: &str) -> Option<Clsid> {
        self.get_resource::<BlueTypesRegistry>()?
            .find_clsid_by_name(name)
            .cloned()
    }

    fn create_blue_instance(
        &mut self,
        class: &Clsid,
        requested_interface: &Iid,
    ) -> Result<Entity, BlueCreateError> {
        blue_types::create_registered_entity(self, class, requested_interface)
    }

    fn create_blue_instance_from_name(
        &mut self,
        name: &str,
        requested_interface: &Iid,
    ) -> Result<Entity, BlueCreateError> {
        blue_types::create_registered_entity_by_name(self, name, requested_interface)
    }

    fn query_blue_interface(&self, entity: Entity, requested_interface: &Iid) -> Option<Entity> {
        let object = self.get::<BlueObject>(entity)?;
        let registry = self.get_resource::<BlueTypesRegistry>()?;
        let mut parent = Some(&object.class_id);
        let mut seen = HashSet::default();

        while let Some(id) = parent {
            if !seen.insert(id.hash()) {
                break; // Corrupt/cyclic parent chain.
            }
            let info = registry.class(id)?;
            if &info.iid == requested_interface {
                return Some(entity);
            }
            if let Some(InterfaceEntry { component_id, .. }) = info
                .interfaces
                .iter()
                .find(|entry| &entry.iid == requested_interface)
            {
                if self.get_entity(entity).ok()?.contains_id(*component_id) {
                    return Some(entity);
                }
            }
            parent = info.parent.as_ref();
        }
        None
    }

    fn find_blue_variable(&self, class: &Clsid, name: &str) -> Option<&VarEntry> {
        self.get_resource::<BlueTypesRegistry>()?.find_variable(class, name)
    }

    fn blue_interface_component_id(&self, class: &Clsid, iid: &Iid) -> Option<ComponentId> {
        let registry = self.get_resource::<BlueTypesRegistry>()?;
        let mut next = Some(class);
        let mut seen = HashSet::default();
        while let Some(id) = next {
            if !seen.insert(id.hash()) {
                return None;
            }
            let info = registry.class(id)?;
            if let Some(component) = info.interface_component_id(iid) {
                return Some(component);
            }
            next = info.parent.as_ref();
        }
        None
    }

    fn update_blue_object_count(
        &mut self,
        entity: Entity,
        instance_delta: i32,
        lock_delta: i32,
    ) -> Result<BlueClassCount, BlueCountError> {
        if self.get_entity(entity).is_err() {
            return Err(BlueCountError::MissingObject);
        }
        let class = self.get::<BlueObject>(entity)
            .ok_or(BlueCountError::MissingClassTag)?
            .class_id.clone();
        self.init_resource::<BlueClassCounts>();
        let mut counts = self.resource_mut::<BlueClassCounts>();
        let old = counts.get(&class);
        // Both deltas are validated before either value is committed.
        let updated = BlueClassCount {
            live: checked_delta(old.live, instance_delta)?,
            locks: checked_delta(old.locks, lock_delta)?,
        };
        let _ = counts.counts.insert(class, updated);
        Ok(updated)
    }

    fn copy_blue_to(
        &mut self,
        source: Entity,
        destination: Option<Entity>,
    ) -> Result<Entity, BlueCopyError> {
        self.init_resource::<BlueCopyState>();
        self.resource_scope(|world, mut copier: bevy::ecs::change_detection::Mut<BlueCopyState>| {
            copier.0.copy_to(world, source, destination)
        })
    }

    fn clone_blue_to(
        &mut self,
        source: Entity,
        destination: Option<Entity>,
    ) -> Result<Entity, BlueCopyError> {
        self.init_resource::<BlueCopyState>();
        self.resource_scope(|world, mut copier: bevy::ecs::change_detection::Mut<BlueCopyState>| {
            copier.0.clone_to(world, source, destination)
        })
    }

    fn set_blue_copy_callbacks(
        &mut self,
        override_callback: Option<CopyOverrideCallback>,
        post_copy_callback: Option<PostCopyCallback>,
    ) {
        self.init_resource::<BlueCopyState>();
        let mut copy_state = self.resource_mut::<BlueCopyState>();
        copy_state.0.set_copy_override_callback(override_callback);
        copy_state.0.set_post_copy_callback(post_copy_callback);
    }

    fn register_blue_custom_assignment(
        &mut self,
        class: Clsid,
        callback: CustomAssignmentCallback,
    ) {
        self.init_resource::<BlueCopyState>();
        self.resource_mut::<BlueCopyState>()
            .0.register_custom_assignment(class, callback);
    }

    fn process_blue_pending_deletes(&mut self) -> usize {
        let budget = self.get_resource::<BlueClassesConfig>()
            .map_or(64, |config| config.pending_delete_budget);
        blue_types::process_pending_blue_deletes(self, budget)
    }

    fn process_all_blue_pending_deletes(&mut self) -> usize {
        blue_types::process_all_pending_blue_deletes(self)
    }

    fn set_blue_pending_deletes_enabled(&mut self, enabled: bool) {
        self.init_resource::<BluePendingDeletes>();
        self.resource_mut::<BluePendingDeletes>().set_enabled(enabled);
    }

    fn blue_pending_deletes_enabled(&self) -> bool {
        self.get_resource::<BluePendingDeletes>()
            .is_none_or(BluePendingDeletes::is_enabled)
    }
}

/// A short alias for the C++ `BlueCopy` convenience wrappers.
///
/// Unlike the C++ overloads, the caller already has the real Bevy `Entity`
/// identity (no `GetRawRoot` or `GetRootObject` branch is required).
pub fn blue_copy(world: &mut World, source: Entity) -> Result<Entity, BlueCopyError> {
    world.copy_blue_to(source, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        blue_string_table::BlueStringTable,
        blue_types::{
            BlueLinks, BlueTypesPlugin, ClassRegistrationFlags,
            CopyOverrideResult, EditFlags, VarType,
        },
    };
    use bevy::prelude::{Component, Resource};

    #[derive(Component, Clone, Debug, PartialEq, Eq)]
    struct Exposed;

    #[derive(Component, Clone, Debug, PartialEq, Eq)]
    struct Payload(u32);

    fn spawn_exposed(world: &mut World, requested: &Iid) -> Option<Entity> {
        (requested.name() == "IExposed").then(|| world.spawn(Exposed).id())
    }

    fn make_registration(
        table: &mut BlueStringTable,
        name: &str,
    ) -> (ClassInfo, ClassRegistration, Clsid, Iid) {
        let id = Clsid::new(table, "module", name);
        let iid = Iid::new(table, "IExposed");
        let info = ClassInfo::new(table, id.clone(), iid.clone(), name);
        let registration = ClassRegistration {
            create_instance: spawn_exposed,
            flags: ClassRegistrationFlags::default(),
        };
        (info, registration, id, iid)
    }

    fn make_world() -> World {
        let mut world = World::new();
        // These exact resources are the native domain boundary; no global state.
        world.init_resource::<BlueTypesRegistry>();
        world.init_resource::<BlueCopyState>();
        world.init_resource::<BlueClassCounts>();
        world.init_resource::<BlueClassesConfig>();
        world.init_resource::<BluePendingDeletes>();
        world
    }

    #[test]
    fn registration_lookup_and_unregistration_round_trip() {
        let mut world = make_world();
        let mut strings = BlueStringTable::default();
        let (class, factory, id, iid) = make_registration(&mut strings, "First");
        assert!(world.get_blue_class_registration(&id).is_none());
        world.register_blue_classes(&[(class, factory)]);
        assert_eq!(world.find_blue_clsid("module", "First"), Some(id.clone()));
        assert_eq!(world.find_blue_clsid_by_name("First"), Some(id.clone()));
        assert!(world.get_blue_class_info(&id).is_some());
        assert!(world.get_blue_class_registration(&id).is_some());
        assert!(world.find_blue_clsid("other", "First").is_none());
        let mut out = Clsid::default();
        assert!(world.find_blue_clsid_into(&mut out, "module", "First"));
        assert_eq!(out, id);
        assert!(!world.find_blue_clsid_into(&mut out, "missing", "First"));
        assert_eq!(out, id);
        assert!(world.create_blue_instance(&id, &iid).is_ok());
        world.unregister_blue_classes(&[id.clone()]);
        assert!(world.get_blue_class_registration(&id).is_none());
        assert!(world.get_blue_class_info(&id).is_none());
        assert!(world.find_blue_clsid_by_name("First").is_none());
    }

    #[test]
    fn creation_respects_iid_factory_and_registers_blue_object() {
        let mut world = make_world();
        let mut strings = BlueStringTable::default();
        let (class, factory, id, iid) = make_registration(&mut strings, "Enemy");
        world.register_blue_classes(&[(class, factory)]);
        let created = world.create_blue_instance_from_name("Enemy", &iid).unwrap();
        assert_eq!(world.get::<BlueObject>(created).unwrap().class_id, id);
        assert_eq!(world.query_blue_interface(created, &iid), Some(created));
        assert!(world.get::<Exposed>(created).is_some());
        let bad = Iid::new(&mut strings, "Unsupported");
        assert_eq!(
            world.create_blue_instance(&id, &bad),
            Err(BlueCreateError::UnsupportedInterface)
        );
    }

    #[test]
    fn colliding_name_only_clsid_hashes_follow_existing_registry_identity() {
        let mut world = make_world();
        let mut strings = BlueStringTable::default();
        let (a, factory, id, _) = make_registration(&mut strings, "Player");
        let mut b = a.clone();
        b.class_id = Clsid::new(&mut strings, "other", "Player");
        world.register_blue_classes(&[(a, factory), (b, factory)]);
        // Carbon's class ID equality is *hash-only*. The existing registry
        // represents this as one identity instead of inventing a new key.
        assert_eq!(world.resource::<BlueTypesRegistry>().len(), 1);
        assert_eq!(world.find_blue_clsid_by_name("Player"), Some(id));
    }

    #[test]
    fn inherited_interface_query_checks_component_presence() {
        let mut world = make_world();
        let mut strings = BlueStringTable::default();
        let (mut base, factory, base_id, interface) =
            make_registration(&mut strings, "Base");
        let actual_component = world.register_component::<Exposed>();
        let extra_iid = Iid::new(&mut strings, "IAdditional");
        base.interfaces.push(InterfaceEntry {
            iid: extra_iid.clone(),
            component_id: actual_component,
        });
        let child_id = Clsid::new(&mut strings, "module", "Child");
        let other_iid = Iid::new(&mut strings, "IChild");
        let mut child = ClassInfo::new(
            &mut strings,
            child_id.clone(),
            other_iid,
            "child class",
        );
        child.parent = Some(base_id.clone());
        world.register_blue_classes(&[(base, factory), (child, factory)]);
        let entity = world.spawn((BlueObject::new(child_id.clone()), Exposed)).id();
        assert_eq!(
            world.blue_interface_component_id(&child_id, &extra_iid),
            Some(actual_component)
        );
        assert_eq!(world.query_blue_interface(entity, &extra_iid), Some(entity));
        // The class's primary IID is still inherited even without this extra component.
        assert_eq!(world.query_blue_interface(entity, &interface), Some(entity));
        world.entity_mut(entity).remove::<Exposed>();
        assert_eq!(world.query_blue_interface(entity, &extra_iid), None);
    }

    #[test]
    fn member_lookup_walks_parent_classes_without_fabricated_offsets() {
        let mut world = make_world();
        let mut strings = BlueStringTable::default();
        let (mut base, factory, base_id, _) =
            make_registration(&mut strings, "BaseThing");
        base.members.push(VarEntry {
            name: strings.intern("health"),
            var_type: VarType::Long,
            size_bytes: 4,
            iid: None,
            description: strings.intern("Hit points"),
            edit_flags: EditFlags::default(),
            choices: Vec::new(),
        });
        let child_id = Clsid::new(&mut strings, "module", "ChildThing");
        let child_iid = Iid::new(&mut strings, "IChildThing");
        let mut child = ClassInfo::new(&mut strings, child_id.clone(), child_iid, "child");
        child.parent = Some(base_id);
        world.register_blue_classes(&[(base, factory), (child, factory)]);
        assert_eq!(
            world.find_blue_variable(&child_id, "health").map(|v| v.var_type),
            Some(VarType::Long)
        );
        assert!(world.find_blue_variable(&child_id, "missing").is_none());
    }

    #[test]
    fn object_counts_check_both_deltas_atomically() {
        let mut world = make_world();
        let mut strings = BlueStringTable::default();
        let class = Clsid::new(&mut strings, "module", "Counted");
        let entity = world.spawn(BlueObject::new(class.clone())).id();
        let one = world.update_blue_object_count(entity, 1, 1).unwrap();
        assert_eq!(one, BlueClassCount { live: 1, locks: 1 });
        assert_eq!(world.update_blue_object_count(entity, 0, -2), Err(BlueCountError::Underflow));
        assert_eq!(world.resource::<BlueClassCounts>().get(&class), one);
        assert_eq!(
            world.update_blue_object_count(entity, -1, -1).unwrap(),
            BlueClassCount::default()
        );
        assert_eq!(world.update_blue_object_count(entity, -1, 0), Err(BlueCountError::Underflow));
        let _ = world.resource_mut::<BlueClassCounts>().counts.insert(
            class.clone(), BlueClassCount { live: u32::MAX, locks: 0 },
        );
        assert_eq!(world.update_blue_object_count(entity, 1, 0), Err(BlueCountError::Overflow));
        assert_eq!(world.resource::<BlueClassCounts>().get(&class).live, u32::MAX);
    }

    #[test]
    fn count_update_rejects_missing_entity_and_missing_class_component() {
        let mut world = make_world();
        let without_tag = world.spawn_empty().id();
        assert_eq!(world.update_blue_object_count(without_tag, 1, 0), Err(BlueCountError::MissingClassTag));
        assert!(world.despawn(without_tag));
        assert_eq!(world.update_blue_object_count(without_tag, 1, 0), Err(BlueCountError::MissingObject));
    }

    #[test]
    fn copy_to_duplicates_shared_links_clone_to_preserves_them() {
        let mut world = make_world();
        let shared = world.spawn(Payload(11)).id();
        let root = world.spawn((Payload(1), BlueLinks(vec![shared, shared]))).id();
        let copy = world.copy_blue_to(root, None).unwrap();
        let copy_children = &world.get::<BlueLinks>(copy).unwrap().0;
        assert_ne!(copy_children[0], copy_children[1]);
        assert_eq!(world.get::<Payload>(copy_children[0]), Some(&Payload(11)));
        let clone = world.clone_blue_to(root, None).unwrap();
        let clone_children = &world.get::<BlueLinks>(clone).unwrap().0;
        assert_eq!(clone_children[0], clone_children[1]);
        assert_ne!(clone_children[0], shared);
    }

    #[test]
    fn clone_to_preserves_cycles_and_copy_to_reports_cycles() {
        let mut world = make_world();
        let source = world.spawn(BlueLinks::default()).id();
        world.entity_mut(source).insert(BlueLinks(vec![source]));
        let cloned = world.clone_blue_to(source, None).unwrap();
        assert_eq!(world.get::<BlueLinks>(cloned).unwrap().0, vec![cloned]);
        assert_eq!(world.copy_blue_to(source, None), Err(BlueCopyError::CycleRequiresCloneTo));
    }

    #[derive(Resource, Default)]
    struct HookCalls(usize);

    fn override_fallback(_: &mut World, _: Entity, _: Option<Entity>) -> CopyOverrideResult {
        CopyOverrideResult::Fallback
    }

    fn post_copy(world: &mut World, _: Entity, _: Entity) {
        world.resource_mut::<HookCalls>().0 += 1;
    }

    fn custom_assignment(world: &mut World, source: Entity, target: Entity) -> bool {
        let Some(number) = world.get::<Payload>(source).map(|payload| payload.0) else {
            return false;
        };
        world.entity_mut(target).insert(Payload(number + 1));
        true
    }

    #[test]
    fn copy_callbacks_and_custom_assignment_use_world_owned_state() {
        let mut world = make_world();
        world.insert_resource(HookCalls::default());
        let mut strings = BlueStringTable::default();
        let class = Clsid::new(&mut strings, "module", "Hooked");
        let source = world.spawn((BlueObject::new(class.clone()), Payload(40))).id();
        world.set_blue_copy_callbacks(Some(override_fallback), Some(post_copy));
        world.register_blue_custom_assignment(class, custom_assignment);
        let target = world.copy_blue_to(source, None).unwrap();
        assert_eq!(world.get::<Payload>(target), Some(&Payload(41)));
        assert_eq!(world.resource::<HookCalls>().0, 1);
        world.set_blue_copy_callbacks(None, None);
    }

    #[test]
    fn copy_can_fill_an_existing_entity_instead_of_allocating() {
        let mut world = make_world();
        let source = world.spawn(Payload(72)).id();
        let destination = world.spawn(Payload(2)).id();
        assert_eq!(world.copy_blue_to(source, Some(destination)), Ok(destination));
        assert_eq!(world.get::<Payload>(destination), Some(&Payload(72)));
        assert_eq!(world.get::<Payload>(source), Some(&Payload(72)));
    }

    #[test]
    fn pending_deletes_can_be_disabled_and_processed_by_budget() {
        let mut world = make_world();
        world.resource_mut::<BlueClassesConfig>().pending_delete_budget = 1;
        let a = world.spawn_empty().id();
        let b = world.spawn_empty().id();
        assert!(blue_types::request_blue_delete(&mut world, a));
        assert!(blue_types::request_blue_delete(&mut world, b));
        assert!(world.blue_pending_deletes_enabled());
        assert_eq!(world.process_blue_pending_deletes(), 1);
        assert!(world.get_entity(a).is_err());
        assert!(world.get_entity(b).is_ok());
        assert_eq!(world.process_all_blue_pending_deletes(), 1);
        let c = world.spawn_empty().id();
        world.set_blue_pending_deletes_enabled(false);
        assert!(!world.blue_pending_deletes_enabled());
        assert!(blue_types::request_blue_delete(&mut world, c));
        assert!(world.get_entity(c).is_err());
    }

    #[test]
    fn plugin_initializes_world_owned_resources() {
        let mut app = App::new();
        app.add_plugins((BlueTypesPlugin, BlueClassesPlugin));
        assert!(app.world().contains_resource::<BlueTypesRegistry>());
        assert!(app.world().contains_resource::<BlueCopyState>());
        assert!(app.world().contains_resource::<BlueClassCounts>());
        assert!(app.world().contains_resource::<BluePendingDeletes>());
    }

    #[test]
    fn blue_copy_convenience_uses_world_extension() {
        let mut world = make_world();
        let original = world.spawn(Payload(4)).id();
        let cloned = blue_copy(&mut world, original).unwrap();
        assert_ne!(cloned, original);
        assert_eq!(world.get::<Payload>(cloned), Some(&Payload(4)));
    }
}
