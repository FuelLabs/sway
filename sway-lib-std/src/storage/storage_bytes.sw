library;

use ::bytes::Bytes;
use ::option::Option::{self, *};
use ::storage::storable_slice::*;
use ::storage::{storage_api::*, storage_key::StorageKey};
use ::codec::*;
use ::debug::*;

/// A persistent storage type to store a collection of tightly packed bytes.
pub struct StorageBytes {}

impl StorableSlice<Bytes> for StorageKey<StorageBytes> {
    /// Takes a `Bytes` type and stores the underlying collection of tightly packed bytes.
    ///
    /// # Arguments
    ///
    /// * `bytes`: [Bytes] - The bytes which will be stored.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Writes: `2`
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_bytes::StorageBytes, bytes::Bytes};
    ///
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
    ///     storage.stored_bytes.write_slice(bytes);
    /// }
    /// ```
    #[storage(read, write)]
    fn write_slice(self, bytes: Bytes) {
        write_slice(self.field_id(), bytes.as_raw_slice());
    }

    /// Constructs a `Bytes` type from a collection of tightly packed bytes in storage.
    ///
    /// # Returns
    ///
    /// * [Option<Bytes>] - The valid `Bytes` stored, otherwise `None`.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `2`
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_bytes::StorageBytes, bytes::Bytes};
    ///
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
    ///     assert(storage.stored_bytes.read_slice().is_none());
    ///     storage.stored_bytes.write_slice(bytes);
    ///     let retrieved_bytes = storage.stored_bytes.read_slice().unwrap();
    ///     assert(bytes == retrieved_bytes);
    /// }
    /// ```
    #[storage(read)]
    fn read_slice(self) -> Option<Bytes> {
        match read_slice(self.field_id()) {
            Some(slice) => {
                Some(Bytes::from(slice))
            },
            None => None,
        }
    }

    /// Clears a collection of tightly packed bytes in storage.
    ///
    /// # Returns
    ///
    /// * [bool] - Indicates whether all of the storage slots cleared were previously set.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1`
    /// * Clears: `2`
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_bytes::StorageBytes, bytes::Bytes};
    ///
    /// storage {
    ///     stored_bytes: StorageBytes = StorageBytes {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///     storage.stored_bytes.write_slice(bytes);
    ///
    ///     assert(storage.stored_bytes.read_slice().is_some());
    ///     let cleared = storage.stored_bytes.clear();
    ///     assert(cleared);
    ///     let retrieved_bytes = storage.stored_bytes.read_slice();
    ///     assert(retrieved_bytes.is_none());
    /// }
    /// ```
    #[storage(read, write)]
    fn clear(self) -> bool {
        clear_slice(self.field_id())
    }

    /// Returns the length of tightly packed bytes in storage.
    ///
    /// # Returns
    ///
    /// * [u64] - The length of the bytes in storage.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1`
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_bytes::StorageBytes, bytes::Bytes};
    ///
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
    ///     storage.stored_bytes.write_slice(bytes);
    ///     assert(storage.stored_bytes.len() == 3);
    /// }
    /// ```
    #[storage(read)]
    fn len(self) -> u64 {
        read::<u64>(self.field_id(), 0).unwrap_or(0)
    }
}
