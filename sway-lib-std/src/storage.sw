library r#storage;

use ::alloc::alloc;
use ::assert::assert;
use ::hash::sha256;
use ::option::Option;
use ::result::Result;

/// Store a stack value in storage. Will not work for heap values.
/// 
/// ### Arguments
///
/// * `key` - The storage slot at which the variable will be stored
/// * `value` - The value to be stored
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
#[storage(write)]
pub fn store<T>(key: b256, value: T) {
    if !__is_reference_type::<T>() {
        // If copy type, then it's a single word
        let value = asm(v: value) { v: u64 };
        __state_store_word(key, value);
    } else {
        // If reference type, then it can be more than a word. Loop over every 32
        // bytes and store sequentially.
        let mut size_left = __size_of::<T>();
        let mut local_key = key;

        // Cast the pointer to `value` to a u64. This lets us increment
        // this pointer later on to iterate over 32 byte chunks of `value`.
        let mut ptr_to_value = asm(ptr: value) { ptr: raw_ptr };

        while size_left > 32 {
            // Store a 4 words (32 byte) at a time
            __state_store_quad(local_key, ptr_to_value);

            // Move by 32 bytes
            ptr_to_value = ptr_to_value.add::<b256>(1);
            size_left -= 32;

            // Generate a new key for each 32 byte chunk TODO Should eventually
            // replace this with `local_key = local_key + 1
            local_key = sha256(local_key);
        }

        // Store the leftover bytes using a single quad store
        __state_store_quad(local_key, ptr_to_value);
    };
}

/// Load a value from storage.
///
/// If the value size is larger than 8 bytes it is read to a heap buffer which is leaked for the
/// duration of the program.
///
/// If no value was previously stored using the key, a byte representation of all zeroes is returned
/// Read more [here](https://fuellabs.github.io/sway/master/common-collections/storage_map.html#accessing-values-in-a-storage-map)
///
/// ### Arguments
///
/// * `key` - The storage slot to load the value from
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
pub fn get<T>(key: b256) -> T {
    if !__is_reference_type::<T>() {
        // If copy type, then it's a single word
        let loaded_word = __state_load_word(key);
        asm(l: loaded_word) { l: T }
    } else {
        // If reference type, then it can be more than a word. Loop over every 32
        // bytes and read sequentially.  NOTE: we are leaking this value on the heap.
        let mut size_left = __size_of::<T>();
        let mut local_key = key;

        // Allocate a buffer for the result.  It needs to be a multiple of 32 bytes so we can make
        // 'quad' storage reads without overflowing.
        let result_ptr = alloc::<u64>(((size_left + 31) & 0xffffffe0) / 8);

        let mut current_pointer = result_ptr;
        while size_left > 32 {
            // Read 4 words (32 bytes) at a time
            __state_load_quad(local_key, current_pointer);

            // Move by 32 bytes
            size_left -= 32;
            current_pointer = current_pointer.add::<b256>(1);

            // Generate a new key for each 32 byte chunk TODO Should eventually
            // replace this with `local_key = local_key + 1
            local_key = sha256(local_key);
        }

        // Read the leftover bytes using a single `srwq`
        __state_load_quad(local_key, current_pointer);

        // Return the final result as type T
        asm(res: result_ptr) { res: T }
    }
}

/// A persistent key-value pair mapping struct
pub struct StorageMap<K, V> {}

impl<K, V> StorageMap<K, V> {
    /// Inserts a key-value pair into the map.
    /// 
    /// ### Arguments
    ///
    /// * `key` - The key to which the value is paired
    /// * `value` - The value to be stored
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
    fn insert(self, key: K, value: V) {
        let key = sha256((key, __get_storage_key()));
        store::<V>(key, value);
    }

    /// Retrieves a value previously stored using a key
    /// 
    /// If no value was previously stored using the key, a byte representation of all zeroes is returned
    /// Read more [here](https://fuellabs.github.io/sway/master/common-collections/storage_map.html#accessing-values-in-a-storage-map)
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
    ///     let retrieved_value = storage.map.get(key);
    ///     assert(value == retrieved_value);
    /// }
    /// ```
    #[storage(read)]
    fn get(self, key: K) -> V {
        let key = sha256((key, __get_storage_key()));
        get::<V>(key)
    }
}

/// A persistant vector struct
pub struct StorageVec<V> {}

impl<V> StorageVec<V> {
    /// Appends the value to the end of the vector
    ///
    /// ### Arguments
    ///
    /// * `value` - The item being added to the end of the vector
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
    ///     let retrieved_value = storage.vec.get(0).unwrap();
    ///     assert(five == retrieved_value);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn push(self, value: V) {
        // The length of the vec is stored in the __get_storage_key() slot
        let len = get::<u64>(__get_storage_key());

        // Storing the value at the current length index (if this is the first item, starts off at 0)
        let key = sha256((len, __get_storage_key()));
        store::<V>(key, value);

        // Incrementing the length
        store(__get_storage_key(), len + 1);
    }

    /// Removes the last element of the vector and returns it, None if empty
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
        let len = get::<u64>(__get_storage_key());
        // if the length is 0, there is no item to pop from the vec
        if len == 0 {
            return Option::None;
        }

        // reduces len by 1, effectively removing the last item in the vec
        store(__get_storage_key(), len - 1);

        let key = sha256((len - 1, __get_storage_key()));
        Option::Some::<V>(get::<V>(key))
    }

    /// Gets the value in the given index, None if index is out of bounds
    ///
    /// ### Arguments
    ///
    /// * `index` - The index of the vec to retrieve the item from
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
    ///     let retrieved_value = storage.vec.get(0).unwrap();
    ///     assert(five == retrieved_value);
    ///     let none_value = storage.vec.get(1);
    ///     assert(none_value.is_none())
    /// }
    /// ```
    #[storage(read)]
    pub fn get(self, index: u64) -> Option<V> {
        let len = get::<u64>(__get_storage_key());
        // if the index is larger or equal to len, there is no item to return
        if len <= index {
            return Option::None;
        }

        let key = sha256((index, __get_storage_key()));
        Option::Some::<V>(get::<V>(key))
    }

    /// Removes the element in the given index and moves all the element in the following indexes
    /// Down one index. Also returns the element
    ///
    /// # WARNING
    ///
    /// Expensive for larger vecs
    ///
    /// ### Arguments
    ///
    /// * `index` - The index of the vec to remove the item from
    ///
    /// ### Reverts
    ///
    /// Reverts if index is larger or equal to length of the vec
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
        let len = get::<u64>(__get_storage_key());
        // if the index is larger or equal to len, there is no item to remove
        assert(index < len);

        // gets the element before removing it, so it can be returned
        let removed_element = get::<V>(sha256((index, __get_storage_key())));

        // for every element in the vec with an index greater than the input index,
        // shifts the index for that element down one
        let mut count = index + 1;
        while count < len {
            // gets the storage location for the previous index
            let key = sha256((count - 1, __get_storage_key()));
            // moves the element of the current index into the previous index
            store::<V>(key, get::<V>(sha256((count, __get_storage_key()))));

            count += 1;
        }

        // decrements len by 1
        store(__get_storage_key(), len - 1);

        removed_element
    }

    /// Removes the element at the specified index and fills it with the last element
    /// Does not preserve ordering. Also returns the element
    ///
    /// ### Arguments
    ///
    /// * `index` - The index of the vec to remove the item from
    ///
    /// ### Reverts
    ///
    /// Reverts if index is larger or equal to length of the vec
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
    ///     let swapped_value = storage.vec.get(0);
    ///     assert(15 == swapped_value);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn swap_remove(self, index: u64) -> V {
        let len = get::<u64>(__get_storage_key());
        // if the index is larger or equal to len, there is no item to remove
        assert(index < len);

        let hash_of_to_be_removed = sha256((index, __get_storage_key()));
        // gets the element before removing it, so it can be returned
        let element_to_be_removed = get::<V>(hash_of_to_be_removed);

        let last_element = get::<V>(sha256((len - 1, __get_storage_key())));
        store::<V>(hash_of_to_be_removed, last_element);

        // decrements len by 1
        store(__get_storage_key(), len - 1);

        element_to_be_removed
    }
    /// Sets/mutates the value at the given index
    ///
    /// ### Arguments
    ///
    /// * `index` - The index of the vec to set the value at
    /// * `value` - The value to be set
    ///
    /// ### Reverts
    ///
    /// Reverts if index is larger than or equal to the length of the vec
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
    ///     let set_value = storage.vec.get(0);
    ///     assert(20 == set_value);
    /// }
    /// ```
    #[storage(read, write)]
    pub fn set(self, index: u64, value: V) {
        let len = get::<u64>(__get_storage_key());
        // if the index is higher than or equal len, there is no element to set
        assert(index < len);

        let key = sha256((index, __get_storage_key()));
        store::<V>(key, value);
    }

    /// Inserts the value at the given index, moving the current index's value aswell as the following's
    /// Up one index
    ///
    /// # WARNING
    ///
    /// Expensive for larger vecs
    ///
    /// ### Arguments
    ///
    /// * `index` - The index of the vec to insert the item into
    /// * `value` - The value to insert into the vec
    ///
    /// ### Reverts
    ///
    /// Reverts if index is larger than length of the vec
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
    ///     assert(5 == storage.vec.get(0));
    ///     assert(10 == storage.vec.get(1))
    ///     assert(15 == storage.vec.get(2));
    /// }
    /// ```
    #[storage(read, write)]
    pub fn insert(self, index: u64, value: V) {
        let len = get::<u64>(__get_storage_key());
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
            store::<V>(key, get::<V>(sha256((count, __get_storage_key()))));

            count -= 1
        }

        // inserts the value into the now unused index
        let key = sha256((index, __get_storage_key()));
        store::<V>(key, value);

        // increments len by 1
        store(__get_storage_key(), len + 1);
    }

    /// Returns the length of the vector
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
    ///
    ///     assert(1 == storage.vec.len());
    ///
    ///     storage.vec.push(10);
    /// 
    ///     assert(2 == storage.vec.len());    
    /// }
    /// ```
    #[storage(read)]
    pub fn len(self) -> u64 {
        get::<u64>(__get_storage_key())
    }

    /// Checks whether the len is 0 or not
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
        let len = get::<u64>(__get_storage_key());
        len == 0
    }

    /// Sets the len to 0
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
    ///
    ///     storage.vec.push(5);
    ///
    ///     assert(1 == storage.vec.len());
    ///
    ///     storage.vec.clear();
    /// 
    ///     assert(0 == storage.vec.len());
    /// }
    /// ```
    #[storage(write)]
    pub fn clear(self) {
        store(__get_storage_key(), 0);
    }
}
