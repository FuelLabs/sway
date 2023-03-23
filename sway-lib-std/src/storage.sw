library;

use ::alloc::{alloc, alloc_bytes, realloc_bytes};
use ::assert::assert;
use ::hash::sha256;
use ::option::Option;
use ::bytes::Bytes;

/// Store a stack value in storage. Will not work for heap values.
///
/// ### Arguments
///
/// * `key` - The storage slot at which the variable will be stored.
/// * `value` - The value to be stored.
///
/// ### Examples
///
/// ```sway
/// use std::{storage::{store, get}, constants::ZERO_B256};
///
/// let five = 5_u64;
/// store(ZERO_B256, five);
/// let stored_five = get::<u64>(ZERO_B256).unwrap();
/// assert(five == stored_five);
/// ```
#[storage(write)]
pub fn store<T>(key: b256, value: T) {
    if !__is_reference_type::<T>() {
        // If `T` is a copy type, then `value` fits in a single word.
        let value = asm(v: value) { v: u64 };
        let _ = __state_store_word(key, value);
    } else {
        // If `T` is a reference type, then `value` can be larger than a word, so we need to use
        // `__state_store_quad`.
        // Get the number of storage slots needed based on the size of `T`
        let number_of_slots = (__size_of::<T>() + 31) >> 5;

        // Cast the pointer to `value` to a `raw_ptr`.
        let mut ptr_to_value = asm(ptr: value) { ptr: raw_ptr };

        // Store `number_of_slots * 32` bytes starting at storage slot `key`.
        let _ = __state_store_quad(key, ptr_to_value, number_of_slots);
    };
}

/// Load a value from storage.
///
/// If the value size is larger than 8 bytes it is read to a heap buffer which is leaked for the
/// duration of the program.
///
/// If no value was previously stored at `key`, `Option::None` is returned. Otherwise,
/// `Option::Some(value)` is returned, where `value` is the value stored at `key`.
///
/// ### Arguments
///
/// * `key` - The storage slot to load the value from.
///
/// ### Examples
///
/// ```sway
/// use std::{storage::{store, get}, constants::ZERO_B256};
///
/// let five = 5_u64;
/// store(ZERO_B256, five);
/// let stored_five = get::<u64>(ZERO_B256);
/// assert(five == stored_five);
/// ```
#[storage(read)]
pub fn get<T>(key: b256) -> Option<T> {
    let (previously_set, value) = if !__is_reference_type::<T>() {
        // If `T` is a copy type, then we can use `srw` to read from storage. `srw` writes two 
        // registers: the loaded word as well as flag indicating whether the storage slot was 
        // written before. We store the two registers on the heap and return the result as a tuple 
        // `(bool, T)` which contains the two values we need.
        // NOTE: we should eventually be using `__state_load_word` here but we are currently unable 
        // to make that intrinsic return two things due to some limitations in IR/codegen.
        let temp_pair = (false, 0_u64);    // Using a `u64` as a placeholder for copy-type `T` value.
        asm(key: key, result_ptr: temp_pair, loaded_word, previously_set) {
            srw  loaded_word previously_set key;
            sw   result_ptr previously_set i0;
            sw   result_ptr loaded_word i1;
            result_ptr: (bool, T)
        }
    } else {
        // If `T` is a reference type, then we need to use `__state_load_quad` because the result
        // might be larger than a word.
        // NOTE: we are leaking this value on the heap.
        
        // Get the number of storage slots needed based on the size of `T` as the ceiling of 
        // `__size_of::<T>() / 32`
        let number_of_slots = (__size_of::<T>() + 31) >> 5; 

        // Allocate a buffer for the result. Its size needs to be a multiple of 32 bytes so we can 
        // make the 'quad' storage instruction read without overflowing.
        let result_ptr = alloc::<u64>(number_of_slots * 32);

        // Read `number_of_slots * 32` bytes starting at storage slot `key`.
        // The return `bool` indicates if all the slots have been set before.
        let previously_set = __state_load_quad(key, result_ptr, number_of_slots);

        // Cast the final result to `T` 
        (previously_set, asm(res: result_ptr) { res: T })
    };

    if previously_set {
        Option::Some(value)
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
/// use std::{storage::{clear, get, store}, constants::ZERO_B256};
///
/// let five = 5_u64;
/// store(ZERO_B256, five);
/// let cleared = clear::<u64>(ZERO_B256);
/// assert(cleared);
/// assert(get::<u64>(ZERO_B256).is_none());
/// ```
#[storage(write)]
pub fn clear<T>(key: b256) -> bool {
    // Get the number of storage slots needed based on the size of `T` as the ceiling of 
    // `__size_of::<T>() / 32`
    let number_of_slots = (__size_of::<T>() + 31) >> 5;

    // Clear `number_of_slots * 32` bytes starting at storage slot `key`.
    __state_clear(key, number_of_slots)
}

/// Store a raw_slice from the heap into storage.
///
/// ### Arguments
///
/// * `key` - The storage slot at which the variable will be stored.
/// * `slice` - The raw_slice to be stored.
///
/// ### Examples
///
/// ```sway
/// use std::{alloc::alloc_bytes, storage::{store_slice, get_slice}, constants::ZERO_B256};
///
/// let slice = asm(ptr: (alloc_bytes(1), 1)) { ptr: raw_slice };
/// assert(get_slice(ZERO_B256).is_none());
/// store_slice(ZERO_B256, slice);
/// let stored_slice = get_slice(ZERO_B256).unwrap();
/// assert(slice == stored_slice);
/// ```
#[storage(write)]
pub fn store_slice(key: b256, slice: raw_slice) {
    // Get the number of storage slots needed based on the size of bytes.
    let number_of_bytes = slice.number_of_bytes();
    let number_of_slots = (number_of_bytes + 31) >> 5;
    let mut ptr = slice.ptr();

    // The capacity needs to be a multiple of 32 bytes so we can 
    // make the 'quad' storage instruction store without accessing unallocated heap memory.
    ptr = realloc_bytes(ptr, number_of_bytes, number_of_slots * 32);

    // Store `number_of_slots * 32` bytes starting at storage slot `key`.
    let _ = __state_store_quad(sha256(key), ptr, number_of_slots);

    // Store the length of the bytes
    store(key, number_of_bytes);
}

/// Load a raw_slice from storage.
///
/// If no value was previously stored at `key`, `Option::None` is returned. Otherwise,
/// `Option::Some(value)` is returned, where `value` is the value stored at `key`.
///
/// ### Arguments
///
/// * `key` - The storage slot to load the value from.
///
/// ### Examples
///
/// ```sway
/// use std::{alloc::alloc_bytes, storage::{store_slice, get_slice}, constants::ZERO_B256};
///
/// let slice = asm(ptr: (alloc_bytes(1), 1)) { ptr: raw_slice };
/// assert(get_slice(ZERO_B256).is_none());
/// store_slice(ZERO_B256, slice);
/// let stored_slice = get_slice(ZERO_B256).unwrap();
/// assert(slice == stored_slice);
/// ```
#[storage(read)]
pub fn get_slice(key: b256) -> Option<raw_slice> {
    // Get the length of the slice that is stored.
    match get::<u64>(key).unwrap_or(0) {
        0 => Option::None,
        len => {
            // Get the number of storage slots needed based on the size.
            let number_of_slots = (len + 31) >> 5;
            let ptr = alloc_bytes(number_of_slots * 32);
            // Load the stored slice into the pointer.
            let _ = __state_load_quad(sha256(key), ptr, number_of_slots);
            Option::Some(asm(ptr: (ptr, len)) { ptr: raw_slice })
        }
    }
}

/// Clear a sequence of storage slots starting at a some key. Returns a Boolean
/// indicating whether all of the storage slots cleared were previously set.
///
/// ### Arguments
///
/// * `key` - The key of the first storage slot that will be cleared
///
/// ### Examples
///
/// ```sway
/// use std::{alloc::alloc_bytes, storage::{clear_slice, store_slice, get_slice}, constants::ZERO_B256};
///
/// let slice = asm(ptr: (alloc_bytes(1), 1)) { ptr: raw_slice };
/// store_slice(ZERO_B256, slice);
/// assert(get_slice(ZERO_B256).is_some());
/// let cleared = clear_slice(ZERO_B256);
/// assert(cleared);
/// assert(get_slice(ZERO_B256).is_none());
/// ```
#[storage(read, write)]
pub fn clear_slice(key: b256) -> bool {
    // Get the number of storage slots needed based on the ceiling of `len / 32`
    let len = get::<u64>(key).unwrap_or(0);
    let number_of_slots = (len + 31) >> 5;

    // Clear length and `number_of_slots` bytes starting at storage slot `sha256(key)`
    let _ = __state_clear(key, 1);
    __state_clear(sha256(key), number_of_slots)
}

pub trait StorableSlice<T> {
    #[storage(write)]
    fn store(self, argument: T);
    #[storage(read)]
    fn load(self) -> Option<T>;
    #[storage(read, write)]
    fn clear(self) -> bool;
    #[storage(read)]
    fn len(self) -> u64;
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
    ///     let retrieved_value = storage.map.get(key);
    ///     assert(value == retrieved_value);
    /// }
    /// ```
    #[storage(write)]
    pub fn insert(self, key: K, value: V) {
        let key = sha256((key, __get_storage_key()));
        store::<V>(key, value);
    }

    /// Retrieves a value previously stored using a key.
    ///
    /// If no value was previously stored at `key`, `Option::None` is returned. Otherwise,
    /// `Option::Some(value)` is returned, where `value` is the value stored at `key`.
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
    ///     let retrieved_value = storage.map.get(key).unwrap();
    ///     assert(value == retrieved_value);
    /// }
    /// ```
    #[storage(read)]
    pub fn get(self, key: K) -> Option<V> {
        let key = sha256((key, __get_storage_key()));
        get::<V>(key)
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
        let key = sha256((key, __get_storage_key()));
        clear::<V>(key)
    }
}

/// A persistant vector struct.
pub struct StorageVec<V> {}

impl<V> StorageVec<V> {
    /// Appends the value to the end of the vector.
    ///
    /// ### Arguments
    ///
    /// * `value` - The item being added to the end of the vector.
    ///
    /// ### Number of Storage Accesses
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
    ///     assert(five == storage.vec.get(0).unwrap());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn push(self, value: V) {
        // The length of the vec is stored in the __get_storage_key() slot
        let len = get::<u64>(__get_storage_key()).unwrap_or(0);

        // Storing the value at the current length index (if this is the first item, starts off at 0)
        let key = sha256((len, __get_storage_key()));
        store::<V>(key, value);

        // Incrementing the length
        store(__get_storage_key(), len + 1);
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
    pub fn pop(self) -> Option<V> {
        let len = get::<u64>(__get_storage_key()).unwrap_or(0);

        // if the length is 0, there is no item to pop from the vec
        if len == 0 {
            return Option::None;
        }

        // reduces len by 1, effectively removing the last item in the vec
        store(__get_storage_key(), len - 1);

        let key = sha256((len - 1, __get_storage_key()));
        get::<V>(key)
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
    ///     assert(five == storage.vec.get(0).unwrap());
    ///     assert(storage.vec.get(1).is_none())
    /// }
    /// ```
    #[storage(read)]
    pub fn get(self, index: u64) -> Option<V> {
        let len = get::<u64>(__get_storage_key()).unwrap_or(0);

        // if the index is larger or equal to len, there is no item to return
        if len <= index {
            return Option::None;
        }

        get::<V>(sha256((index, __get_storage_key())))
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
    pub fn remove(self, index: u64) -> V {
        let len = get::<u64>(__get_storage_key()).unwrap_or(0);

        // if the index is larger or equal to len, there is no item to remove
        assert(index < len);

        // gets the element before removing it, so it can be returned
        let removed_element = get::<V>(sha256((index, __get_storage_key()))).unwrap();

        // for every element in the vec with an index greater than the input index,
        // shifts the index for that element down one
        let mut count = index + 1;
        while count < len {
            // gets the storage location for the previous index
            let key = sha256((count - 1, __get_storage_key()));
            // moves the element of the current index into the previous index
            store::<V>(key, get::<V>(sha256((count, __get_storage_key()))).unwrap());

            count += 1;
        }

        // decrements len by 1
        store(__get_storage_key(), len - 1);

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
    ///     let swapped_value = storage.vec.get(0).unwrap();
    ///     assert(15 == swapped_value);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn swap_remove(self, index: u64) -> V {
        let len = get::<u64>(__get_storage_key()).unwrap_or(0);

        // if the index is larger or equal to len, there is no item to remove
        assert(index < len);

        let hash_of_to_be_removed = sha256((index, __get_storage_key()));
        // gets the element before removing it, so it can be returned
        let element_to_be_removed = get::<V>(hash_of_to_be_removed).unwrap();

        let last_element = get::<V>(sha256((len - 1, __get_storage_key()))).unwrap();
        store::<V>(hash_of_to_be_removed, last_element);

        // decrements len by 1
        store(__get_storage_key(), len - 1);

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
    ///     let set_value = storage.vec.get(0).unwrap();
    ///     assert(20 == set_value);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn set(self, index: u64, value: V) {
        let len = get::<u64>(__get_storage_key()).unwrap_or(0);

        // if the index is higher than or equal len, there is no element to set
        assert(index < len);

        let key = sha256((index, __get_storage_key()));
        store::<V>(key, value);
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
    ///     assert(5 == storage.vec.get(0).unwrap());
    ///     assert(10 == storage.vec.get(1).unwrap());
    ///     assert(15 == storage.vec.get(2).unwrap());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn insert(self, index: u64, value: V) {
        let len = get::<u64>(__get_storage_key()).unwrap_or(0);

        // if the index is larger than len, there is no space to insert
        assert(index <= len);

        // if len is 0, index must also be 0 due to above check
        if len == index {
            let key = sha256((index, __get_storage_key()));
            store::<V>(key, value);

            // increments len by 1
            store(__get_storage_key(), len + 1);

            return;
        }

        // for every element in the vec with an index larger than the input index,
        // move the element up one index.
        // performed in reverse to prevent data overwriting
        let mut count = len - 1;
        while count >= index {
            let key = sha256((count + 1, __get_storage_key()));
            // shifts all the values up one index
            store::<V>(key, get::<V>(sha256((count, __get_storage_key()))).unwrap());

            count -= 1
        }

        // inserts the value into the now unused index
        let key = sha256((index, __get_storage_key()));
        store::<V>(key, value);

        // increments len by 1
        store(__get_storage_key(), len + 1);
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
    pub fn len(self) -> u64 {
        get::<u64>(__get_storage_key()).unwrap_or(0)
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
    pub fn is_empty(self) -> bool {
        get::<u64>(__get_storage_key()).unwrap_or(0) == 0
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
    pub fn clear(self) {
        let _ = clear::<u64>(__get_storage_key());
    }

    /// Swaps two elements.
    ///
    /// ### Arguments
    ///
    /// * element1_index - The index of the first element.
    /// * element2_index - The index of the second element.
    ///
    /// ### Reverts
    ///
    /// * If `element1_index` or `element2_index` is greater than the length of the vector.
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
    ///
    ///     storage.vec.swap(0, 2);
    ///     assert(15 == storage.vec.get(0).unwrap());
    ///     assert(10 == storage.vec.get(1).unwrap());
    ///     assert(5 == storage.vec.get(2).unwrap());
    /// ```
    #[storage(read, write)]
    pub fn swap(self, element1_index: u64, element2_index: u64) {
        let len = get::<u64>(__get_storage_key()).unwrap_or(0);
        assert(element1_index < len);
        assert(element2_index < len);

        if element1_index == element2_index {
            return;
        }

        let element1_key = sha256((element1_index, __get_storage_key()));
        let element2_key = sha256((element2_index, __get_storage_key()));

        let element1_value = get::<V>(element1_key).unwrap();
        store::<V>(element1_key, get::<V>(element2_key).unwrap());
        store::<V>(element2_key, element1_value);
    }

    /// Returns the first element of the vector, or `None` if it is empty.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads: `2`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///    assert(storage.vec.first().is_none());
    ///
    ///     storage.vec.push(5);
    ///
    ///     assert(5 == storage.vec.first().unwrwap());
    /// }
    /// ```
    #[storage(read)]
    pub fn first(self) -> Option<V> {
        match get::<u64>(__get_storage_key()).unwrap_or(0) {
            0 => Option::None,
            _ => get::<V>(sha256((0, __get_storage_key()))),
        }
    }

    /// Returns the last element of the vector, or `None` if it is empty.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads: `2`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///    assert(storage.vec.last().is_none());
    ///
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///
    ///     assert(10 == storage.vec.last().unwrap());
    /// }
    /// ```
    #[storage(read)]
    pub fn last(self) -> Option<V> {
        match get::<u64>(__get_storage_key()).unwrap_or(0) {
            0 => Option::None,
            len => get::<V>(sha256((len - 1, __get_storage_key()))),
        }
    }

    /// Reverses the order of elements in the vector, in place.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads: `1 + (2 * (self.len() / 2))`
    /// * Writes: `2 * (self.len() / 2)`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     storage.vec.push(15);
    ///     storage.vec.reverse();
    ///
    ///     assert(15 == storage.vec.get(0).unwrap());
    ///     assert(10 == storage.vec.get(1).unwrap());
    ///     assert(5 == storage.vec.get(2).unwrap());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn reverse(self) {
        let len = get::<u64>(__get_storage_key()).unwrap_or(0);

        if len < 2 {
            return;
        }

        let mid = len / 2;
        let mut i = 0;
        while i < mid {
            let element1_key = sha256((i, __get_storage_key()));
            let element2_key = sha256((len - i - 1, __get_storage_key()));

            let element1_value = get::<V>(element1_key).unwrap();
            store::<V>(element1_key, get::<V>(element2_key).unwrap());
            store::<V>(element2_key, element1_value);

            i += 1;
        }
    }

    /// Fills `self` with elements by cloning `value`.
    ///
    /// ### Arguments
    ///
    /// * value - Value to copy to each element of the vector.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads: `1`
    /// * Writes: `self.len()`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     storage.vec.push(15);
    ///     storage.vec.fill(20);
    ///
    ///     assert(20 == storage.vec.get(0).unwrap());
    ///     assert(20 == storage.vec.get(1).unwrap());
    ///     assert(20 == storage.vec.get(2).unwrap());
    /// }
    /// ```
    #[storage(read, write)]
    pub fn fill(self, value: V) {
        let len = get::<u64>(__get_storage_key()).unwrap_or(0);

        let mut i = 0;
        while i < len {
            store::<V>(sha256((i, __get_storage_key())), value);
            i += 1;
        }
    }

    /// Resizes `self` in place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, `self` is extended by the difference, with each
    /// additional slot being filled with `value`. If the `new_len` is less than `len`, `self` is
    /// simply truncated.
    ///
    /// ### Arguments
    ///
    /// * new_len - The new length to expand or truncate to
    /// * value - The value to fill into new slots if the `new_len` is greater than the current length
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads - `1`
    /// * Writes - `if new_len > self.len() { new_len - len + 1 } else { 1 }`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// storage {
    ///     vec: StorageVec<u64> = StorageVec {},
    /// }
    ///
    /// fn foo() {
    ///     storage.vec.push(5);
    ///     storage.vec.push(10);
    ///     storage.vec.resize(4, 20);
    ///
    ///     assert(5 == storage.vec.get(0).unwrap());
    ///     assert(10 == storage.vec.get(1).unwrap());
    ///     assert(20 == storage.vec.get(2).unwrap());
    ///     assert(20 == storage.vec.get(3).unwrap());
    ///
    ///     storage.vec.resize(2, 0);
    ///
    ///     assert(5 == storage.vec.get(0).unwrap());
    ///     assert(10 == storage.vec.get(1).unwrap());
    ///     assert(Option::None == storage.vec.get(2));
    ///     assert(Option::None == storage.vec.get(3));
    /// }
    /// ```
    #[storage(read, write)]
    pub fn resize(self, new_len: u64, value: V) {
        let mut len = get::<u64>(__get_storage_key()).unwrap_or(0);
        while len < new_len {
            store::<V>(sha256((len, __get_storage_key())), value);
            len += 1;
        }
        store::<u64>(__get_storage_key(), new_len);
    }
}

pub struct StorageBytes {}

impl StorableSlice<Bytes> for StorageBytes {
    /// Takes a `Bytes` type and stores the underlying collection of tightly packed bytes.
    ///
    /// ### Arguments
    ///
    /// * `bytes` - The bytes which will be stored.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Writes: `2`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// storage {
    ///     stored_bytes: StorageBytes = StorageBytes {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///
    ///     storage.stored_bytes.store(bytes);
    /// }
    /// ```
    #[storage(write)]
    fn store(self, bytes: Bytes) {
        let key = __get_storage_key();
        store_slice(key, bytes.as_raw_slice());
    }

    /// Constructs a `Bytes` type from a collection of tightly packed bytes in storage.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads: `2`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// storage {
    ///     stored_bytes: StorageBytes = StorageBytes {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///
    ///     assert(storage.stored_bytes.load(key).is_none());
    ///     storage.stored_bytes.store(bytes);
    ///     let retrieved_bytes = storage.stored_bytes.load(key).unwrap();
    ///     assert(bytes == retrieved_bytes);
    /// }
    /// ```
    #[storage(read)]
    fn load(self) -> Option<Bytes> {
        let key = __get_storage_key();
        match get_slice(key) {
            Option::Some(slice) => {
                 Option::Some(Bytes::from_raw_slice(slice))
            },
            Option::None => Option::None,
        }
    }

    /// Clears a collection of tightly packed bytes in storage.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads: `1`
    /// * Clears: `2`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// storage {
    ///     stored_bytes: StorageBytes = StorageBytes {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///     storage.stored_bytes.store(bytes);
    ///
    ///     assert(storage.stored_bytes.load(key).is_some());
    ///     let cleared = storage.stored_bytes.clear();
    ///     assert(cleared);
    ///     let retrieved_bytes = storage.stored_bytes.load(key);
    ///     assert(retrieved_bytes.is_none());
    /// }
    /// ```
    #[storage(read, write)]
    fn clear(self) -> bool {
        let key = __get_storage_key();
        clear_slice(key)
    }

    /// Returns the length of tightly packed bytes in storage.
    ///
    /// ### Number of Storage Accesses
    ///
    /// * Reads: `1`
    ///
    /// ### Examples
    ///
    /// ```sway
    /// storage {
    ///     stored_bytes: StorageBytes = StorageBytes {}
    /// }
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(5_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(9_u8);
    ///
    ///     assert(storage.stored_bytes.len() == 0)
    ///     storage.stored_bytes.store(bytes);
    ///     assert(storage.stored_bytes.len() == 3);
    /// }
    /// ```
    #[storage(read)]
    fn len(self) -> u64 {
        get::<u64>(__get_storage_key()).unwrap_or(0)
    }
}
