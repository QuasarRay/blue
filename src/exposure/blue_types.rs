//! Bevy-native equivalents of the domain-level contracts in `BlueTypes.h`
//! and `BlueTypes.cpp` (Carbon Blue).
//!
//! ## Parity boundary
//!
//! **Preserved:** legacy `Clsid`/`IID` hash-only comparison and ordering,
//! `Clsid`'s *name-only* hash, first-dot splitting and failed-parse reset,
//! assignment-state semantics, numeric `VARTYPE` values, edit flags, legacy
//! array-size accessors, and supported `BeCast` bit-copy semantics.
//!
//! **Adapted:** C++ pointers, virtual interfaces and raw member offsets become
//! Bevy `Entity` and `ComponentId` handles, Bevy reflection metadata, and ECS
//! lifecycle. These are NOT binary/ABI compatible with the C++ structures.
//!
//! This module expects `crate::blue_string_table::BlueStringTable`. It does
//! not add a second intern pool or an unrelated global cache.

use crate::blue_string_table::BlueStringTable;
use bevy::ecs::entity::EntityCloner;
use bevy::{
    ecs::component::ComponentId,
    platform::{
        collections::{HashMap, HashSet},
        sync::Arc,
    },
    prelude::{App, ChildOf, Children, Component, Entity, Last, Plugin, Reflect, Resource, World},
};
use core::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not},
};

/// C++ `Be::Time`: signed FILETIME-style 100 ns ticks.
///
/// This is an absolute tick representation, **not** `bevy::time::Time`,
/// which represents a frame-oriented clock.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlueTime(pub i64);

impl BlueTime {
    pub const TICKS_PER_SECOND: i64 = 10_000_000;

    #[must_use]
    pub const fn ticks(self) -> i64 {
        self.0
    }
}

/// C++ `Be::Optional<T>` is *not* Rust `Option<T>`: an omitted argument
/// still has a readable and mutable payload.
///
/// In particular, mutating `value_mut()` does NOT change `is_assigned()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Optional<T> {
    value: T,
    is_assigned: bool,
}

impl<T: Default> Default for Optional<T> {
    fn default() -> Self {
        Self::omitted(T::default())
    }
}

impl<T> Optional<T> {
    #[must_use]
    pub const fn omitted(value: T) -> Self {
        Self {
            value,
            is_assigned: false,
        }
    }

    #[must_use]
    pub const fn assigned(value: T) -> Self {
        Self {
            value,
            is_assigned: true,
        }
    }

    pub fn assign(&mut self, value: T) {
        self.value = value;
        self.is_assigned = true;
    }

    pub fn set_assigned(&mut self, assigned: bool) {
        self.is_assigned = assigned;
    }

    #[must_use]
    pub const fn is_assigned(&self) -> bool {
        self.is_assigned
    }

    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub fn into_value(self) -> T {
        self.value
    }
}

impl<T> From<T> for Optional<T> {
    fn from(value: T) -> Self {
        Self::assigned(value)
    }
}

/// Runtime representation of C++ `OptionalWithDefaultValue<T, defaultValue>`.
///
/// Rust stable generic const parameters cannot represent arbitrary `T` values.
/// Specify the C++ compile-time default when constructing the wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionalWithDefaultValue<T>(Optional<T>);

impl<T> OptionalWithDefaultValue<T> {
    #[must_use]
    pub const fn omitted(default_value: T) -> Self {
        Self(Optional::omitted(default_value))
    }

    #[must_use]
    pub const fn assigned(value: T) -> Self {
        Self(Optional::assigned(value))
    }

    #[must_use]
    pub const fn as_optional(&self) -> &Optional<T> {
        &self.0
    }

    pub fn as_optional_mut(&mut self) -> &mut Optional<T> {
        &mut self.0
    }

    pub fn into_optional(self) -> Optional<T> {
        self.0
    }
}

/// The legacy C++ array has exactly 16 slots.
pub const MAX_BLUE_EXPOSURE_ARGUMENTS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureError {
    TooManyArguments,
    TooManyOptionalArguments,
}

/// `BlueExposureFunctionSignature`, with validated counts and no dangling
/// `const char*`. `argument_count()` is derived from the stored arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlueExposureFunctionSignature {
    return_type: Arc<str>,
    argument_types: Vec<Arc<str>>,
    optional_count: u32,
}

impl BlueExposureFunctionSignature {
    pub fn try_new(
        table: &mut BlueStringTable,
        return_type: &str,
        arguments: &[&str],
        optional_count: u32,
    ) -> Result<Self, SignatureError> {
        if arguments.len() > MAX_BLUE_EXPOSURE_ARGUMENTS {
            return Err(SignatureError::TooManyArguments);
        }
        if usize::try_from(optional_count).unwrap_or(usize::MAX) > arguments.len() {
            return Err(SignatureError::TooManyOptionalArguments);
        }

        Ok(Self {
            return_type: table.intern(return_type),
            argument_types: arguments.iter().map(|arg| table.intern(arg)).collect(),
            optional_count,
        })
    }

    #[must_use]
    pub fn return_type(&self) -> &str {
        &self.return_type
    }

    #[must_use]
    pub fn argument_types(&self) -> &[Arc<str>] {
        &self.argument_types
    }

    #[must_use]
    pub fn argument_count(&self) -> u32 {
        self.argument_types.len() as u32
    }

    #[must_use]
    pub const fn optional_count(&self) -> u32 {
        self.optional_count
    }
}

/// The Carbon C++ `CcpHashFNV1` implementation loads `int8_t`, and therefore
/// sign-extends bytes >= 0x80 before XOR. Ordinary Rust byte-wise FNV-1 differs
/// for non-ASCII strings. Source: carbonengine/core/CcpHash.cpp.
///
/// As assumed by the supplied `blue_string_table.rs`, include the terminating
/// NUL in the hash input. The original `BlueStringTable.h` implementation
/// must still be checked to certify the precise input length.
fn cpp_fnv1_with_nul(input: &str) -> u32 {
    input
        .bytes()
        .chain(core::iter::once(0))
        .fold(0x811c_9dc5_u32, |hash, byte| {
            hash.wrapping_mul(0x0100_0193) ^ (i32::from(byte as i8) as u32)
        })
}

fn c_string_prefix(input: &str) -> &str {
    input.split_once('\0').map_or(input, |(prefix, _)| prefix)
}

/// Carbon `Be::Clsid`. Identity, ordering and hashing intentionally use
/// **only the 32-bit class-name FNV hash**, as in the supplied C++ code.
///
/// Different module names (and even actual hash collisions) are therefore
/// indistinguishable for legacy equality. Do not "fix" this without an
/// explicit compatibility-breaking migration.
#[derive(Debug, Clone)]
pub struct Clsid {
    module: Arc<str>,
    name: Arc<str>,
    hash: u32,
}

impl Default for Clsid {
    fn default() -> Self {
        Self {
            module: Arc::from(""),
            name: Arc::from(""),
            hash: 0,
        }
    }
}

impl Clsid {
    #[must_use]
    pub fn new(table: &mut BlueStringTable, module: &str, name: &str) -> Self {
        // C++ receives NUL-terminated `const char*`: bytes after the first NUL
        // are invisible to `GetString` and `CcpHashFNV1`.
        let module = table.intern(c_string_prefix(module));
        let name = c_string_prefix(name);
        let hash = cpp_fnv1_with_nul(name);
        let name = table.intern(name);
        Self { module, name, hash }
    }

    /// Parse at the *first* dot, exactly as `InitFromString` does in C++.
    ///
    /// On failure, the instance becomes the legacy invalid `Clsid`.
    pub fn init_from_string(&mut self, table: &mut BlueStringTable, input: &str) -> bool {
        *self = Self::default();
        let Some((module, name)) = c_string_prefix(input).split_once('.') else {
            return false;
        };
        *self = Self::new(table, module, name);
        true
    }

    #[must_use]
    pub fn module(&self) -> &str {
        &self.module
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn hash(&self) -> u32 {
        self.hash
    }

    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.hash != 0
    }
}

impl PartialEq for Clsid {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for Clsid {}

impl Hash for Clsid {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialOrd for Clsid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Clsid {
    fn cmp(&self, other: &Self) -> Ordering {
        self.hash.cmp(&other.hash)
    }
}

/// Carbon `Be::IID`. The C++ comparison, ordering and truth test use only
/// the name's 32-bit hash. Input is not split on dots.
#[derive(Debug, Clone)]
pub struct Iid {
    name: Arc<str>,
    hash: u32,
}

impl Iid {
    #[must_use]
    pub fn new(table: &mut BlueStringTable, name: &str) -> Self {
        let name = c_string_prefix(name);
        let hash = cpp_fnv1_with_nul(name);
        let name = table.intern(name);
        Self { name, hash }
    }

    /// C++ `IID::InitFromString` always returns `true` for a valid C string.
    pub fn init_from_string(&mut self, table: &mut BlueStringTable, input: &str) -> bool {
        *self = Self::new(table, input);
        true
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn hash(&self) -> u32 {
        self.hash
    }

    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.hash != 0
    }
}

impl PartialEq for Iid {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for Iid {}

impl Hash for Iid {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialOrd for Iid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Iid {
    fn cmp(&self, other: &Self) -> Ordering {
        self.hash.cmp(&other.hash)
    }
}

/// The exact numeric C++ `Be::VARTYPE` tags.
/// The holes 18, 19 and 26..=30 are intentionally not assigned.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VarType {
    Invalid = 0,
    Long = 1,
    Float = 2,
    Double = 3,
    Bool = 4,
    Iroot = 5,
    IrootPtr = 6,
    CharArray = 7,
    CString = 8,
    Int64 = 9,
    PyObject = 10,
    PyObjectPtr = 11,
    Reference = 12,
    WCString = 13,
    WReference = 14,
    FloatArray = 15,
    DoubleArray = 16,
    IntArray = 17,
    RotReference = 20,
    IrootWeakRef = 21,
    PyProperty = 22,
    StdString = 23,
    StdWString = 24,
    BinaryBlock = 25,
    Byte = 31,
    Short = 32,
    ScriptCallback = 33,
    SharedString = 34,
    SharedStringW = 35,
    Ulong = 36,
    Uint64 = 37,
    ForceLong = 0xffff_ffff,
}

impl VarType {
    pub const MAX: u32 = 37;
}

impl TryFrom<u32> for VarType {
    type Error = ();

    fn try_from(raw: u32) -> Result<Self, Self::Error> {
        let value = match raw {
            0 => Self::Invalid,
            1 => Self::Long,
            2 => Self::Float,
            3 => Self::Double,
            4 => Self::Bool,
            5 => Self::Iroot,
            6 => Self::IrootPtr,
            7 => Self::CharArray,
            8 => Self::CString,
            9 => Self::Int64,
            10 => Self::PyObject,
            11 => Self::PyObjectPtr,
            12 => Self::Reference,
            13 => Self::WCString,
            14 => Self::WReference,
            15 => Self::FloatArray,
            16 => Self::DoubleArray,
            17 => Self::IntArray,
            20 => Self::RotReference,
            21 => Self::IrootWeakRef,
            22 => Self::PyProperty,
            23 => Self::StdString,
            24 => Self::StdWString,
            25 => Self::BinaryBlock,
            31 => Self::Byte,
            32 => Self::Short,
            33 => Self::ScriptCallback,
            34 => Self::SharedString,
            35 => Self::SharedStringW,
            36 => Self::Ulong,
            37 => Self::Uint64,
            0xffff_ffff => Self::ForceLong,
            _ => return Err(()),
        };
        Ok(value)
    }
}

/// `EDITFLAGS` as composable, losslessly retained raw bits. The source C++
/// flags are a bitmask, not mutually exclusive enum variants.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EditFlags(u32);

impl EditFlags {
    pub const NONE: Self = Self(0);
    pub const READ: Self = Self(0x001);
    pub const WRITE: Self = Self(0x002);
    pub const READWRITE: Self = Self(0x003);
    pub const NOTIFY: Self = Self(0x004);
    pub const HIDDEN: Self = Self(0x008);
    pub const MODMASK: Self = Self(0x00f);
    pub const PERSIST: Self = Self(0x010);
    pub const RPERSIST: Self = Self(0x020);
    pub const SERMASK: Self = Self(0x0f0);
    pub const FLAGS: Self = Self(0x100);
    pub const ENUM: Self = Self(0x200);
    pub const EDMASK: Self = Self(0xf00);
    pub const PERSISTONLY: Self = Self(0x018);
    pub const EDIT_FORCELONG: Self = Self(u32::MAX);

    #[must_use]
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }

    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[must_use]
    pub const fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

impl BitOr for EditFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for EditFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for EditFlags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for EditFlags {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Not for EditFlags {
    type Output = Self;

    fn not(self) -> Self {
        Self(!self.0)
    }
}

/// Type-safe native counterpart of the C++ untagged `Be::Var` union.
/// `Entity` is a Bevy identity, not an `IRoot*` ABI pointer.
#[derive(Debug, Clone, PartialEq)]
pub enum Var {
    Double(f64),
    Long(i32),
    Ulong(u32),
    Float(f32),
    Bool(bool),
    Byte(u8),
    Short(u16),
    Entity(Entity),
    Utf8(Arc<str>),
    Wide(Arc<str>),
    Int64(i64),
    Uint64(u64),
}

impl Var {
    /// Returns the matching legacy scalar tag. Pointer-backed strings use
    /// the closest respective string tag; the original union has no tag.
    #[must_use]
    pub const fn var_type(&self) -> VarType {
        match self {
            Self::Double(_) => VarType::Double,
            Self::Long(_) => VarType::Long,
            Self::Ulong(_) => VarType::Ulong,
            Self::Float(_) => VarType::Float,
            Self::Bool(_) => VarType::Bool,
            Self::Byte(_) => VarType::Byte,
            Self::Short(_) => VarType::Short,
            Self::Entity(_) => VarType::IrootPtr,
            Self::Utf8(_) => VarType::CString,
            Self::Wide(_) => VarType::WCString,
            Self::Int64(_) => VarType::Int64,
            Self::Uint64(_) => VarType::Uint64,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarChooser {
    pub key: Arc<str>,
    pub value: Var,
    pub description: Arc<str>,
}

/// Replaces C++ `IID*` plus multiple-inheritance byte offsets with Bevy's
/// registered `ComponentId`: no vtable or pointer offset is exposed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceEntry {
    pub iid: Iid,
    pub component_id: ComponentId,
}

/// Reflectable member metadata. `name` identifies the corresponding field
/// on the registered Bevy `Reflect` type; reflection access is performed by
/// callers using Bevy's `Reflect` / `TypeRegistry`, not raw byte offsets.
#[derive(Debug, Clone, PartialEq)]
pub struct VarEntry {
    pub name: Arc<str>,
    pub var_type: VarType,
    /// Original `mSize` in bytes; used for the legacy array-size helpers.
    pub size_bytes: usize,
    pub iid: Option<Iid>,
    pub description: Arc<str>,
    pub edit_flags: EditFlags,
    pub choices: Vec<VarChooser>,
}

impl VarEntry {
    #[must_use]
    pub fn float_array_size(&self) -> usize {
        if self.var_type == VarType::FloatArray {
            self.size_bytes / size_of::<f32>()
        } else {
            1
        }
    }

    #[must_use]
    pub fn double_array_size(&self) -> usize {
        if self.var_type == VarType::DoubleArray {
            self.size_bytes / size_of::<f64>()
        } else {
            1
        }
    }

    /// Assumes 32-bit C++ `int`, as in the original supported targets.
    #[must_use]
    pub fn int_array_size(&self) -> usize {
        if self.var_type == VarType::IntArray {
            self.size_bytes / size_of::<i32>()
        } else {
            1
        }
    }
}

/// The table entry itself is not `#[derive(Reflect)]`: it is the metadata
/// *about* Bevy-reflected components. Use Bevy's `AppTypeRegistry` for actual
/// field access and type registration.
#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub class_id: Clsid,
    pub iid: Iid,
    pub description: Arc<str>,
    pub interfaces: Vec<InterfaceEntry>,
    pub members: Vec<VarEntry>,
    pub parent: Option<Clsid>,
    pub function_signatures: HashMap<Arc<str>, BlueExposureFunctionSignature>,
}

impl ClassInfo {
    #[must_use]
    pub fn new(
        table: &mut BlueStringTable,
        class_id: Clsid,
        iid: Iid,
        description: &str,
    ) -> Self {
        Self {
            class_id,
            iid,
            description: table.intern(description),
            interfaces: Vec::new(),
            members: Vec::new(),
            parent: None,
            function_signatures: HashMap::default(),
        }
    }

    /// Resolves an interface by its legacy hash ID to a registered Bevy
    /// `ComponentId`. First match wins, as for an ordered C++ table.
    #[must_use]
    pub fn interface_component_id(&self, iid: &Iid) -> Option<ComponentId> {
        self.interfaces
            .iter()
            .find(|entry| &entry.iid == iid)
            .map(|entry| entry.component_id)
    }
}

/// Native Bevy factory, replacing C++ `bool(const IID&, void**)`.
/// The returned handle is a generational ECS entity identifier.
pub type CreateInstance = fn(&mut World, &Iid) -> Option<Entity>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClassRegistrationFlags(u32);

impl ClassRegistrationFlags {
    /// Preserved legacy flag bit; Python policy belongs in a separate PyO3 layer.
    pub const DISABLE_PYTHON_CONSTRUCTION: Self = Self(1);

    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }

    #[must_use]
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }

    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ClassRegistration {
    pub create_instance: CreateInstance,
    pub flags: ClassRegistrationFlags,
}

/// Runtime metadata in a World, rather than an out-of-band global C++ table.
#[derive(Resource, Default, Debug)]
pub struct BlueTypesRegistry {
    classes: HashMap<Clsid, ClassInfo>,
    constructors: HashMap<Clsid, ClassRegistration>,
}

impl BlueTypesRegistry {
    pub fn register_class(&mut self, class: ClassInfo) -> Option<ClassInfo> {
        self.classes.insert(class.class_id.clone(), class)
    }

    #[must_use]
    pub fn class(&self, id: &Clsid) -> Option<&ClassInfo> {
        self.classes.get(id)
    }

    /// Register a complete class/factory pair. The constructor is only called
    /// after the caller asks for this class explicitly.
    pub fn register(&mut self, class: ClassInfo, factory: ClassRegistration) {
        let id = class.class_id.clone();
        let _ = self.register_class(class);
        self.register_factory(id, factory);
    }

    /// Register a slice (unlike the C++ null-terminated array, no sentinel).
    pub fn register_classes(&mut self, registrations: &[(ClassInfo, ClassRegistration)]) {
        for (class, constructor) in registrations {
            self.register(class.clone(), *constructor);
        }
    }

    /// Unregisters both metadata and its constructor.
    pub fn unregister_class(&mut self, id: &Clsid) -> Option<ClassInfo> {
        let _ = self.constructors.remove(id);
        self.classes.remove(id)
    }

    pub fn unregister_classes(&mut self, ids: &[Clsid]) {
        for id in ids {
            let _ = self.unregister_class(id);
        }
    }

    /// Exact textual lookup, independent of iteration order in `HashMap`.
    /// Note that different module names can nevertheless share a legacy hash.
    #[must_use]
    pub fn find_clsid(&self, module: &str, name: &str) -> Option<&Clsid> {
        self.classes
            .values()
            .find(|info| info.class_id.module() == module && info.class_id.name() == name)
            .map(|info| &info.class_id)
    }

    #[must_use]
    pub fn find_clsid_by_name(&self, name: &str) -> Option<&Clsid> {
        self.classes
            .values()
            .find(|info| info.class_id.name() == name)
            .map(|info| &info.class_id)
    }

    /// Lookup in the class followed by its parent chain.
    /// C++ member byte offsets do not apply to a Bevy ECS component.
    #[must_use]
    pub fn find_variable<'a>(&'a self, class: &Clsid, name: &str) -> Option<&'a VarEntry> {
        let mut current = self.class(class);
        let mut visited = HashSet::default();
        while let Some(info) = current {
            if !visited.insert(info.class_id.hash()) {
                return None;
            }
            if let Some(variable) = info.members.iter().find(|field| field.name.as_ref() == name) {
                return Some(variable);
            }
            current = info.parent.as_ref().and_then(|parent| self.class(parent));
        }
        None
    }

    pub fn register_factory(&mut self, id: Clsid, factory: ClassRegistration) {
        let _ = self.constructors.insert(id, factory);
    }

    #[must_use]
    pub fn factory(&self, id: &Clsid) -> Option<&ClassRegistration> {
        self.constructors.get(id)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.classes.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.classes.is_empty()
    }
}

/// Native `IRoot` identity tag. Storage, ownership and deletion are handled
/// by Bevy's `World` and its component lifecycle, not manual `Lock/Unlock`.
#[derive(Component, Debug, Clone)]
pub struct BlueObject {
    pub class_id: Clsid,
    pub flags: u32,
}

impl BlueObject {
    #[must_use]
    pub fn new(class_id: Clsid) -> Self {
        Self { class_id, flags: 0 }
    }
}

/// Marker for objects that participate in Bevy-native weak referencing.
#[derive(Component, Reflect, Debug, Default, Clone, Copy)]
#[reflect(Component)]
pub struct BlueWeakObject;

/// Native weak identity: a generational `Entity` is not owning.
/// It does not provide the C++ `IWeakRef::WeakRefNotify` callback or resurrection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlueWeakRef(Entity);

impl BlueWeakRef {
    #[must_use]
    pub const fn new(entity: Entity) -> Self {
        Self(entity)
    }

    #[must_use]
    pub fn resolve(self, world: &World) -> Option<Entity> {
        world.get::<BlueWeakObject>(self.0).map(|_| self.0)
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LoadObjectInitFlag {
    #[default]
    Initialize = 0,
    DontInitialize = 1,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum QueryInterfaceOption {
    #[default]
    Normal = 0,
    Silent = 1,
}

pub const BLUE_RT_AUTOVAR: u32 = 0x1;

/// C++ `BeCast` copies the source's bytes into a zero-initialized `double`.
/// This is not numeric conversion. We implement the documented scalar case,
/// restricted to native little-endian memory layouts; no `unsafe` or padding.
#[must_use]
pub fn be_cast_f32(value: f32) -> f64 {
    be_cast_u32(value.to_bits())
}

#[must_use]
pub fn be_cast_u32(value: u32) -> f64 {
    if cfg!(target_endian = "little") {
        f64::from_bits(u64::from(value))
    } else {
        f64::from_bits(u64::from(value) << 32)
    }
}

#[must_use]
pub fn be_cast_i32(value: i32) -> f64 {
    be_cast_u32(value as u32)
}

#[must_use]
pub fn be_cast_u64(value: u64) -> f64 {
    f64::from_bits(value)
}

#[must_use]
pub fn be_cast_i64(value: i64) -> f64 {
    f64::from_bits(value as u64)
}

/// Initializes both the supplied `BlueStringTable` and this metadata registry.
/// `BlueStringTablePlugin` can also be registered on its own.
pub struct BlueTypesPlugin;

impl Plugin for BlueTypesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlueStringTable>()
            .init_resource::<BlueTypesRegistry>()
            .init_resource::<BluePendingDeletes>()
            .register_type::<BlueWeakObject>()
            .add_systems(Last, process_pending_deletes_system);
    }
}


/// IBlueClasses::CreateInstance, represented using Bevy's generational Entity ID.
/// A factory receives an IID and can decline unsupported interfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlueCreateError {
    UnregisteredClass,
    UnsupportedInterface,
    InvalidFactoryEntity,
    ClassMismatch,
}

pub fn create_registered_entity(
    world: &mut World,
    clsid: &Clsid,
    iid: &Iid,
) -> Result<Entity, BlueCreateError> {
    let registration = world
        .get_resource::<BlueTypesRegistry>()
        .and_then(|registry| registry.factory(clsid))
        .copied()
        .ok_or(BlueCreateError::UnregisteredClass)?;

    let entity = (registration.create_instance)(world, iid)
        .ok_or(BlueCreateError::UnsupportedInterface)?;
    if world.get_entity(entity).is_err() {
        return Err(BlueCreateError::InvalidFactoryEntity);
    }
    if let Some(existing) = world.get::<BlueObject>(entity) {
        if existing.class_id != *clsid {
            return Err(BlueCreateError::ClassMismatch);
        }
    } else {
        world.entity_mut(entity).insert(BlueObject::new(clsid.clone()));
    }
    Ok(entity)
}

pub fn create_registered_entity_by_name(
    world: &mut World,
    class_name: &str,
    iid: &Iid,
) -> Result<Entity, BlueCreateError> {
    let clsid = world
        .get_resource::<BlueTypesRegistry>()
        .and_then(|registry| registry.find_clsid_by_name(class_name))
        .cloned()
        .ok_or(BlueCreateError::UnregisteredClass)?;
    create_registered_entity(world, &clsid, iid)
}

/// IBlueClasses::QueryThisInterface for native ECS components.
/// Does not yield a C++ multiple-inheritance-adjusted pointer.
#[must_use]
pub fn query_this_interface(world: &World, entity: Entity, iid: &Iid) -> Option<Entity> {
    let object = world.get::<BlueObject>(entity)?;
    let registry = world.get_resource::<BlueTypesRegistry>()?;
    let class = registry.class(&object.class_id)?;
    if let Some(component_id) = class.interface_component_id(iid) {
        return world
            .get_entity(entity)
            .ok()?
            .contains_id(component_id)
            .then_some(entity);
    }
    (&class.iid == iid).then_some(entity)
}

/// Non-owning directed edges between Bevy entities. This component models
/// multiply referenced Carbon `Child` objects, which Bevy's single-parent
/// `ChildOf` relationship cannot represent on its own.
#[derive(Component, Debug, Default, Clone, PartialEq, Eq)]
pub struct BlueLinks(pub Vec<Entity>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlueCopyError {
    MissingSource,
    MissingDestination,
    SourceEqualsDestination,
    ClassMismatch,
    CycleRequiresCloneTo,
    OverrideFailed,
    InvalidOverrideTarget,
}

/// C++ ICopier::OverrideResult, with a Bevy destination returned on success.
/// Callback overrides must not despawn the source during a copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyOverrideResult {
    Success(Entity),
    Failure,
    Fallback,
}

pub type CopyOverrideCallback = fn(&mut World, Entity, Option<Entity>) -> CopyOverrideResult;
pub type PostCopyCallback = fn(&mut World, Entity, Entity);
/// ICopierCustomAssignment::AssignTo. Return `false` to reject the copy.
pub type CustomAssignmentCallback = fn(&mut World, Entity, Entity) -> bool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GraphCopyMode {
    DuplicateSharedChildren,
    PreserveSharedChildren,
}

/// Native ICopier-compatible subset. All ECS data is owned by Bevy `World`;
/// component copying delegates to Bevy's `EntityCloner`.
///
/// Only edges explicitly represented in `BlueLinks` are graph-traversed.
/// Additional entity-valued component fields need their own clone/remap rules.
/// Default Bevy component cloning can skip components lacking `Clone` / Reflect.
/// Existing destination entities are modified in place; failed recursive copies
/// cannot guarantee a transactional rollback of those existing entities.
#[derive(Default)]
pub struct BlueCopier {
    override_callback: Option<CopyOverrideCallback>,
    post_copy_callback: Option<PostCopyCallback>,
    mapped: HashMap<Entity, Entity>,
    visiting: HashSet<Entity>,
    allocated: Vec<Entity>,
    custom_assignments: HashMap<Clsid, CustomAssignmentCallback>,
}

impl BlueCopier {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_copy_override_callback(&mut self, callback: Option<CopyOverrideCallback>) {
        self.override_callback = callback;
    }

    pub fn set_post_copy_callback(&mut self, callback: Option<PostCopyCallback>) {
        self.post_copy_callback = callback;
    }

    /// Register a class-specific extension that copies data not represented by
    /// reflected/default-clonable ECS components (ICopierCustomAssignment).
    pub fn register_custom_assignment(
        &mut self,
        class: Clsid,
        assignment: CustomAssignmentCallback,
    ) {
        let _ = self.custom_assignments.insert(class, assignment);
    }

    /// Like `IBlueClasses::CopyTo`: repeated child references are copied
    /// independently. Cyclic graphs require `clone_to` instead.
    pub fn copy_to(
        &mut self,
        world: &mut World,
        source: Entity,
        destination: Option<Entity>,
    ) -> Result<Entity, BlueCopyError> {
        self.execute(world, source, destination, GraphCopyMode::DuplicateSharedChildren)
    }

    /// Like `IBlueClasses::CloneTo`: repeated child references and cycles
    /// point at the same newly cloned entities.
    pub fn clone_to(
        &mut self,
        world: &mut World,
        source: Entity,
        destination: Option<Entity>,
    ) -> Result<Entity, BlueCopyError> {
        self.execute(world, source, destination, GraphCopyMode::PreserveSharedChildren)
    }

    fn execute(
        &mut self,
        world: &mut World,
        source: Entity,
        destination: Option<Entity>,
        mode: GraphCopyMode,
    ) -> Result<Entity, BlueCopyError> {
        self.mapped.clear();
        self.visiting.clear();
        self.allocated.clear();
        let result = self.clone_node(world, source, destination, mode);
        if result.is_err() {
            // New entities are rolled back. Modifications to an existing
            // destination cannot be rolled back generically by EntityCloner.
            for entity in self.allocated.drain(..).rev() {
                let _ = world.despawn(entity);
            }
        }
        self.mapped.clear();
        self.visiting.clear();
        self.allocated.clear();
        result
    }

    fn clone_node(
        &mut self,
        world: &mut World,
        source: Entity,
        destination: Option<Entity>,
        mode: GraphCopyMode,
    ) -> Result<Entity, BlueCopyError> {
        if mode == GraphCopyMode::PreserveSharedChildren {
            if let Some(&existing) = self.mapped.get(&source) {
                return Ok(existing);
            }
        }
        if world.get_entity(source).is_err() {
            return Err(BlueCopyError::MissingSource);
        }
        if let Some(destination) = destination {
            if source == destination {
                return Err(BlueCopyError::SourceEqualsDestination);
            }
            if world.get_entity(destination).is_err() {
                return Err(BlueCopyError::MissingDestination);
            }
            if let (Some(source_obj), Some(target_obj)) = (
                world.get::<BlueObject>(source),
                world.get::<BlueObject>(destination),
            ) {
                if source_obj.class_id != target_obj.class_id {
                    return Err(BlueCopyError::ClassMismatch);
                }
            }
        }
        if !self.visiting.insert(source) {
            return Err(BlueCopyError::CycleRequiresCloneTo);
        }
        if let Some(override_callback) = self.override_callback {
            match override_callback(world, source, destination) {
                CopyOverrideResult::Success(entity) => {
                    if world.get_entity(entity).is_err() {
                        return Err(BlueCopyError::InvalidOverrideTarget);
                    }
                    if mode == GraphCopyMode::PreserveSharedChildren {
                        let _ = self.mapped.insert(source, entity);
                    }
                    if let Some(post_copy) = self.post_copy_callback {
                        post_copy(world, source, entity);
                    }
                    self.visiting.remove(&source);
                    return Ok(entity);
                }
                CopyOverrideResult::Failure => return Err(BlueCopyError::OverrideFailed),
                CopyOverrideResult::Fallback => {}
            }
        }

        if world.get_entity(source).is_err() {
            return Err(BlueCopyError::MissingSource);
        }
        let source_links = world.get::<BlueLinks>(source).cloned();
        let target = destination.unwrap_or_else(|| {
            let entity = world.spawn_empty().id();
            self.allocated.push(entity);
            entity
        });
        if mode == GraphCopyMode::PreserveSharedChildren {
            let _ = self.mapped.insert(source, target);
        }

        // This is intentionally the Bevy native cloning mechanism rather
        // than manual reinterpretation of Carbon's byte offsets or vtables.
        // `ChildOf` / `Children` are not the Carbon multi-parent graph.
        EntityCloner::build_opt_out(world)
            .deny::<BlueLinks>()
            .deny::<ChildOf>()
            .deny::<Children>()
            .clone_entity(source, target);

        if let Some(links) = source_links {
            let mut cloned_links = Vec::with_capacity(links.0.len());
            for child in links.0 {
                cloned_links.push(self.clone_node(world, child, None, mode)?);
            }
            world.entity_mut(target).insert(BlueLinks(cloned_links));
        } else {
            world.entity_mut(target).remove::<BlueLinks>();
        }
        if let Some(class) = world.get::<BlueObject>(source).map(|object| object.class_id.clone()) {
            if let Some(assignment) = self.custom_assignments.get(&class).copied() {
                if !assignment(world, source, target) {
                    return Err(BlueCopyError::OverrideFailed);
                }
            }
        }
        if let Some(post_copy) = self.post_copy_callback {
            post_copy(world, source, target);
        }
        self.visiting.remove(&source);
        Ok(target)
    }
}

/// IBlueClasses::ProcessPendingDeletes as a Bevy-owned deferred-despawn queue.
/// The original C++ refcount/FinalDelete rules are not implemented here.
#[derive(Resource, Debug)]
pub struct BluePendingDeletes {
    enabled: bool,
    per_frame: usize,
    pending: Vec<Entity>,
}

impl Default for BluePendingDeletes {
    fn default() -> Self {
        Self {
            enabled: true,
            per_frame: 64,
            pending: Vec::new(),
        }
    }
}

impl BluePendingDeletes {
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_per_frame(&mut self, maximum: usize) {
        self.per_frame = maximum;
    }

    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

/// If pending deletion is disabled, an entity is despawned immediately.
/// Duplicate requests enqueue a single despawn, preserving submission order.
pub fn request_blue_delete(world: &mut World, entity: Entity) -> bool {
    if world.get_entity(entity).is_err() {
        return false;
    }
    let enabled = world
        .get_resource::<BluePendingDeletes>()
        .is_some_and(|queue| queue.enabled);
    if !enabled {
        return world.despawn(entity);
    }
    let mut queue = world.resource_mut::<BluePendingDeletes>();
    if !queue.pending.contains(&entity) {
        queue.pending.push(entity);
    }
    true
}

/// Despawn up to `maximum` pending entities and return the actual number.
/// A zero maximum performs no work.
pub fn process_pending_blue_deletes(world: &mut World, maximum: usize) -> usize {
    let expired = {
        let Some(mut queue) = world.get_resource_mut::<BluePendingDeletes>() else {
            return 0;
        };
        let count = maximum.min(queue.pending.len());
        queue.pending.drain(..count).collect::<Vec<_>>()
    };
    expired.into_iter().filter(|&e| world.despawn(e)).count()
}

pub fn process_all_pending_blue_deletes(world: &mut World) -> usize {
    process_pending_blue_deletes(world, usize::MAX)
}

fn process_pending_deletes_system(world: &mut World) {
    let maximum = world
        .get_resource::<BluePendingDeletes>()
        .map_or(0, |queue| queue.per_frame);
    process_pending_blue_deletes(world, maximum);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{ResMut, Update};

    #[test]
    fn clsid_constructor_interns_name_and_module_and_hashes_name_only() {
        let mut table = BlueStringTable::default();
        let first = Clsid::new(&mut table, "alpha", "Player");
        let second = Clsid::new(&mut table, "beta", "Player");

        assert_eq!(first.name(), "Player");
        assert_eq!(first.module(), "alpha");
        assert_eq!(first, second); // Intentional legacy behavior.
        assert_eq!(first.cmp(&second), Ordering::Equal);
        assert!(first.is_valid());
        assert_eq!(table.len(), 3);
    }

    #[test]
    fn clsid_init_from_string_splits_on_first_dot_including_empty_segments() {
        let mut table = BlueStringTable::default();
        let mut id = Clsid::default();

        assert!(id.init_from_string(&mut table, ".Class.SubClass"));
        assert_eq!(id.module(), "");
        assert_eq!(id.name(), "Class.SubClass");

        assert!(id.init_from_string(&mut table, "Module."));
        assert_eq!(id.module(), "Module");
        assert_eq!(id.name(), "");
    }

    #[test]
    fn clsid_invalid_parse_resets_all_fields_and_hash() {
        let mut table = BlueStringTable::default();
        let mut id = Clsid::new(&mut table, "pkg", "Class");

        assert!(!id.init_from_string(&mut table, "invalid"));
        assert!(!id.is_valid());
        assert_eq!(id.hash(), 0);
        assert_eq!(id.name(), "");
        assert_eq!(id.module(), "");
    }

    #[test]
    fn iid_init_from_string_does_not_parse_dots() {
        let mut table = BlueStringTable::default();
        let mut iid = Iid::new(&mut table, "Old");
        assert!(iid.init_from_string(&mut table, "foo.Bar"));
        assert_eq!(iid.name(), "foo.Bar");
        assert!(iid.is_valid());
    }

    #[test]
    fn clsid_and_iid_preserve_legacy_fnv_collisions() {
        let mut table = BlueStringTable::default();
        let a = Iid::new(&mut table, "s77998");
        let b = Iid::new(&mut table, "s390206");

        assert_eq!(a.hash(), 0xf612_e748);
        assert_eq!(a, b);
        assert_ne!(a.name(), b.name());

        let ca = Clsid::new(&mut table, "m", "s77998");
        let cb = Clsid::new(&mut table, "m", "s390206");
        assert_eq!(ca, cb);
    }

    #[test]
    fn cpp_signed_byte_hash_and_c_string_termination_are_preserved() {
        let mut table = BlueStringTable::default();
        let non_ascii = Iid::new(&mut table, "é");
        assert_eq!(non_ascii.hash(), 0x59bd_db47);
        assert_eq!(cpp_fnv1_with_nul("é"), 0x59bd_db47);

        let id = Clsid::new(&mut table, "module\0ignored", "Name\0ignored");
        assert_eq!(id.module(), "module");
        assert_eq!(id.name(), "Name");
        assert_eq!(id.hash(), cpp_fnv1_with_nul("Name"));
    }

    #[test]
    fn optionals_preserve_payload_when_unassigned() {
        let mut arg = Optional::<u32>::default();
        assert_eq!(*arg.value(), 0);
        assert!(!arg.is_assigned());

        *arg.value_mut() = 9;
        assert!(!arg.is_assigned());
        arg.set_assigned(true);
        arg.set_assigned(false);
        assert_eq!(*arg.value(), 9);

        arg.assign(12);
        assert!(arg.is_assigned());
        assert_eq!(*arg.value(), 12);
    }

    #[test]
    fn optional_with_default_preserves_omitted_distinction() {
        let absent = OptionalWithDefaultValue::omitted(42);
        let supplied = OptionalWithDefaultValue::assigned(42);
        assert_eq!(absent.as_optional().value(), supplied.as_optional().value());
        assert!(!absent.as_optional().is_assigned());
        assert!(supplied.as_optional().is_assigned());
    }

    #[test]
    fn signature_enforces_16_arguments_and_optionals() {
        let mut table = BlueStringTable::default();
        let args = ["i32"; 16];
        let sig =
            BlueExposureFunctionSignature::try_new(&mut table, "bool", &args, 3).unwrap();
        assert_eq!(sig.argument_count(), 16);
        assert_eq!(sig.optional_count(), 3);
        assert_eq!(sig.argument_types().len(), 16);

        assert_eq!(
            BlueExposureFunctionSignature::try_new(&mut table, "bool", &["x"; 17], 0),
            Err(SignatureError::TooManyArguments),
        );
        assert_eq!(
            BlueExposureFunctionSignature::try_new(&mut table, "bool", &["x"], 2),
            Err(SignatureError::TooManyOptionalArguments),
        );
    }

    #[test]
    fn var_type_preserves_sparse_numeric_discriminants() {
        for (tag, expected) in [
            (0, VarType::Invalid),
            (1, VarType::Long),
            (17, VarType::IntArray),
            (20, VarType::RotReference),
            (25, VarType::BinaryBlock),
            (31, VarType::Byte),
            (37, VarType::Uint64),
            (u32::MAX, VarType::ForceLong),
        ] {
            assert_eq!(VarType::try_from(tag), Ok(expected));
            assert_eq!(expected as u32, tag);
        }
        for reserved in [18, 19, 26, 27, 28, 29, 30, 38] {
            assert!(VarType::try_from(reserved).is_err());
        }
        assert_eq!(VarType::MAX, 37);
    }

    #[test]
    fn edit_flags_preserve_masks_and_combinations() {
        let flags = EditFlags::READ | EditFlags::WRITE | EditFlags::PERSIST;
        assert_eq!(flags.bits(), 0x13);
        assert!(flags.contains(EditFlags::READWRITE));
        assert_eq!((flags & EditFlags::SERMASK).bits(), 0x10);
        assert_eq!(EditFlags::PERSISTONLY.bits(), 0x18);
        assert_eq!(EditFlags::EDIT_FORCELONG.bits(), u32::MAX);
        assert_eq!(EditFlags::from_bits_retain(0x8000_0000).bits(), 0x8000_0000);
    }

    #[test]
    fn array_accessors_reproduce_legacy_sizes_and_non_array_fallback() {
        let mut field = VarEntry {
            name: Arc::from("values"),
            var_type: VarType::FloatArray,
            size_bytes: 20,
            iid: None,
            description: Arc::from(""),
            edit_flags: EditFlags::NONE,
            choices: Vec::new(),
        };
        assert_eq!(field.float_array_size(), 5);
        assert_eq!(field.double_array_size(), 1);
        field.var_type = VarType::DoubleArray;
        assert_eq!(field.double_array_size(), 2);
        field.var_type = VarType::IntArray;
        assert_eq!(field.int_array_size(), 5);
        field.var_type = VarType::Long;
        assert_eq!(field.int_array_size(), 1);
    }

    #[test]
    fn registry_stores_bevy_component_metadata_and_names() {
        let mut table = BlueStringTable::default();
        let id = Clsid::new(&mut table, "ui", "Window");
        let iid = Iid::new(&mut table, "IWindow");
        let class = ClassInfo::new(&mut table, id.clone(), iid, "A window");
        let mut registry = BlueTypesRegistry::default();

        assert!(registry.register_class(class).is_none());
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.class(&id).unwrap().description.as_ref(), "A window");
    }

    #[test]
    fn factory_can_create_a_native_bevy_entity() {
        fn create(world: &mut World, iid: &Iid) -> Option<Entity> {
            (iid.name() == "IRoot").then(|| world.spawn(BlueWeakObject).id())
        }

        let mut table = BlueStringTable::default();
        let class_id = Clsid::new(&mut table, "example", "FactoryType");
        let iid = Iid::new(&mut table, "IRoot");
        let mut registry = BlueTypesRegistry::default();
        registry.register_factory(
            class_id.clone(),
            ClassRegistration {
                create_instance: create,
                flags: ClassRegistrationFlags::DISABLE_PYTHON_CONSTRUCTION,
            },
        );

        let mut world = World::new();
        let factory = registry.factory(&class_id).unwrap();
        assert!(factory.flags.contains(ClassRegistrationFlags::DISABLE_PYTHON_CONSTRUCTION));
        let entity = (factory.create_instance)(&mut world, &iid).unwrap();
        assert!(world.get::<BlueWeakObject>(entity).is_some());
    }

    #[test]
    fn weak_entity_reference_is_invalid_after_despawn() {
        let mut world = World::new();
        let entity = world.spawn(BlueWeakObject).id();
        let weak = BlueWeakRef::new(entity);

        assert_eq!(weak.resolve(&world), Some(entity));
        let _ = world.despawn(entity);
        assert_eq!(weak.resolve(&world), None);
    }

    #[test]
    fn becast_copies_bits_instead_of_performing_numeric_conversion() {
        assert_eq!(be_cast_f32(1.0).to_bits(), be_cast_u32(1_f32.to_bits()).to_bits());
        assert_ne!(be_cast_f32(1.0), 1.0_f64);
        assert_eq!(be_cast_i64(-1).to_bits(), u64::MAX);
        assert_eq!(be_cast_u64(42).to_bits(), 42);
    }

    fn register_type(mut registry: ResMut<BlueTypesRegistry>, mut pool: ResMut<BlueStringTable>) {
        let id = Clsid::new(&mut pool, "example", "Sprite");
        let iid = Iid::new(&mut pool, "ISprite");
        let _ = registry.register_class(ClassInfo::new(&mut pool, id, iid, "sprite"));
    }

    #[test]
    fn plugin_exposes_both_resources_to_ecs_systems() {
        let mut app = App::new();
        app.add_plugins(BlueTypesPlugin);
        app.add_systems(Update, register_type);
        app.update();

        assert_eq!(app.world().resource::<BlueTypesRegistry>().len(), 1);
        assert!(!app.world().resource::<BlueStringTable>().is_empty());
    }

    #[test]
    fn registry_bulk_register_lookup_unregister_and_inherited_fields() {
        let mut pool = BlueStringTable::default();
        let parent = Clsid::new(&mut pool, "objects", "Parent");
        let child = Clsid::new(&mut pool, "objects", "Child");
        let iid = Iid::new(&mut pool, "IRoot");
        let mut parent_info = ClassInfo::new(&mut pool, parent.clone(), iid.clone(), "parent");
        parent_info.members.push(VarEntry {
            name: pool.intern("size"),
            var_type: VarType::Long,
            size_bytes: 4,
            iid: None,
            description: pool.intern("size"),
            edit_flags: EditFlags::READ,
            choices: Vec::new(),
        });
        let mut child_info = ClassInfo::new(&mut pool, child.clone(), iid, "child");
        child_info.parent = Some(parent.clone());
        fn factory(world: &mut World, _iid: &Iid) -> Option<Entity> {
            Some(world.spawn_empty().id())
        }
        let registrations = [
            (parent_info, ClassRegistration { create_instance: factory, flags: ClassRegistrationFlags::default() }),
            (child_info, ClassRegistration { create_instance: factory, flags: ClassRegistrationFlags::default() }),
        ];
        let mut registry = BlueTypesRegistry::default();
        registry.register_classes(&registrations);
        assert_eq!(registry.len(), 2);
        assert_eq!(registry.find_clsid("objects", "Child"), Some(&child));
        assert_eq!(registry.find_clsid_by_name("Parent"), Some(&parent));
        assert_eq!(registry.find_variable(&child, "size").unwrap().size_bytes, 4);
        registry.unregister_classes(&[child.clone()]);
        assert!(registry.class(&child).is_none());
        assert!(registry.factory(&child).is_none());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn registered_factory_spawns_bevy_object_and_queries_its_native_interface() {
        #[derive(Component)]
        struct TestInterface;
        fn factory(world: &mut World, iid: &Iid) -> Option<Entity> {
            (iid.name() == "ITestInterface")
                .then(|| world.spawn(TestInterface).id())
        }
        let mut app = App::new();
        app.add_plugins(BlueTypesPlugin);
        let component_id = app.world_mut().register_component::<TestInterface>();
        let mut pool = BlueStringTable::default();
        let clsid = Clsid::new(&mut pool, "testing", "TestClass");
        let iid = Iid::new(&mut pool, "ITestInterface");
        let mut info = ClassInfo::new(&mut pool, clsid.clone(), iid.clone(), "test class");
        info.interfaces.push(InterfaceEntry { iid: iid.clone(), component_id });
        app.world_mut().resource_mut::<BlueTypesRegistry>().register(
            info,
            ClassRegistration { create_instance: factory, flags: ClassRegistrationFlags::default() },
        );
        let created = create_registered_entity_by_name(app.world_mut(), "TestClass", &iid).unwrap();
        assert!(app.world().get::<BlueObject>(created).is_some());
        assert_eq!(query_this_interface(app.world(), created, &iid), Some(created));
        app.world_mut().entity_mut(created).remove::<TestInterface>();
        assert_eq!(query_this_interface(app.world(), created, &iid), None);
        let unsupported = Iid::new(&mut pool, "Unsupported");
        assert_eq!(create_registered_entity(app.world_mut(), &clsid, &unsupported), Err(BlueCreateError::UnsupportedInterface));
    }

    #[test]
    fn copy_to_duplicates_repeated_child_but_clone_to_preserves_it() {
        #[derive(Component, Clone, PartialEq, Eq, Debug)]
        struct Payload(u32);
        let mut world = World::new();
        let child = world.spawn(Payload(7)).id();
        let root = world.spawn((Payload(3), BlueLinks(vec![child, child]))).id();
        let mut copier = BlueCopier::default();
        let copied = copier.copy_to(&mut world, root, None).unwrap();
        let edges = &world.get::<BlueLinks>(copied).unwrap().0;
        assert_ne!(edges[0], edges[1]);
        assert_eq!(world.get::<Payload>(edges[0]).unwrap(), &Payload(7));
        assert_eq!(world.get::<Payload>(edges[1]).unwrap(), &Payload(7));
        let cloned = copier.clone_to(&mut world, root, None).unwrap();
        let edges = &world.get::<BlueLinks>(cloned).unwrap().0;
        assert_eq!(edges[0], edges[1]);
        assert_ne!(edges[0], child);
    }

    #[test]
    fn clone_to_supports_cycles_but_copy_to_rolls_back_new_entities() {
        let mut world = World::new();
        let source = world.spawn(BlueLinks::default()).id();
        world.entity_mut(source).insert(BlueLinks(vec![source]));
        let mut copier = BlueCopier::new();
        let cloned = copier.clone_to(&mut world, source, None).unwrap();
        assert_eq!(world.get::<BlueLinks>(cloned).unwrap().0, vec![cloned]);
        let before = world.entities().count_spawned();
        assert_eq!(copier.copy_to(&mut world, source, None), Err(BlueCopyError::CycleRequiresCloneTo));
        assert_eq!(world.entities().count_spawned(), before);
    }

    #[test]
    fn copy_callback_failure_and_postcopy_are_observable() {
        #[derive(Resource, Default)]
        struct CopyCount(usize);
        fn fallback(_: &mut World, _: Entity, _: Option<Entity>) -> CopyOverrideResult {
            CopyOverrideResult::Fallback
        }
        fn post(world: &mut World, _: Entity, _: Entity) {
            world.resource_mut::<CopyCount>().0 += 1;
        }
        fn failure(_: &mut World, _: Entity, _: Option<Entity>) -> CopyOverrideResult {
            CopyOverrideResult::Failure
        }
        let mut world = World::new();
        world.insert_resource(CopyCount::default());
        let original = world.spawn(BlueLinks::default()).id();
        let mut copier = BlueCopier::new();
        copier.set_copy_override_callback(Some(fallback));
        copier.set_post_copy_callback(Some(post));
        let _ = copier.copy_to(&mut world, original, None).unwrap();
        assert_eq!(world.resource::<CopyCount>().0, 1);
        copier.set_copy_override_callback(Some(failure));
        assert_eq!(copier.copy_to(&mut world, original, None), Err(BlueCopyError::OverrideFailed));
    }

    #[test]
    fn custom_assignment_copies_extension_data() {
        #[derive(Component, Clone, Debug, PartialEq, Eq)]
        struct Extra(u32);
        fn assign(world: &mut World, source: Entity, destination: Entity) -> bool {
            let Some(value) = world.get::<Extra>(source).map(|v| v.0) else {
                return false;
            };
            world.entity_mut(destination).insert(Extra(value + 1));
            true
        }
        let mut strings = BlueStringTable::default();
        let id = Clsid::new(&mut strings, "m", "Custom");
        let mut world = World::new();
        let src = world.spawn((BlueObject::new(id.clone()), Extra(41))).id();
        let mut copier = BlueCopier::default();
        copier.register_custom_assignment(id, assign);
        let dst = copier.copy_to(&mut world, src, None).unwrap();
        assert_eq!(world.get::<Extra>(dst), Some(&Extra(42)));
    }

    #[test]
    fn delayed_delete_queue_deduplicates_and_respects_batch_limit() {
        let mut world = World::new();
        world.insert_resource(BluePendingDeletes::default());
        let a = world.spawn_empty().id();
        let b = world.spawn_empty().id();
        assert!(request_blue_delete(&mut world, a));
        assert!(request_blue_delete(&mut world, a));
        assert!(request_blue_delete(&mut world, b));
        assert_eq!(world.resource::<BluePendingDeletes>().pending_count(), 2);
        assert_eq!(process_pending_blue_deletes(&mut world, 1), 1);
        assert!(world.get_entity(a).is_err());
        assert!(world.get_entity(b).is_ok());
        assert_eq!(process_all_pending_blue_deletes(&mut world), 1);
        assert!(world.get_entity(b).is_err());
        let c = world.spawn_empty().id();
        world.resource_mut::<BluePendingDeletes>().set_enabled(false);
        assert!(request_blue_delete(&mut world, c));
        assert!(world.get_entity(c).is_err());
    }

    #[test]
    fn last_schedule_processes_pending_delete_batch() {
        let mut app = App::new();
        app.add_plugins(BlueTypesPlugin);
        app.world_mut().resource_mut::<BluePendingDeletes>().set_per_frame(1);
        let a = app.world_mut().spawn_empty().id();
        let b = app.world_mut().spawn_empty().id();
        assert!(request_blue_delete(app.world_mut(), a));
        assert!(request_blue_delete(app.world_mut(), b));
        app.update();
        assert!(app.world().get_entity(a).is_err());
        assert!(app.world().get_entity(b).is_ok());
        app.update();
        assert!(app.world().get_entity(b).is_err());
    }

}
