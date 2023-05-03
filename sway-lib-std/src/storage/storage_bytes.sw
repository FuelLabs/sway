library;

use ::bytes::Bytes;
use ::option::Option::{self, *};
use ::storage::storable_slice::*;
use ::storage::storage_api::*;

/// A persistent storage type to store a collection of tightly packed bytes.
pub struct StorageBytes {}

impl StorableSlice<Bytes> for StorageKey<StorageBytes> {
    /// Takes a `Bytes` type and stores the underlying collection of tightly packed bytes.
    ///
    /// ### Arguments
    ///
    /// * `bytes` - The bytes which will be stored.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Writes: `2`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// storage {
    ///     stored_bytes: StorageBytes = StorageBytes {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///
    ///     storage.stored_bytes.store(bytes);
    /// }
    /// ```
    #[storage(read, write)]
    fn store(self, bytes: Bytes) {
        let key = sha256((self.slot, self.offset));
        store_slice(key, bytes.as_raw_slice());
    }

    /// Constructs a `Bytes` type from a collection of tightly packed bytes in storage.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads: `2`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// storage {
    ///     stored_bytes: StorageBytes = StorageBytes {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///
    ///     assert(storage.stored_bytes.load(key).is_none());
    ///     storage.stored_bytes.store(bytes);
    ///     let retrieved_bytes = storage.stored_bytes.load(key).unwrap();
    ///     assert(bytes == retrieved_bytes);
    /// }
    /// ```
    #[storage(read)]
    fn load(self) -> Option<Bytes> {
        let key = sha256((self.slot, self.offset));
        match get_slice(key) {
            Some(slice) => {
                Some(Bytes::from_raw_slice(slice))
            },
            None => None,
        }
    }

    /// Clears a collection of tightly packed bytes in storage.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads: `1`
    /// * Clears: `2`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// storage {
    ///     stored_bytes: StorageBytes = StorageBytes {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///     storage.stored_bytes.store(bytes);
    ///
    ///     assert(storage.stored_bytes.load(key).is_some());
    ///     let cleared = storage.stored_bytes.clear();
    ///     assert(cleared);
    ///     let retrieved_bytes = storage.stored_bytes.load(key);
    ///     assert(retrieved_bytes.is_none());
    /// }
    /// ```
    #[storage(read, write)]
    fn clear(self) -> bool {
        let key = sha256((self.slot, self.offset));
        clear_slice(key)
    }

    /// Returns the length of tightly packed bytes in storage.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads: `1`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// storage {
    ///     stored_bytes: StorageBytes = StorageBytes {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///
    ///     assert(storage.stored_bytes.len() == 0)
    ///     storage.stored_bytes.store(bytes);
    ///     assert(storage.stored_bytes.len() == 3);
    /// }
    /// ```
    #[storage(read)]
    fn len(self) -> u64 {
        read::<u64>(sha256((self.slot, self.offset)), 0).unwrap_or(0)
    }
}
