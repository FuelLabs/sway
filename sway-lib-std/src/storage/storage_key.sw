library;

use ::option::Option;
use ::storage::storage_api::*;

impl StorageKey {
    /// Reads a value of type `T` starting at the location specified by `self`. If the value
    /// crosses the boundary of a storage slot, reading continues at the following slot.
    ///
    /// # Returns
    ///
    /// * [T] - Returns the value previously stored if a the storage slots read were
    /// valid and contain `value`. Reverts otherwise.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1`
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let r: StorageKey = StorageKey {
    ///         slot: 0x0000000000000000000000000000000000000000000000000000000000000000,
    ///         offset: 2,
    ///     };
    ///
    ///     // Reads the third word from storage slot with key 0x000...0
    ///     let x: u64 = r.read_unchecked();
    /// }
    /// ```
    #[storage(read)]
    pub fn read_unchecked<T>(self) -> T {
        read::<T>(self.slot, self.offset).unwrap()
    }

    /// Reads a value of type `T` starting at the location specified by `self`. If the value
    /// crosses the boundary of a storage slot, reading continues at the following slot.
    ///
    /// # Returns
    ///
    /// * [Option<T>] - Returns `Some(value)` if a the storage slots read were valid and contain `value`.
    /// Otherwise, return `None`.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1`
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let r: StorageKey = StorageKey {
    ///         slot: 0x0000000000000000000000000000000000000000000000000000000000000000,
    ///         offset: 2,
    ///     };
    ///
    ///     // Reads the third word from storage slot with key 0x000...0
    ///     let x: Option<u64> = r.read();
    /// }
    /// ```
    #[storage(read)]
    pub fn read<T>(self) -> Option<T> {
        read(self.slot, self.offset)
    }

    /// Writes a value of type `T` starting at the location specified by `self`. If the value
    /// crosses the boundary of a storage slot, writing continues at the following slot.
    ///
    /// # Arguments
    ///
    /// * `value`: [T] - The value of type `T` to write.
    ///
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1`
    /// * Writes: `1`
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let r: StorageKey = StorageKey {
    ///         slot: 0x0000000000000000000000000000000000000000000000000000000000000000,
    ///         offset: 2,
    ///     };
    ///
    ///     // Writes 42 at the third word of storage slot with key 0x000...0
    ///     r.write(42);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn write<T>(self, value: T) {
        write(self.slot, self.offset, value);
    }

    /// Create a new `StorageKey`.
    ///
    /// # Arguments
    ///
    /// * `slot`: [b256] - The assigned location in storage for the new `StorageKey`.
    /// * `offset`: [u64] - The assigned offset for the new `StorageKey`.
    ///
    /// # Returns
    ///
    /// * [StorageKey] - The newly created `StorageKey`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{constants::ZERO_B256, hash::sha256};
    ///
    /// fn foo() {
    ///     let my_key = StorageKey::new(ZERO_B256, 0, sha256(ZERO_B256));
    ///     assert(my_key.slot == ZERO_B256);
    /// }
    /// ```
    pub fn new(slot: b256, offset: u64) -> Self {
        Self {
            slot,
            offset,
        }
    }

    pub fn offset_by(self, value: u64) -> Self {
        Self {
            slot: self.slot,
            offset: self.offset + value,
        }
    }

    // Add padding to type so it can correctly use the storage api
    pub fn offset_by_type<T>(self, count: u64) -> Self {
        let size_in_bytes = __size_of::<T>();
        let size_in_slots = (size_in_bytes + 32 - 1) / 32;
        Self {
            slot: self.slot,
            offset: self.offset + size_in_slots * count,
        }
    }
}

#[test]
fn test_storage_key_new() {
    use ::constants::ZERO_B256;
    use ::assert::assert;

    let key = StorageKey::new(ZERO_B256, 0);
    assert(key.slot == ZERO_B256);
    assert(key.offset == 0);
}
