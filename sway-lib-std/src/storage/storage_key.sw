library;

use ::option::Option;
use ::storage::storage_api::*;

impl<T> StorageKey<T> {
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
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///     let r: StorageKey<u64> = StorageKey::new(ZERO_B256, 2, ZERO_B256);s
    ///     // Reads the third word from storage slot with key 0x000...0
    ///     let x: u64 = r.read();
    /// }
    /// ```
    #[storage(read)]
    pub fn read(self) -> T {
        read::<T>(self.slot(), self.offset()).unwrap()
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
    ///     let r: StorageKey<u64> = StorageKey::new(ZERO_B256, 2, ZERO_B256);
    ///
    ///     // Reads the third word from storage slot with key 0x000...0
    ///     let x: Option<u64> = r.try_read();
    /// }
    /// ```
    #[storage(read)]
    pub fn try_read(self) -> Option<T> {
        read(self.slot(), self.offset())
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
    ///     let r: StorageKey<u64> = StorageKey::new(ZERO_B256, 2, ZERO_B256);
    ///
    ///     // Writes 42 at the third word of storage slot with key 0x000...0
    ///     let x = r.write(42);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn write(self, value: T) {
        write(self.slot(), self.offset(), value);
    }

    /// Clears the value at `self`.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Clears: `1`
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let r: StorageKey<u64> = StorageKey::new(ZERO_B256, 2, ZERO_B256);
    ///     r.write(42);
    ///
    ///     let cleared = r.clear();
    ///     assert(cleared);
    /// }
    /// ```
    #[storage(write)]
    pub fn clear(self) -> bool {
        if __size_of::<T>() == 0 {
            // If the generic doesn't have a size, this is an empty struct and nothing can be stored at the slot.
            // This clears the length value for StorageVec, StorageString, and StorageBytes 
            // or any other Storage type.
            clear::<u64>(self.field_id(), 0)
        } else {
            clear::<T>(self.slot(), self.offset())
        }
    }
}

#[test]
fn test_storage_key_new() {
    use ::constants::ZERO_B256;
    use ::assert::assert;

    let key = StorageKey::<u64>::new(ZERO_B256, 0, ZERO_B256);
    assert(key.slot() == ZERO_B256);
    assert(key.offset() == 0);
    assert(key.field_id() == ZERO_B256);
}