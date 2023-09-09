library;

use ::bytes::Bytes;
use ::option::Option;
use ::storage::storable_slice::*;
use ::storage::storage_api::read;
use ::string::String;

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
    /// use std::{storage::storage_string::StorageString, bytes::Bytes, string::String};
    ///
    /// storage {
    ///     stored_string: StorageString = StorageString {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///     let string = String::from(bytes);
    ///
    ///     storage.stored_string.write_slice(string);
    /// }
    /// ```
    #[storage(read, write)]
    fn write_slice(self, string: String) {
        write_slice(self.slot, string.as_raw_slice());
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
    /// use std::{storage::storage_string::StorageString, bytes::Bytes, string::String};
    ///
    /// storage {
    ///     stored_string: StorageString = StorageString {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///     let string = String::from(bytes);
    ///
    ///     assert(storage.stored_string.read_slice(key).is_none());
    ///     storage.stored_string.write_slice(string);
    ///     let retrieved_string = storage.stored_string.read_slice(key).unwrap();
    ///     assert(string == retrieved_string);
    /// }
    /// ```
    #[storage(read)]
    fn read_slice(self) -> Option<String> {
        match read_slice(self.slot) {
            Option::Some(slice) => {
                Option::Some(String::from(slice))
            },
            Option::None => Option::None,
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
    /// use std::{storage::storage_string::StorageString, bytes::Bytes, string::String};
    ///
    /// storage {
    ///     stored_string: StorageString = StorageString {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///     let string = String::from(bytes);
    ///
    ///     storage.stored_string.write_slice(string);
    ///
    ///     assert(storage.stored_string.read_slice(key).is_some());
    ///     let cleared = storage.stored_string.clear();
    ///     assert(cleared);
    ///     let retrieved_string = storage.stored_string.read_slice(key);
    ///     assert(retrieved_string.is_none());
    /// }
    /// ```
    #[storage(read, write)]
    fn clear(self) -> bool {
        clear_slice(self.slot)
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
    /// use std::{storage::storage_string::StorageString, bytes::Bytes, string::String};
    ///
    /// storage {
    ///     stored_string: StorageString = StorageString {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///     let string = String::from(bytes);
    ///
    ///     assert(storage.stored_string.len() == 0)
    ///     storage.stored_string.write_slice(string);
    ///     assert(storage.stored_string.len() == 3);
    /// }
    /// ```
    #[storage(read)]
    fn len(self) -> u64 {
        read::<u64>(self.slot, 0).unwrap_or(0)
    }
}
