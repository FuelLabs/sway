library;

use ::option::Option;
use ::storage::storage_api::*;
use ::codec::*;
use ::debug::*;

/// Describes a location in storage.
///
/// # Additional Information
///
/// The location in storage is specified by the `b256` key of a particular storage slot and an
/// offset, in words, from the start of the storage slot at `key`. The parameter `T` is the type of
/// the data to be read from or written to at `offset`.
/// `field_id` is a unique identifier for the storage field being referred to, it is different even
/// for multiple zero sized fields that might live at the same location but
/// represent different storage constructs.
pub struct StorageKey<T> {
    /// The assigned location in storage.
    slot: b256,
    /// The assigned offset based on the data structure `T`.
    offset: u64,
    /// A unique identifier.
    field_id: b256,
}

impl<T> StorageKey<T> {
    /// Create a new `StorageKey`.
    ///
    /// # Arguments
    ///
    /// * `slot`: [b256] - The assigned location in storage for the new `StorageKey`.
    /// * `offset`: [u64] - The assigned offset based on the data structure `T` for the new `StorageKey`.
    /// * `field_id`: [b256] - A unique identifier for the new `StorageKey`.
    ///
    /// # Returns
    ///
    /// * [StorageKey] - The newly created `StorageKey`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::hash::sha256;
    ///
    /// fn foo() {
    ///     let my_key = StorageKey::<u64>::new(b256::zero(), 0, sha256(b256::zero()));
    ///     assert(my_key.slot() == b256::zero());
    /// }
    /// ```
    pub fn new(slot: b256, offset: u64, field_id: b256) -> Self {
        Self {
            slot,
            offset,
            field_id,
        }
    }

    /// Returns the storage slot address.
    ///
    /// # Returns
    ///
    /// * [b256] - The address in storage that this storage slot points to.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::hash::sha256;
    ///
    /// fn foo() {
    ///     let my_key = StorageKey::<u64>::new(b256::zero(), 0, sha256(b256::zero()));
    ///     assert(my_key.slot() == b256::zero());
    /// }
    /// ```
    pub fn slot(self) -> b256 {
        self.slot
    }

    /// Returns the offset on the storage slot.
    ///
    /// # Returns
    ///
    /// * [u64] - The offset in storage that this storage slot points to.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::hash::sha256;
    ///
    /// fn foo() {
    ///     let my_key = StorageKey::<u64>::new(b256::zero(), 0, sha256(b256::zero()));
    ///     assert(my_key.offset() == 0);
    /// }
    /// ```
    pub fn offset(self) -> u64 {
        self.offset
    }

    /// Returns the storage slot field id.
    ///
    /// # Additional Information
    ///
    /// The field id is a unique identifier for the storage field being referred to, it is different even
    /// for multiple zero sized fields that might live at the same location but
    /// represent different storage constructs.
    ///
    /// # Returns
    ///
    /// * [b256] - The field id for this storage slot.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::hash::sha256;
    ///
    /// fn foo() {
    ///     let my_key = StorageKey::<u64>::new(b256::zero(), 0, sha256(b256::zero()));
    ///     assert(my_key.field_id() == sha256(b256::zero()));
    /// }
    /// ```
    pub fn field_id(self) -> b256 {
        self.field_id
    }
}

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
    /// fn foo() {
    ///     let r: StorageKey<u64> = StorageKey::new(b256::zero(), 2, b256::zero());
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
    ///     let r: StorageKey<u64> = StorageKey::new(b256::zero(), 2, b256::zero());
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
    ///     let r: StorageKey<u64> = StorageKey::new(b256::zero(), 2, b256::zero());
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
    ///     let r: StorageKey<u64> = StorageKey::new(b256::zero(), 2, b256::zero());
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

    /// Returns the zero value for the `StorageKey<T>` type.
    ///
    /// # Returns
    ///
    /// * [StorageKey<T>] -> The zero value for the `StorageKey<T>` type.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_storage_key: StorageKey<u64> = StorageKey::zero();
    ///     assert(zero_storage_key.slot() == b256::zero());
    ///     assert(zero_storage_key.offset() == 0);
    ///     assert(zero_storage_key.field_id() == b256::zero());
    /// }
    /// ```
    pub fn zero() -> Self {
        Self::new(b256::zero(), 0, b256::zero())
    }

    /// Returns whether a `StorageKey<T>` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `StorageKey<T>` is set to zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_storage_key: StorageKey<u64> = StorageKey::zero();
    ///     assert(zero_storage_key.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        self.slot() == b256::zero() && self.field_id() == b256::zero() && self.offset() == 0
    }
}
