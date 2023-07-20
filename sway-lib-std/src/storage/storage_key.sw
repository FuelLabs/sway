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
    /// valid and contain `value`. Panics otherwise.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1`
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let r: StorageKey<u64> = StorageKey {
    ///         slot: 0x0000000000000000000000000000000000000000000000000000000000000000,
    ///         offset: 2,
    ///         field_id: 0x0000000000000000000000000000000000000000000000000000000000000000,
    ///     };
    ///
    ///     // Reads the third word from storage slot with key 0x000...0
    ///     let x: u64 = r.read();
    /// }
    /// ```
    #[storage(read)]
    pub fn read(self) -> T {
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
    ///     let r: StorageKey<u64> = StorageKey {
    ///         slot: 0x0000000000000000000000000000000000000000000000000000000000000000,
    ///         offset: 2,
    ///         field_id: 0x0000000000000000000000000000000000000000000000000000000000000000,
    ///     };
    ///
    ///     // Reads the third word from storage slot with key 0x000...0
    ///     let x: Option<u64> = r.try_read();
    /// }
    /// ```
    #[storage(read)]
    pub fn try_read(self) -> Option<T> {
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
    ///     let r: StorageKey<u64> = StorageKey {
    ///         slot: 0x0000000000000000000000000000000000000000000000000000000000000000,
    ///         offset: 2,
    ///         field_id: 0x0000000000000000000000000000000000000000000000000000000000000000,
    ///     };
    ///
    ///     // Writes 42 at the third word of storage slot with key 0x000...0
    ///     let x = r.write(42); 
    /// }
    /// ```
    #[storage(read, write)]
    pub fn write(self, value: T) {
        write(self.slot, self.offset, value);
    }
}
