library;

use ::bytes::Bytes;
use ::option::Option::{self, *};
use ::storage::{storable_slice::*, storage_key::StorageKey};
use ::storage::storage_api::read_quads;
use ::string::String;
use ::codec::*;
use ::debug::*;

/// A persistent storage type to store a UTF-8 encoded string as a collection of tightly packed bytes.
pub struct StorageString {}

// Note: `StorageString` is a zero-sized storage type that can be nested
//       within other storage types. For example, a `StorageMap<K, StorageString>`.
//       That's why we are **always using the `self.field_id`** as a storage slot
//       for all of the methods of `StorageString`, and **never the `self.slot`**.

#[cfg(experimental_dynamic_storage = false)]
impl StorableSlice<String> for StorageKey<StorageString> {
    /// Takes a `String` type and saves the underlying data in storage.
    ///
    /// # Arguments
    ///
    /// * `string`: [String] - The string which will be stored.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Writes: `2` (one for the `String` length, and one for the content)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_string::StorageString, string::String};
    ///
    /// storage {
    ///     stored_string: StorageString = StorageString {}
    /// }
    ///
    /// fn foo() {
    ///     let string = String::from_ascii_str("Fuel is blazingly fast");
    ///
    ///     storage.stored_string.write_slice(string);
    /// }
    /// ```
    #[storage(read, write)]
    fn write_slice(self, string: String) {
        write_slice_quads(self.field_id(), string.as_raw_slice());
    }

    /// Constructs a `String` type from a collection of tightly packed bytes in storage.
    ///
    /// # Returns
    ///
    /// * [Option<String>] - The valid `String` stored, otherwise `None`.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `2` (one for the `String` length, and one for the content)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_string::StorageString, string::String};
    ///
    /// storage {
    ///     stored_string: StorageString = StorageString {}
    /// }
    ///
    /// fn foo() {
    ///     let string = String::from_ascii_str("Fuel is blazingly fast");
    ///
    ///     assert(storage.stored_string.read_slice().is_none());
    ///     storage.stored_string.write_slice(string);
    ///     let retrieved_string = storage.stored_string.read_slice().unwrap();
    ///     assert_eq(string, retrieved_string);
    /// }
    /// ```
    #[storage(read)]
    fn read_slice(self) -> Option<String> {
        match read_slice_quads(self.field_id()) {
            Some(slice) => {
                Some(String::from(slice))
            },
            None => None,
        }
    }

    /// Clears a stored `String` in storage.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if _all_ of the cleared storage slots were previously set. Otherwise, `false`.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1` (to determine the `String` length)
    /// * Clears: `2` (one for the `String` length, and one for the content)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_string::StorageString, string::String};
    ///
    /// storage {
    ///     stored_string: StorageString = StorageString {}
    /// }
    ///
    /// fn foo() {
    ///     let string = String::from_ascii_str("Fuel is blazingly fast");
    ///
    ///     storage.stored_string.write_slice(string);
    ///
    ///     assert(storage.stored_string.read_slice().is_some());
    ///     let cleared = storage.stored_string.clear();
    ///     assert(cleared);
    ///     let retrieved_string = storage.stored_string.read_slice();
    ///     assert(retrieved_string.is_none());
    /// }
    /// ```
    #[storage(read, write)]
    fn clear(self) -> bool {
        clear_slice_quads(self.field_id())
    }

    /// Returns the length of a `String` in storage, in bytes.
    ///
    /// # Returns
    ///
    /// * [u64] - The length of the `String` in storage, in bytes, or `0` if there is no valid `String` in storage.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1` (to read the `String` length)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_string::StorageString, string::String};
    ///
    /// storage {
    ///     stored_string: StorageString = StorageString {}
    /// }
    ///
    /// fn foo() {
    ///     let string = String::from_ascii_str("Fuel is blazingly fast");
    ///
    ///     assert_eq!(storage.stored_string.len(), 0);
    ///     storage.stored_string.write_slice(string);
    ///     assert_eq!(storage.stored_string.len(), 22);
    /// }
    /// ```
    #[storage(read)]
    fn len(self) -> u64 {
        read_quads::<u64>(self.field_id(), 0).unwrap_or(0)
    }
}

#[cfg(experimental_dynamic_storage = true)]
impl StorableSlice<String> for StorageKey<StorageString> {
    /// Takes a `String` type and saves the underlying data in storage.
    ///
    /// # Arguments
    ///
    /// * `string`: [String] - The string which will be stored.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Writes: `1` (for storing the `String` content)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_string::StorageString, string::String};
    ///
    /// storage {
    ///     stored_string: StorageString = StorageString {}
    /// }
    ///
    /// fn foo() {
    ///     let string = String::from_ascii_str("Fuel is blazingly fast");
    ///
    ///     storage.stored_string.write_slice(string);
    /// }
    /// ```
    #[storage(write)]
    fn write_slice(self, string: String) {
        write_slice_slot(self.field_id(), string.as_raw_slice());
    }

    /// Constructs a `String` type from a collection of tightly packed bytes in storage.
    ///
    /// # Returns
    ///
    /// * [Option<String>] - The valid `String` stored, otherwise `None`.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Preloads: `1` (for the `String` length)
    /// * Reads: `1` (for the `String` content)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_string::StorageString, string::String};
    ///
    /// storage {
    ///     stored_string: StorageString = StorageString {}
    /// }
    ///
    /// fn foo() {
    ///     let string = String::from_ascii_str("Fuel is blazingly fast");
    ///
    ///     assert(storage.stored_string.read_slice().is_none());
    ///     storage.stored_string.write_slice(string);
    ///     let retrieved_string = storage.stored_string.read_slice().unwrap();
    ///     assert_eq(string, retrieved_string);
    /// }
    /// ```
    #[storage(read)]
    fn read_slice(self) -> Option<String> {
        match read_slice_slot(self.field_id()) {
            Some(slice) => {
                Some(String::from(slice))
            },
            None => None,
        }
    }

    /// Clears a stored `String` in storage.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Clears: `1` (for the `String` content)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_string::StorageString, string::String};
    ///
    /// storage {
    ///     stored_string: StorageString = StorageString {}
    /// }
    ///
    /// fn foo() {
    ///     let string = String::from_ascii_str("Fuel is blazingly fast");
    ///
    ///     storage.stored_string.write_slice(string);
    ///
    ///     assert(storage.stored_string.read_slice().is_some());
    ///     storage.stored_string.clear();
    ///     let retrieved_string = storage.stored_string.read_slice();
    ///     assert(retrieved_string.is_none());
    /// }
    /// ```
    #[storage(read, write)]
    fn clear(self) {
        clear_slice_slot(self.field_id())
    }

    /// Clears a stored `String` in storage and returns whether it existed.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if the cleared storage slot was previously set. Otherwise, `false`.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Preload: `1` (to determine if the slot was previously set)
    /// * Clears: `1` (for the `String` content)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_string::StorageString, string::String};
    ///
    /// storage {
    ///     stored_string: StorageString = StorageString {}
    /// }
    ///
    /// fn foo() {
    ///     let string = String::from_ascii_str("Fuel is blazingly fast");
    ///
    ///     storage.stored_string.write_slice(string);
    ///
    ///     assert(storage.stored_string.read_slice().is_some());
    ///     let cleared = storage.stored_string.clear();
    ///     assert(cleared);
    ///     let retrieved_string = storage.stored_string.read_slice();
    ///     assert(retrieved_string.is_none());
    /// }
    /// ```
    #[storage(read, write)]
    fn clear_existed(self) -> bool {
        clear_slice_slot_existed(self.field_id())
    }

    /// Returns the length of a `String` in storage, in bytes.
    ///
    /// # Returns
    ///
    /// * [u64] - The length of the `String` in storage, in bytes, or `0` if there is no valid `String` in storage.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Preloads: `1` (to read the `String` length)
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{storage::storage_string::StorageString, string::String};
    ///
    /// storage {
    ///     stored_string: StorageString = StorageString {}
    /// }
    ///
    /// fn foo() {
    ///     let string = String::from_ascii_str("Fuel is blazingly fast");
    ///
    ///     assert_eq!(storage.stored_string.len(), 0);
    ///     storage.stored_string.write_slice(string);
    ///     assert_eq!(storage.stored_string.len(), 22);
    /// }
    /// ```
    #[storage(read)]
    fn len(self) -> u64 {
        __state_preload(self.field_id())
    }
}
