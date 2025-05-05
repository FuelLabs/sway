library;

use ::bytes::Bytes;
use ::option::Option::{self, *};
use ::storage::{storable_slice::*, storage_key::StorageKey};
use ::storage::storage_api::read;
use ::string::String;
use ::codec::*;
use ::debug::*;

pub struct StorageString {}

impl StorableSlice<String> for StorageKey<StorageString> {
    /// Takes a `String` type and saves the underlying data in storage.
    ///
    /// # Arguments
    ///
    /// * `string`: [String] - The string which will be stored.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Writes: `2`
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
        write_slice(self.field_id(), string.as_raw_slice());
    }

    /// Constructs a `String` type from storage.
    ///
    /// # Returns
    ///
    /// * [Option<String>] - The valid `String` stored, otherwise `None`.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `2`
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
    ///     assert(string == retrieved_string);
    /// }
    /// ```
    #[storage(read)]
    fn read_slice(self) -> Option<String> {
        match read_slice(self.field_id()) {
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
        clear_slice(self.field_id())
    }

    /// Returns the length of `String` in storage.
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
    /// use std::{storage::storage_string::StorageString, string::String};
    ///
    /// storage {
    ///     stored_string: StorageString = StorageString {}
    /// }
    ///
    /// fn foo() {
    ///     let string = String::from_ascii_str("Fuel is blazingly fast");
    ///
    ///     assert(storage.stored_string.len() == 0)
    ///     storage.stored_string.write_slice(string);
    ///     assert(storage.stored_string.len() == 3);
    /// }
    /// ```
    #[storage(read)]
    fn len(self) -> u64 {
        read::<u64>(self.field_id(), 0).unwrap_or(0)
    }
}
