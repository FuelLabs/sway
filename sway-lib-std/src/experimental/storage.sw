library;

use ::alloc::{alloc, realloc_bytes};
use ::assert::assert;
use ::hash::sha256;
use ::option::Option;
use core::experimental::storage::StorageHandle;

/// Store a stack value in storage. Will not work for heap values.
///
/// ### Arguments
///
/// * `key` - The storage slot at which the variable will be stored.
/// * `value` - The value to be stored.
/// * `offset` - An offset, in words, from the beginning of slot at `key`, at which `value` should
///              be stored.
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
pub fn write<T>(key: b256, offset: u64, value: T) {
    if __size_of::<T>() == 0 {
        return;
    }

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
    let _ = __state_load_quad(key, padded_value, number_of_slots);

    // Copy the value to be stored to `padded_value + offset`.
    padded_value.add::<u64>(offset).write::<T>(value);

    // Now store back the data at `padded_value` which now contains the old data but partially 
    // overwritten by the new data in the desired locations.
    let _ = __state_store_quad(key, padded_value, number_of_slots);
}

/// Reads a value of type `T` starting at the location specified by `key` and `offset`. If the
/// value crosses the boundary of a storage slot, reading continues at the following slot.
///
/// Returns `Option(value)` if a the storage slots read were valid and contain `value`.
/// Otherwise, return `None`.
///
/// ### Arguments
///
/// * `key` - The storage slot to load the value from.
/// * `offset` - An offset, in words, from the start of slot at `key`, from which the value should
///              be read.
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
pub fn read<T>(key: b256, offset: u64) -> Option<T> {
    if __size_of::<T>() == 0 {
        return Option::None;
    }

    // NOTE: we are leaking this value on the heap.
    // Get the number of storage slots needed based on the size of `T`
    let number_of_slots = (offset * 8 + __size_of::<T>() + 31) >> 5;

    // Allocate a buffer for the result. Its size needs to be a multiple of 32 bytes so we can 
    // make the 'quad' storage instruction read without overflowing.
    let result_ptr = alloc::<u64>(number_of_slots * 32);

    // Read `number_of_slots * 32` bytes starting at storage slot `key` and return an `Option` 
    // wrapping the value stored at `result_ptr + offset` if all the slots are valid. Otherwise, 
    // return `Option::None`.
    if __state_load_quad(key, result_ptr, number_of_slots) {
        Option::Some(result_ptr.add::<u64>(offset).read::<T>())
    } else {
        Option::None
    }
}

/// Clear a sequence of consecutive storage slots starting at a some key. Returns a Boolean
/// indicating whether all of the storage slots cleared were previously set.
///
/// ### Arguments
///
/// * `key` - The key of the first storage slot that will be cleared
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
fn clear<T>(key: b256) -> bool {
    // Get the number of storage slots needed based on the size of `T` as the ceiling of 
    // `__size_of::<T>() / 32`
    let number_of_slots = (__size_of::<T>() + 31) >> 5;

    // Clear `number_of_slots * 32` bytes starting at storage slot `key`.
    __state_clear(key, number_of_slots)
}

impl<T> StorageHandle<T> {
    /// Reads a value of type `T` starting at the location specified by `self`. If the value
    /// crosses the boundary of a storage slot, reading continues at the following slot.
    ///
    /// Returns the value previously stored if the storage slots read were valid and contain 
    /// `value`. Reverts otherwise.
    ///
    /// ### Reverts
    ///
    /// Reverts if at least one of the storage slots needed to read a value of type `T` is not set.
    /// 
    /// ### Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let r: StorageHandle<u64> = StorageHandle {
    ///         key: 0x0000000000000000000000000000000000000000000000000000000000000000,
    ///         offset: 2,
    ///     };
    ///
    ///     // Reads the third word from storage slot with key 0x000...0
    ///     let x:u64 = r.read();
    /// }
    /// ```
    #[storage(read)]
    pub fn read(self) -> T {
        read::<T>(self.key, self.offset).unwrap()
    }

    /// Reads a value of type `T` starting at the location specified by `self`. If the value
    /// crosses the boundary of a storage slot, reading continues at the following slot.
    ///
    /// Returns `Option(value)` if a the storage slots read were valid and contain `value`.
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
    ///     let r: StorageHandle<u64> = StorageHandle {
    ///         key: 0x0000000000000000000000000000000000000000000000000000000000000000,
    ///         offset: 2,
    ///     };
    ///
    ///     // Reads the third word from storage slot with key 0x000...0
    ///     let x:Option<u64> = r.try_read();
    /// }
    /// ```
    #[storage(read)]
    pub fn try_read(self) -> Option<T> {
        read(self.key, self.offset)
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
    ///     let r: StorageHandle<u64> = StorageHandle {
    ///         key: 0x0000000000000000000000000000000000000000000000000000000000000000,
    ///         offset: 2,
    ///     };
    ///     let x = r.write(42); // Writes 42 at the third word of storage slot with key 0x000...0
    /// }
    /// ```
    #[storage(read, write)]
    pub fn write(self, value: T) {
        write(self.key, self.offset, value);
    }
}

/// A persistent key-value pair mapping struct.
pub struct StorageMap<K, V> {}

impl<K, V> StorageMap<K, V> {
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
    ///     let retrieved = storage.map.get(key).read();
    ///     assert(value == retrieved);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn insert(self: StorageHandle<Self>, key: K, value: V) {
        let key = sha256((key, self.key));
        write::<V>(key, 0, value);
    }

    /// Inserts a key-value pair into the map using the `[]` operator
    ///
    /// This is temporary until we are able to implement `trait IndexAssign`. The Sway compiler will
    /// de-sugar the index operator `[]` in an assignment expression to a call to `index_assign()`.
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
    ///     storage.map[key] = value; // de-sugars to `storage.map.index_assign(key, value);`
    ///     let retrieved = storage.map.get(key).read();
    ///     assert(value == retrieved);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn index_assign(self: StorageHandle<Self>, key: K, value: V) {
        let key = sha256((key, self.key));
        write::<V>(key, 0, value);
    }

    /// Retrieves the `StorageHandle` that describes the raw location in storage of the value
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
    ///     let retrieved = storage.map.get(key).read();
    ///     assert(value == retrieved);
    /// }
    /// ```
    pub fn get(self: StorageHandle<Self>, key: K) -> StorageHandle<V> {
        StorageHandle {
            key: sha256((key, self.key)),
            offset: 0,
        }
    }

    /// Retrieves the `StorageHandle` that describes the raw location in storage of the value 
    /// stored at `key`, regardless of whether a value is actually stored at that location or not.
    ///
    /// This is temporary until we are able to implement `trait Index`. The Sway compiler will
    /// de-sugar the index operator `[]` in an expression to a call to `index()`.
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
    ///     let retrieved = storage.map[key].read(); // de-sugars to `storage.map.get(key).read()`
    ///     assert(value == retrieved);
    /// }
    pub fn index(self: StorageHandle<Self>, key: K) -> StorageHandle<V> {
        StorageHandle {
            key: sha256((key, self.key)),
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
    pub fn remove(self: StorageHandle<Self>, key: K) -> bool {
        let key = sha256((key, self.key));
        clear::<V>(key)
    }
}

/// A persistant vector struct.
pub struct StorageVec<V> {
}

impl<V> StorageVec<V> {
    /// Appends the value to the end of the vector.
    ///
    /// ### Arguments
    ///
    /// * `value` - The item being added to the end of the vector.
    ///
    /// ### Number of Number of Storage Accesses
    ///
    /// * Reads: `1`
    /// * Writes: `2`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::storage::StorageVec;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     let five = 5_u64;
    ///     storage.vec.push(five);
    ///     assert(five == storage.vec[0].read());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn push(self: StorageHandle<Self>, value: V) {
        let len = read::<u64>(self.key, 0).unwrap_or(0);

        // Storing the value at the current length index (if this is the first item, starts off at 0)
        let key = sha256((len, self.key));
        write::<V>(key, 0, value);

        // Incrementing the length
        write(self.key, 0, len + 1);
    }

    /// Removes the last element of the vector and returns it, `None` if empty.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads: `2`
    /// * Writes: `1`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::storage::StorageVec;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     let five = 5_u64;
    ///     storage.vec.push(five);
    ///     let popped_value = storage.vec.pop().unwrap();
    ///     assert(five == popped_value);
    ///     let none_value = storage.vec.pop();
    ///     assert(none_value.is_none())
    /// }
    /// ```
    #[storage(read, write)]
    pub fn pop(self: StorageHandle<Self>) -> Option<V> {
        let len = read::<u64>(self.key, 0).unwrap_or(0);

        // if the length is 0, there is no item to pop from the vec
        if len == 0 {
            return Option::None;
        }

        // reduces len by 1, effectively removing the last item in the vec
        write(self.key, 0, len - 1);

        let key = sha256((len - 1, self.key));
        read::<V>(key, 0)
    }

    /// Gets the value in the given index, `None` if index is out of bounds.
    ///
    /// ### Arguments
    ///
    /// * `index` - The index of the vec to retrieve the item from.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads: `2`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::storage::StorageVec;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     let five = 5_u64;
    ///     storage.vec.push(five);
    ///     assert(five == storage.vec.get(0).unwrap().read());
    ///     assert(storage.vec.get(1).is_none())
    /// }
    /// ```
    #[storage(read)]
    pub fn get(self: StorageHandle<Self>, index: u64) -> Option<StorageHandle<V>> {
        let len = read::<u64>(self.key, 0).unwrap_or(0);

        // if the index is larger or equal to len, there is no item to return
        if len <= index {
            return Option::None;
        }

        Option::Some(StorageHandle {
            key: sha256((index, self.key)),
            offset: 0,
        })
    }

    /// Retrieves the `StorageHandle` that describes the raw location in storage of the value 
    /// stored at `index`. Reverts if `index` is out of bounds.
    ///
    /// This is temporary until we are able to implement `trait Index`. The Sway compiler will
    /// de-sugar the index operator `[]` in an expression to a call to `index()`.
    ///
    /// ### Arguments
    ///
    /// * `index` - The index of the vec to retrieve the item from.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads: `2`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::storage::StorageVec;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     let five = 5_u64;
    ///     storage.vec.push(five);
    ///     assert(five == storage.vec[0].read());
    ///     assert(storage.vec.get(1).is_none())
    /// }
    /// ```
    #[storage(read)]
    pub fn index(self: StorageHandle<Self>, index: u64) -> StorageHandle<V> {
        let len = read::<u64>(self.key, 0).unwrap_or(0);

        assert(index < len);

        StorageHandle {
            key: sha256((index, self.key)),
            offset: 0,
        }
    }

    /// Removes the element in the given index and moves all the elements in the following indexes
    /// down one index. Also returns the element.
    ///
    /// > **_WARNING:_** Expensive for larger vecs.
    ///
    /// ### Arguments
    ///
    /// * `index` - The index of the vec to remove the item from.
    ///
    /// ### Reverts
    ///
    /// Reverts if index is larger or equal to length of the vec.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads: `2 + self.len() - index`
    /// * Writes: `self.len() - index`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::storage::StorageVec;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     storage.vec.push(15);
    ///     let removed_value = storage.vec.remove(1);
    ///     assert(10 == removed_value);
    ///     assert(storage.vec.len() == 2);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn remove(self: StorageHandle<Self>, index: u64) -> V {
        let len = read::<u64>(self.key, 0).unwrap_or(0);

        // if the index is larger or equal to len, there is no item to remove
        assert(index < len);

        // gets the element before removing it, so it can be returned
        let removed_element = read::<V>(sha256((index, self.key)), 0).unwrap();

        // for every element in the vec with an index greater than the input index,
        // shifts the index for that element down one
        let mut count = index + 1;
        while count < len {
            // gets the storage location for the previous index
            let key = sha256((count - 1, self.key));
            // moves the element of the current index into the previous index
            write::<V>(key, 0, read::<V>(sha256((count, self.key)), 0).unwrap());

            count += 1;
        }

        // decrements len by 1
        write(self.key, 0, len - 1);

        removed_element
    }

    /// Removes the element at the specified index and fills it with the last element.
    /// This does not preserve ordering and returns the element.
    ///
    /// ### Arguments
    ///
    /// * `index` - The index of the vec to remove the item from.
    ///
    /// ### Reverts
    ///
    /// Reverts if index is larger or equal to length of the vec.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads: `3`
    /// * Writes: `2`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::storage::StorageVec;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     storage.vec.push(15);
    ///     let removed_value = storage.vec.swap_remove(0);
    ///     assert(5 == removed_value);
    ///     let swapped_value = storage.vec[0].read();
    ///     assert(15 == swapped_value);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn swap_remove(self: StorageHandle<Self>, index: u64) -> V {
        let len = read::<u64>(self.key, 0).unwrap_or(0);

        // if the index is larger or equal to len, there is no item to remove
        assert(index < len);

        let hash_of_to_be_removed = sha256((index, self.key));
        // gets the element before removing it, so it can be returned
        let element_to_be_removed = read::<V>(hash_of_to_be_removed, 0).unwrap();

        let last_element = read::<V>(sha256((len - 1, self.key)), 0).unwrap();
        write::<V>(hash_of_to_be_removed, 0, last_element);

        // decrements len by 1
        write(self.key, 0, len - 1);

        element_to_be_removed
    }

    /// Sets or mutates the value at the given index.
    ///
    /// ### Arguments
    ///
    /// * `index` - The index of the vec to set the value at
    /// * `value` - The value to be set
    ///
    /// ### Reverts
    ///
    /// Reverts if index is larger than or equal to the length of the vec.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads: `1`
    /// * Writes: `1`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::storage::StorageVec;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     storage.vec.push(15);
    ///
    ///     storage.vec.set(0, 20);
    ///     let set_value = storage.vec[0].read();
    ///     assert(20 == set_value);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn set(self: StorageHandle<Self>, index: u64, value: V) {
        let len = read::<u64>(self.key, 0).unwrap_or(0);

        // if the index is higher than or equal len, there is no element to set
        assert(index < len);

        let key = sha256((index, self.key));
        write::<V>(key, 0, value);
    }

    /// Sets or mutates the value at the given index.
    ///
    /// This is temporary until we are able to implement `trait IndexAssign`. The Sway compiler will
    /// de-sugar the index operator `[]` in an assignment expression to a call to `index_assign()`.
    ///
    /// ### Arguments
    ///
    /// * `index` - The index of the vec to set the value at
    /// * `value` - The value to be set
    ///
    /// ### Reverts
    ///
    /// Reverts if index is larger than or equal to the length of the vec.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads: `1`
    /// * Writes: `1`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::storage::StorageVec;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     storage.vec.push(15);
    ///
    ///     storage.vec[0] = 20;
    ///     let set_value = storage.vec[0].read();
    ///     assert(20 == set_value);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn index_assign(self: StorageHandle<Self>, index: u64, value: V) {
        let len = read::<u64>(self.key, 0).unwrap_or(0);

        // if the index is higher than or equal len, there is no element to set
        assert(index < len);

        let key = sha256((index, self.key));
        write::<V>(key, 0, value);
    }

    /// Inserts the value at the given index, moving the current index's value
    /// as well as the following index's value up by one index.
    ///
    /// > **_WARNING:_** Expensive for larger vecs.
    ///
    /// ### Arguments
    ///
    /// * `index` - The index of the vec to insert the item into.
    /// * `value` - The value to insert into the vec.
    ///
    /// ### Reverts
    ///
    /// Reverts if index is larger than the length of the vec.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads: `if self.len() == index { 1 } else { 1 + self.len() - index }`
    /// * Writes: `if self.len() == index { 2 } else { 2 + self.len() - index }`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::storage::StorageVec;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(15);
    ///
    ///     storage.vec.insert(1, 10);
    ///
    ///     assert(5 == storage.vec[0].read());
    ///     assert(10 == storage.vec[1].read());
    ///     assert(15 == storage.vec[2].read());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn insert(self: StorageHandle<Self>, index: u64, value: V) {
        let len = read::<u64>(self.key, 0).unwrap_or(0);

        // if the index is larger than len, there is no space to insert
        assert(index <= len);

        // if len is 0, index must also be 0 due to above check
        if len == index {
            let key = sha256((index, self.key));
            write::<V>(key, 0, value);

            // increments len by 1
            write(self.key, 0, len + 1);

            return;
        }

        // for every element in the vec with an index larger than the input index,
        // move the element up one index.
        // performed in reverse to prevent data overwriting
        let mut count = len - 1;
        while count >= index {
            let key = sha256((count + 1, self.key));
            // shifts all the values up one index
            write::<V>(key, 0, read::<V>(sha256((count, self.key)), 0).unwrap());

            count -= 1
        }

        // inserts the value into the now unused index
        let key = sha256((index, self.key));
        write::<V>(key, 0, value);

        // increments len by 1
        write(self.key, 0, len + 1);
    }

    /// Returns the length of the vector.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads: `1`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::storage::StorageVec;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     assert(0 == storage.vec.len());
    ///     storage.vec.push(5);
    ///     assert(1 == storage.vec.len());
    ///     storage.vec.push(10);
    ///     assert(2 == storage.vec.len());
    /// }
    /// ```
    #[storage(read)]
    pub fn len(self: StorageHandle<Self>) -> u64 {
        read::<u64>(self.key, 0).unwrap_or(0)
    }

    /// Checks whether the len is zero or not.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads: `1`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::storage::StorageVec;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     assert(true == storage.vec.is_empty());
    ///
    ///     storage.vec.push(5);
    ///
    ///     assert(false == storage.vec.is_empty());
    ///
    ///     storage.vec.clear();
    ///
    ///     assert(true == storage.vec.is_empty());
    /// }
    /// ```
    #[storage(read)]
    pub fn is_empty(self: StorageHandle<Self>) -> bool {
        read::<u64>(self.key, 0).unwrap_or(0) == 0
    }

    /// Sets the len to zero.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Clears: `1`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// use std::storage::StorageVec;
    ///
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {}
    /// }
    ///
    /// fn foo() {
    ///     assert(0 == storage.vec.len());
    ///     storage.vec.push(5);
    ///     assert(1 == storage.vec.len());
    ///     storage.vec.clear();
    ///     assert(0 == storage.vec.len());
    /// }
    /// ```
    #[storage(write)]
    pub fn clear(self: StorageHandle<Self>) {
        let _ = clear::<u64>(self.key);
    }
}
