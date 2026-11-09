//! A Bevy-native replacement for Carbon's `BlueStringTable`.
//!
//! Source design: BlueStringTable (copyright (c) 2026 CCP Games).
//! All stored values are immutable and remain resident until the owning
//! [`BlueStringTable`] is dropped. Returned shared values may outlive it.

use bevy::{
    platform::{collections::HashSet, sync::Arc},
    prelude::{App, Plugin, Resource},
};

/// An append-only pool that interns UTF-8 strings by their *entire content*.
///
/// The pool is owned by a Bevy [`World`](bevy::prelude::World) as a resource.
/// Callers receive an `Arc<str>`, so they can keep interned strings without
/// borrowing the ECS resource or copying the underlying string bytes.
#[derive(Debug, Default, Resource)]
pub struct BlueStringTable {
    strings: HashSet<Arc<str>>,
    table_size_bytes: usize,
}

impl BlueStringTable {
    /// Returns a shared, canonical allocation for `text` within this pool.
    ///
    /// This copies UTF-8 bytes only when the string is first encountered.
    /// Subsequent calls clone only the shared-ownership handle.
    pub fn intern(&mut self, text: &str) -> Arc<str> {
        if let Some(existing) = self.strings.get(text) {
            return Arc::clone(existing);
        }

        let new_size = text
            .len()
            .checked_add(1)
            .and_then(|size| self.table_size_bytes.checked_add(size))
            .expect("BlueStringTable byte count overflow");

        let interned: Arc<str> = Arc::from(text);
        self.strings.insert(Arc::clone(&interned));
        self.table_size_bytes = new_size;
        interned
    }

    /// Returns the canonical string and its 32-bit FNV-1 hash, including a
    /// synthetic trailing NUL byte to match the *intended* narrow C++ input.
    ///
    /// Full-string comparison, NOT this hash, determines interning identity.
    /// Check the original `CcpHashFNV1` implementation before relying on
    /// cross-language hash compatibility.
    pub fn intern_hashed(&mut self, text: &str) -> (Arc<str>, u32) {
        let hash = fnv1_with_nul(text);
        (self.intern(text), hash)
    }

    /// Number of distinct interned strings.
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    /// Returns `true` if no strings have been interned.
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }

    /// Sum of unique UTF-8 lengths plus one synthetic NUL byte per entry.
    ///
    /// This matches the narrow-string counter's *logical* units, but does
    /// not include hash-table buckets or `Arc` control-block overhead.
    pub fn table_size_bytes(&self) -> usize {
        self.table_size_bytes
    }
}

fn fnv1_with_nul(text: &str) -> u32 {
    const OFFSET_BASIS: u32 = 0x811c_9dc5;
    const PRIME: u32 = 0x0100_0193;

    text.bytes()
        .chain(std::iter::once(0))
        .fold(OFFSET_BASIS, |hash, byte| {
            hash.wrapping_mul(PRIME) ^ u32::from(byte)
        })
}

/// Registers [`BlueStringTable`] for access by Bevy systems via `ResMut`.
pub struct BlueStringTablePlugin;

impl Plugin for BlueStringTablePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlueStringTable>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::{ResMut, Update};

    #[test]
    fn duplicate_strings_share_one_allocation_and_count_once() {
        let mut table = BlueStringTable::default();
        let first = table.intern("shared");
        let second = table.intern(&format!("{}{}", "sha", "red"));

        assert_eq!(first.as_ref(), "shared");
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(table.len(), 1);
        assert_eq!(table.table_size_bytes(), 7);
    }

    #[test]
    fn different_strings_with_same_fnv_hash_remain_distinct() {
        // A fixed, independently enumerated collision for 32-bit FNV-1
        // over UTF-8 bytes INCLUDING the synthetic trailing NUL byte.
        let mut table = BlueStringTable::default();
        let (first, first_hash) = table.intern_hashed("s77998");
        let (second, second_hash) = table.intern_hashed("s390206");

        assert_eq!(first_hash, 0xf612_e748);
        assert_eq!(first_hash, second_hash);
        assert_ne!(first, second);
        assert!(!Arc::ptr_eq(&first, &second));
        assert_eq!(table.len(), 2);
    }

    #[test]
    fn table_size_counts_utf8_bytes_and_one_terminator_per_string() {
        let mut table = BlueStringTable::default();
        assert!(table.is_empty());

        table.intern("");      // 0 + 1
        table.intern("é");     // 2 + 1
        table.intern("cat");   // 3 + 1
        table.intern("cat");   // no increase

        assert_eq!(table.len(), 3);
        assert_eq!(table.table_size_bytes(), 8);
    }

    #[test]
    fn borrowed_string_survives_pool_destruction() {
        let interned = {
            let mut table = BlueStringTable::default();
            table.intern("retained")
        };
        assert_eq!(interned.as_ref(), "retained");
    }

    #[test]
    fn fnv1_matches_fixed_test_vector() {
        assert_eq!(fnv1_with_nul("a"), 0x7077_2d5a);
    }

    fn intern_from_system(mut table: ResMut<BlueStringTable>) {
        table.intern("bevy");
    }

    #[test]
    fn plugin_exposes_pool_as_a_bevy_resource() {
        let mut app = App::new();
        app.add_plugins(BlueStringTablePlugin);
        app.add_systems(Update, intern_from_system);
        app.update();

        let table = app.world().resource::<BlueStringTable>();
        assert_eq!(table.len(), 1);
        assert_eq!(table.table_size_bytes(), 5);
    }
}
