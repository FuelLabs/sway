library;

use ::option::Option;
use ::storage::storage_api::*;
use ::codec::*;
use ::debug::*;

/// Describes a location in storage, made of slots of 32 bytes in size,
/// at which a value of type `T` can be read or written.
///
/// # Additional Information
///
/// The location in storage is specified by the `b256` key of a particular storage slot and an
/// `offset`, given in words, from the start of that `slot`. The parameter `T` is the type of
/// the data to be read from or written to at the `offset`.
///
/// The `T` must be a non-zero-sized type in order to be written to or read from storage.
///
/// Depending on the size of `T` and the `offset`, reading or writing a value of type `T`
/// may require accessing multiple consecutive storage slots.
///
/// A value can share the slot with another values within the same storage slot.
///
/// If `T` is a zero-sized type, no storage access will occur. Moreover, if `T` is a zero-sized type,
/// `StorageKey` will assume that it is a _storage type_, i.e., a type that provides a custom access
/// to the storage. `StorageVec`, `StorageString`, and `StorageBytes` given in the Sway
/// standard library are all examples of _storage types_.
///
/// The `field_id` is a unique identifier for the storage slot being referred to.
/// It is used for zero-sized _storage types_ to differentiate between multiple zero-sized storage entries
/// that might live at the same storage location but represent different storage constructs.
#[cfg(experimental_dynamic_storage = false)]
pub struct StorageKey<T> {
    /// The key of the 32-byte-long storage slot.
    slot: b256,
    /// The offset, *in words*, starting from the beginning of the `slot`.
    offset: u64,
    /// The unique identifier for the storage slot being referred to, used by zero-sized _storage types_.
    field_id: b256,
}

/// Describes a location in storage, within a single dynamic slot of a variable length,
/// at which a value of type `T` can be read or written.
///
/// # Additional Information
///
/// The location in storage is specified by the `b256` key of a particular storage slot and an
/// `offset`, given in bytes, from the start of that `slot`. The parameter `T` is the type of
/// the data to be read from or written to at the `offset`.
///
/// The `T` must be a non-zero-sized type in order to be written to or read from storage.
///
/// The value is stored in a single dynamic slot, so reading or writing a value of type `T`
/// will require accessing only one storage slot.
///
/// A value can share the slot with another values within the same storage slot.
///
/// If `T` is a zero-sized type, no storage access will occur. Moreover, if `T` is a zero-sized type,
/// `StorageKey` will assume that it is a _storage type_, i.e., a type that provides a custom access
/// to the storage. `StorageVec`, `StorageString`, and `StorageBytes` given in the Sway
/// standard library are all examples of _storage types_.
///
/// The `field_id` is a unique identifier for the storage slot being referred to.
/// It is used for zero-sized _storage types_ to differentiate between multiple zero-sized storage entries
/// that might live at the same storage location but represent different storage constructs.
#[cfg(experimental_dynamic_storage = true)]
pub struct StorageKey<T> {
    /// The key of the dynamic storage slot.
    slot: b256,
    /// The offset, *in bytes*, starting from the beginning of the `slot`.
    offset: u64,
    /// The unique identifier for the storage slot being referred to, used by zero-sized _storage types_.
    field_id: b256,
}

impl<T> StorageKey<T> {
    /// Creates a new `StorageKey`.
    ///
    /// # Arguments
    ///
    /// * `slot`: [b256] - The key of the location in storage where the value will be stored.
    /// * `offset`: [u64] - The offset, *in words*, from the start of the `slot` at which the value will be stored.
    /// * `field_id`: [b256] - A unique identifier used by zero-sized _storage types_.
    ///
    /// # Returns
    ///
    /// * [StorageKey<T>] - The newly created `StorageKey`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::hash::sha256;
    ///
    /// fn foo() {
    ///     let key = StorageKey::<u64>::new(b256::zero(), 0, sha256(b256::zero()));
    ///     assert_eq(key.slot(), b256::zero());
    ///     assert_eq(key.offset(), 0);
    ///     assert_eq(key.field_id(), sha256(b256::zero()));
    /// }
    /// ```
    #[cfg(experimental_dynamic_storage = false)]
    pub fn new(slot: b256, offset: u64, field_id: b256) -> Self {
        Self {
            slot,
            offset,
            field_id,
        }
    }

    /// Creates a new `StorageKey`.
    ///
    /// # Arguments
    ///
    /// * `slot`: [b256] - The key of the location in storage where the value will be stored.
    /// * `offset`: [u64] - The offset, *in bytes*, from the start of the `slot`, at which the value will be stored.
    /// * `field_id`: [b256] - A unique identifier used by zero-sized _storage types_.
    ///
    /// # Returns
    ///
    /// * [StorageKey<T>] - The newly created `StorageKey`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::hash::sha256;
    ///
    /// fn foo() {
    ///     let key = StorageKey::<u64>::new(b256::zero(), 0, sha256(b256::zero()));
    ///     assert_eq(key.slot(), b256::zero());
    ///     assert_eq(key.offset(), 0);
    ///     assert_eq(key.field_id(), sha256(b256::zero()));
    /// }
    /// ```
    #[cfg(experimental_dynamic_storage = true)]
    pub fn new(slot: b256, offset: u64, field_id: b256) -> Self {
        Self {
            slot,
            offset,
            field_id,
        }
    }

    /// Returns the storage slot key.
    ///
    /// # Returns
    ///
    /// * [b256] - The key of the storage slot that this `StorageKey` points to.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::hash::sha256;
    ///
    /// fn foo() {
    ///     let key = StorageKey::<u64>::new(b256::zero(), 0, sha256(b256::zero()));
    ///     assert_eq(key.slot(), b256::zero());
    /// }
    /// ```
    pub fn slot(self) -> b256 {
        self.slot
    }

    /// Returns the offset, *in words*, from the start of the `slot`, at which the value will be stored.
    ///
    /// # Returns
    ///
    /// * [u64] - The offset from `slot`, *in words*, that this `StorageKey` points to.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::hash::sha256;
    ///
    /// fn foo() {
    ///     let key = StorageKey::<u64>::new(b256::zero(), 0, sha256(b256::zero()));
    ///     assert_eq(key.offset(), 0);
    /// }
    /// ```
    #[cfg(experimental_dynamic_storage = false)]
    pub fn offset(self) -> u64 {
        self.offset
    }

    /// Returns the offset, *in bytes*, from the start of the `slot`, at which the value will be stored.
    ///
    /// # Returns
    ///
    /// * [u64] - The offset in `slot`, *in bytes*, that this `StorageKey` points to.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::hash::sha256;
    ///
    /// fn foo() {
    ///     let key = StorageKey::<u64>::new(b256::zero(), 0, sha256(b256::zero()));
    ///     assert_eq(key.offset(), 0);
    /// }
    /// ```
    #[cfg(experimental_dynamic_storage = true)]
    pub fn offset(self) -> u64 {
        self.offset
    }

    /// Returns the storage slot field id.
    ///
    /// # Returns
    ///
    /// * [b256] - The field id for this `StorageKey`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::hash::sha256;
    ///
    /// fn foo() {
    ///     let key = StorageKey::<u64>::new(b256::zero(), 0, sha256(b256::zero()));
    ///     assert_eq(key.field_id(), sha256(b256::zero()));
    /// }
    /// ```
    pub fn field_id(self) -> b256 {
        self.field_id
    }

    /// Creates and returns a new zero value for the `StorageKey<T>` type.
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
    ///     assert_eq!(zero_storage_key.slot(), b256::zero());
    ///     assert_eq!(zero_storage_key.offset(), 0);
    ///     assert_eq!(zero_storage_key.field_id(), b256::zero());
    /// }
    /// ```
    pub fn zero() -> Self {
        Self::new(b256::zero(), 0, b256::zero())
    }

    /// Returns whether a `StorageKey<T>` is equal to its zero value.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `StorageKey<T>` is equal to its zero value, otherwise false.
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
        self.slot == b256::zero() && self.offset == 0 && self.field_id == b256::zero()
    }
}

impl<T> StorageKey<T> {
    /// Reads a value of type `T` starting at the location specified by `self`. If the value
    /// crosses the boundary of a 32-byte-long storage slot, reading continues at the following slot.
    ///
    /// # Returns
    ///
    /// * [T] - Returns the value previously stored, if the storage reads were
    /// valid and contain a value.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1`
    ///
    /// # Reverts
    ///
    /// * When `T` is a zero-sized type.
    /// * When any of the storage slots to read from do not contain a value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let key: StorageKey<u64> = StorageKey::new(b256::zero(), 2, b256::zero());
    ///     // Reads the third word from the storage slot with key 0x000...0.
    ///     let _: u64 = key.read();
    /// }
    /// ```
    #[cfg(experimental_dynamic_storage = false)]
    #[storage(read)]
    pub fn read(self) -> T {
        read_quads::<T>(self.slot, self.offset).unwrap()
    }

    /// Reads a value of type `T` starting at the location specified by `self`.
    ///
    /// # Returns
    ///
    /// * [T] - Returns the value previously stored, if the storage reads were
    /// valid and contain a value.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1`
    ///
    /// # Reverts
    ///
    /// * When `T` is a zero-sized type.
    /// * When the slot to read from does not contain a value.
    /// * When the `offset` is out of bounds of the currently used portion of the slot, if the slot is not empty.
    /// * When the storage slot is not large enough to contain a value of size of `T` at the given offset.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let key: StorageKey<u64> = StorageKey::new(b256::zero(), 2, b256::zero());
    ///     // Reads a word at the third byte from the storage slot with key 0x000...0.
    ///     let _: u64 = key.read();
    /// }
    /// ```
    #[cfg(experimental_dynamic_storage = true)]
    #[storage(read)]
    pub fn read(self) -> T {
        read_slot::<T>(self.slot, self.offset).unwrap()
    }

    /// Reads a value of type `T` starting at the location specified by `self`. If the value
    /// crosses the boundary of a 32-byte-long storage slot, reading continues at the following slot.
    ///
    /// # Returns
    ///
    /// * [Option<T>] - Returns `Some(value)`, if the storage slots reads were valid and contain `value`.
    /// Otherwise, `None`.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1`
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let key: StorageKey<u64> = StorageKey::new(b256::zero(), 2, b256::zero());
    ///     // Reads the third word from storage slot with key 0x000...0.
    ///     let _: Option<u64> = key.try_read();
    /// }
    /// ```
    #[cfg(experimental_dynamic_storage = false)]
    #[storage(read)]
    pub fn try_read(self) -> Option<T> {
        read_quads::<T>(self.slot, self.offset)
    }

    /// Reads a value of type `T` starting at the location specified by `self`.
    ///
    /// # Returns
    ///
    /// * [Option<T>] - Returns `Some(value)`, if the storage slot read was valid and contain `value`.
    /// Otherwise, `None`.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `1`
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let key: StorageKey<u64> = StorageKey::new(b256::zero(), 2, b256::zero());
    ///     // Reads a word at the third byte from the storage slot with key 0x000...0.
    ///     let _: Option<u64> = key.try_read();
    /// }
    /// ```
    #[cfg(experimental_dynamic_storage = true)]
    #[storage(read)]
    pub fn try_read(self) -> Option<T> {
        read_slot::<T>(self.slot, self.offset)
    }

    /// Writes a value of type `T` starting at the location specified by `self`. If the value
    /// crosses the boundary of a 32-byte-long storage slot, writing continues at the following slot.
    ///
    /// # Arguments
    ///
    /// * `value`: [T] - The value of type `T` to write.
    ///
    ///
    /// # Number of Storage Accesses
    ///
    /// * Reads: `0` if the `value` occupies full slots, `1` otherwise (to read the existing data that will be partially overwritten)
    /// * Writes: `1`
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let key: StorageKey<u64> = StorageKey::new(b256::zero(), 2, b256::zero());
    ///     // Writes 42 at the third word of storage slot with key 0x000...0.
    ///     key.write(42);
    /// }
    /// ```
    #[cfg(experimental_dynamic_storage = false)]
    #[storage(read, write)]
    pub fn write(self, value: T) {
        write_quads::<T>(self.slot, self.offset, value);
    }

    /// Writes a value of type `T` starting at the location specified by `self`.
    ///
    /// # Arguments
    ///
    /// * `value`: [T] - The value of type `T` to write.
    ///
    ///
    /// # Number of Storage Accesses
    ///
    /// * Preloads: `1`
    /// * Writes: `1`
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let key: StorageKey<u64> = StorageKey::new(b256::zero(), 2, b256::zero());
    ///     // Writes 42 at the third byte of storage slot with key 0x000...0.
    ///     key.write(42);
    /// }
    /// ```
    #[cfg(experimental_dynamic_storage = true)]
    #[storage(write)]
    pub fn write(self, value: T) {
        update_slot::<T>(self.slot, self.offset, value);
    }

    /// Clears the value at `self`. Returns `true` if the value existed in the storage before clearing, otherwise `false`.
    ///
    /// # Additional Information
    ///
    /// The whole slot or multiple slots will be cleared, even if the `StorageKey` points to an offset within the slot.
    /// This means that if there are multiple values stored in a same slot, they will all be cleared.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Clears: `1`
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let key: StorageKey<u64> = StorageKey::new(b256::zero(), 2, b256::zero());
    ///     key.write(42);
    ///     let cleared = key.clear();
    ///     assert(cleared); // The value 42 existed before clearing, so `clear` returns `true`.
    ///     let cleared = key.clear();
    ///     assert(!cleared); // The value was already cleared, so `clear` returns `false`.
    /// }
    /// ```
    #[cfg(experimental_dynamic_storage = false)]
    #[storage(write)]
    pub fn clear(self) -> bool {
        const IS_STORAGE_TYPE: bool = __size_of::<T>() == 0;

        if IS_STORAGE_TYPE {
            // If the generic is a zero-sized type, we assume it to be a storage type.
            // Additional assumption, a far fetched one, is that the storage types will
            // have their data structured in the storage in a way that clearing the
            // storage slot at `self.field_id` will have the semantics of clearing the
            // data of the storage type.
            //
            // This is true for the `StorageVec`, `StorageString`, and `StorageBytes`
            // where the length of the stored data is stored at the storage slot with key `self.field_id`.
            // So, this, e.g., clears the length value for `StorageVec`, `StorageString`, and `StorageBytes`
            // and has no impact on `StorageMap`.
            //
            // Enforcing this semantic on the `StorageKey` level is far from ideal
            // and is error prone, but the storage access based on `StorageKey`s doesn't
            // allow for a better solution.
            //
            // This and other `StorageKey` related issues will be addressed in
            // the Configurable and Composable Storage RFC:
            //      https://github.com/FuelLabs/sway-rfcs/pull/40

            // To make the `clear_quads` actually clear the slot, we need
            // to call it with a non-zero sized type, hence `u64` here,
            // as a dummy non-zero sized type.

            // Note that we are clearing the `self.field_id` slot, and not the `self.slot`,
            // which is where the value of type `T` is stored when `T` is a zero-sized type.
            // This is because of the assumptions described above on how zero-sized
            // storage types are stored in storage.
            clear_quads::<u64>(self.field_id, 0)
        } else {
            // For non-zero sized types, we directly clear the storage slot at
            // `self.slot` where the value is stored.
            clear_quads::<T>(self.slot, self.offset)
        }
    }

    /// Clears the value at `self`.
    ///
    /// # Additional Information
    ///
    /// The whole dynamic slot will be cleared, even if the `StorageKey` points to an offset within the slot.
    /// This means that if there are multiple values stored in the same dynamic slot, they will all be cleared.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Clears: `1`
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let key: StorageKey<u64> = StorageKey::new(b256::zero(), 2, b256::zero());
    ///     key.write(42);
    ///     key.clear();
    /// }
    /// ```
    #[cfg(experimental_dynamic_storage = true)]
    #[storage(write)]
    pub fn clear(self) {
        const IS_STORAGE_TYPE: bool = __size_of::<T>() == 0;

        if IS_STORAGE_TYPE {
            // For dynamic storage, we still assume that zero-sized types are storage types
            // and have their data structured in the storage in a way that clearing the
            // storage slot at `self.field_id` will have the semantics of clearing the
            // data of the storage type.
            //
            // This is again true for the `StorageVec`, `StorageString`, and `StorageBytes`,
            // because they store their entire content at the storage slot with key `self.field_id`.
            clear_slots(self.field_id, 1);
        } else {
            clear_slots(self.slot, 1);
        }
    }

    /// Clears the value at `self` and returns whether the cleared slot was previously set.
    ///
    /// # Additional Information
    ///
    /// The whole dynamic slot will be cleared, even if the `StorageKey` points to an offset within the slot.
    /// This means that if there are multiple values stored in the same dynamic slot, they will all be cleared.
    ///
    /// # Number of Storage Accesses
    ///
    /// * Preload: `1` (to determine if the slot was previously set)
    /// * Clears: `1`
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let key: StorageKey<u64> = StorageKey::new(b256::zero(), 2, b256::zero());
    ///     key.write(42);
    ///     let existed = key.clear_existed();
    ///     assert(existed);
    /// }
    /// ```
    #[cfg(experimental_dynamic_storage = true)]
    #[storage(write)]
    pub fn clear_existed(self) -> bool {
        const IS_STORAGE_TYPE: bool = __size_of::<T>() == 0;

        let slot = if IS_STORAGE_TYPE {
            // For dynamic storage, we still assume that zero-sized types are storage types
            // and have their data structured in the storage in a way that clearing the
            // storage slot at `self.field_id` will have the semantics of clearing the
            // data of the storage type.
            //
            // This is again true for the `StorageVec`, `StorageString`, and `StorageBytes`,
            // because they store their entire content at the storage slot with key `self.field_id`.
            self.field_id
        } else {
            self.slot
        };

        let existed = __state_preload(slot) != 0;
        clear_slots(slot, 1);
        existed
    }
}
