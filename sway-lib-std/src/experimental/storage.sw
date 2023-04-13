library;

use ::alloc::{alloc, realloc_bytes};
use ::hash::sha256;
use ::option::Option;
use core::experimental::storage::StorageKey;

/// Store a stack value in storage. Will not work for heap values.
///
/// ### Arguments
///
/// * `slot` - The storage slot at which the variable will be stored.
/// * `value` - The value to be stored.
/// * `offset` - An offset, in words, from the beginning of `slot`, at which `value` should be 
///              stored.
///
/// ### Examples
///
/// ```sway
/// let five = 5_u64;
/// write(ZERO_B256, 2, five);
/// let stored_five = read::<u64>(ZERO_B256, 2).unwrap();
/// assert(five == stored_five);
/// ```
#[storage(read, write)]
pub fn write<T>(slot: b256, offset: u64, value: T) {
    // Get the number of storage slots needed based on the size of `T`
    let number_of_slots = (offset * 8 + __size_of::<T>() + 31) >> 5;

    // Allocate enough memory on the heap for `value` as well as any potential padding required due 
    // to `offset`.
    let padded_value = alloc::<u64>(number_of_slots * 32);

    // Read the values that currently exist in the affected storage slots.
    // NOTE: we can do better here by only reading from the slots that we know could be affected. 
    // These are the two slots where the start and end of `T` fall in considering `offset`. 
    // However, doing so requires that we perform addition on `b256` to compute the corresponding 
    // keys, and that is not possible today.
    let _ = __state_load_quad(slot, padded_value, number_of_slots);

    // Copy the value to be stored to `padded_value + offset`.
    padded_value.add::<u64>(offset).write::<T>(value);

    // Now store back the data at `padded_value` which now contains the old data but partially 
    // overwritten by the new data in the desired locations.
    let _ = __state_store_quad(slot, padded_value, number_of_slots);
}

/// Reads a value of type `T` starting at the location specified by `slot` and `offset`. If the
/// value crosses the boundary of a storage slot, reading continues at the following slot.
///
/// Returns `Option(value)` if the storage slots read were valid and contain `value`. Otherwise, 
/// return `None`.
///
/// ### Arguments
///
/// * `slot` - The storage slot to load the value from.
/// * `offset` - An offset, in words, from the start of `slot`, from which the value should be read.
///
/// ### Examples
///
/// ```sway
/// let five = 5_u64;
/// write(ZERO_B256, 2, five);
/// let stored_five = read::<u64>(ZERO_B256, 2);
/// assert(five == stored_five);
/// ```
#[storage(read)]
pub fn read<T>(slot: b256, offset: u64) -> Option<T> {
    // NOTE: we are leaking this value on the heap.
    // Get the number of storage slots needed based on the size of `T`
    let number_of_slots = (offset * 8 + __size_of::<T>() + 31) >> 5;

    // Allocate a buffer for the result. Its size needs to be a multiple of 32 bytes so we can 
    // make the 'quad' storage instruction read without overflowing.
    let result_ptr = alloc::<u64>(number_of_slots * 32);

    // Read `number_of_slots * 32` bytes starting at storage slot `slot` and return an `Option` 
    // wrapping the value stored at `result_ptr + offset` if all the slots are valid. Otherwise, 
    // return `Option::None`.
    if __state_load_quad(slot, result_ptr, number_of_slots) {
        Option::Some(result_ptr.add::<u64>(offset).read::<T>())
    } else {
        Option::None
    }
}

/// Clear a sequence of consecutive storage slots starting at a some slot. Returns a Boolean
/// indicating whether all of the storage slots cleared were previously set.
///
/// ### Arguments
///
/// * `slot` - The key of the first storage slot that will be cleared
///
/// ### Examples
///
/// ```sway
/// let five = 5_u64;
/// write(ZERO_B256, 0, five);
/// let cleared = clear::<u64>(ZERO_B256);
/// assert(cleared);
/// assert(read::<u64>(ZERO_B256, 0).is_none());
/// ```
#[storage(write)]
pub fn clear<T>(slot: b256) -> bool {
    // Get the number of storage slots needed based on the size of `T` as the ceiling of 
    // `__size_of::<T>() / 32`
    let number_of_slots = (__size_of::<T>() + 31) >> 5;

    // Clear `number_of_slots * 32` bytes starting at storage slot `slot`.
    __state_clear(slot, number_of_slots)
}

impl<T> StorageKey<T> {
    /// Reads a value of type `T` starting at the location specified by `self`. If the value
    /// crosses the boundary of a storage slot, reading continues at the following slot.
    ///
    /// Returns the value previously stored if a the storage slots read were
    /// valid and contain `value`. Panics otherwise.
    ///
    /// ### Arguments
    ///
    /// None
    ///
    /// ### Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let r: StorageKey<u64> = StorageKey {
    ///         slot: 0x0000000000000000000000000000000000000000000000000000000000000000,
    ///         offset: 2,
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
    /// Returns `Some(value)` if a the storage slots read were valid and contain `value`.
    /// Otherwise, return `None`.
    ///
    /// ### Arguments
    ///
    /// None
    ///
    /// ### Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let r: StorageKey<u64> = StorageKey {
    ///         slot: 0x0000000000000000000000000000000000000000000000000000000000000000,
    ///         offset: 2,
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
    /// ### Arguments
    ///
    /// * value: the value of type `T` to write
    ///
    /// ### Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let r: StorageKey<u64> = StorageKey {
    ///         slot: 0x0000000000000000000000000000000000000000000000000000000000000000,
    ///         offset: 2,
    ///     };
    ///     let x = r.write(42); // Writes 42 at the third word of storage slot with key 0x000...0
    /// }
    /// ```
    #[storage(read, write)]
    pub fn write(self, value: T) {
        write(self.slot, self.offset, value);
    }
}

/// A persistent key-value pair mapping struct.
pub struct StorageMap<K, V> {}

impl<K, V> StorageKey<StorageMap<K, V>> {
    /// Inserts a key-value pair into the map.
    ///
    /// ### Arguments
    ///
    /// * `key` - The key to which the value is paired.
    /// * `value` - The value to be stored.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// storage {
    ///     map: StorageMap<u64, bool> = StorageMap {}
    /// }
    ///
    /// fn foo() {
    ///     let key = 5_u64;
    ///     let value = true;
    ///     storage.map.insert(key, value);
    ///     let retrieved_value = storage.map.get(key).read();
    ///     assert(value == retrieved_value);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn insert(self, key: K, value: V) {
        let key = sha256((key, self.slot));
        write::<V>(key, 0, value);
    }

    /// Retrieves the `StorageKey` that describes the raw location in storage of the value
    /// stored at `key`, regardless of whether a value is actually stored at that location or not.
    ///
    /// ### Arguments
    ///
    /// * `key` - The key to which the value is paired.
    ///
    /// ### Examples
    ///
    /// ```sway
    /// storage {
    ///     map: StorageMap<u64, bool> = StorageMap {}
    /// }
    ///
    /// fn foo() {
    ///     let key = 5_u64;
    ///     let value = true;
    ///     storage.map.insert(key, value);
    ///     let retrieved_value = storage.map.get(key).read();
    ///     assert(value == retrieved_value);
    /// }
    /// ```
    #[storage(read)]
    pub fn get(self, key: K) -> StorageKey<V> {
        StorageKey {
            slot: sha256((key, self.slot)),
            offset: 0,
        }
    }

    /// Clears a value previously stored using a key
    ///
    /// Return a Boolean indicating whether there was a value previously stored at `key`.
    ///
    /// ### Arguments
    ///
    /// * `key` - The key to which the value is paired
    ///
    /// ### Examples
    ///
    /// ```sway
    /// storage {
    ///     map: StorageMap<u64, bool> = StorageMap {}
    /// }
    ///
    /// fn foo() {
    ///     let key = 5_u64;
    ///     let value = true;
    ///     storage.map.insert(key, value);
    ///     let removed = storage.map.remove(key);
    ///     assert(removed);
    ///     assert(storage.map.get(key).is_none());
    /// }
    /// ```
    #[storage(write)]
    pub fn remove(self, key: K) -> bool {
        let key = sha256((key, self.slot));
        clear::<V>(key)
    }
}
