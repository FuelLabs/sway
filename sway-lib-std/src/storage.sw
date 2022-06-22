library storage;

use ::hash::sha256;
use ::option::Option;
use ::result::Result;
use ::context::registers::stack_ptr;

/// Store a stack variable in storage.
#[storage(write)]pub fn store<T>(key: b256, value: T) {
    if !__is_reference_type::<T>() {
        // If copy type, then it's a single word and can be stored with a single SWW.
        asm(k: key, v: value) {
            sww k v;
        };
    } else {
        // If reference type, then it's more than a word. Loop over every 32
        // bytes and store sequentially.
        let mut size_left = __size_of::<T>();
        let mut local_key = key;

        // Cast the the pointer to `value` to a u64. This lets us increment
        // this pointer later on to iterate over 32 byte chunks of `value`.
        let mut ptr_to_value = asm(v: value) {
            v
        };

        while size_left > 32 {
            // Store a 4 words (32 byte) at a time using `swwq`
            asm(k: local_key, v: ptr_to_value) {
                swwq k v;
            };

            // Move by 32 bytes
            ptr_to_value = ptr_to_value + 32;
            size_left -= 32;

            // Generate a new key for each 32 byte chunk TODO Should eventually
            // replace this with `local_key = local_key + 1
            local_key = sha256(local_key);
        }

        // Store the leftover bytes using a single `swwq`
        asm(k: local_key, v: ptr_to_value) {
            swwq k v;
        };
    };
}

/// Load a stack variable from storage.
#[storage(read)]pub fn get<T>(key: b256) -> T {
    if !__is_reference_type::<T>() {
        // If copy type, then it's a single word and can be read with a single
        // SRW.
        asm(k: key, v) {
            srw v k;
            v: T
        }
    } else {
        // If reference type, then it's more than a word. Loop over every 32
        // bytes and read sequentially.
        let mut size_left = __size_of::<T>();
        let mut local_key = key;

        // Keep track of the base pointer for the final result
        let result_ptr = stack_ptr();

        while size_left > 32 {
            // Read 4 words (32 bytes) at a time using `srwq`
            let current_pointer = stack_ptr();
            asm(k: local_key, v: current_pointer) {
                cfei i32;
                srwq v k;
            };

            // Move by 32 bytes
            size_left -= 32;

            // Generate a new key for each 32 byte chunk TODO Should eventually
            // replace this with `local_key = local_key + 1
            local_key = sha256(local_key);
        }

        // Read the leftover bytes using a single `srwq`
        let current_pointer = stack_ptr();
        asm(k: local_key, v: current_pointer) {
            cfei i32;
            srwq v k;
        }

        // Return the final result as type T
        asm(res: result_ptr) {
            res: T
        }
    }
}

pub struct StorageMap<K, V> {
}

impl<K, V> StorageMap<K, V> {
    #[storage(write)]fn insert(self, key: K, value: V) {
        let key = sha256((key, __get_storage_key()));
        store::<V>(key, value);
    }

    #[storage(read)]fn get(self, key: K) -> V {
        let key = sha256((key, __get_storage_key()));
        get::<V>(key)
    }
}


enum StorageVecError {
    IndexOutOfBounds: (),
    CannotSwapAndRemoveTheSameElement: (),
}

/// A persistant vector struct
pub struct StorageVec<V> {}

impl<V> StorageVec<V> {
    /// Appends the value to the end of the vector
    #[storage(read, write)]
    pub fn push(self, value: V) {
        // The length of the vec is stored in the __get_storage_key() slot
        let len = get::<u64>(__get_storage_key());
        
        // Storing the value at the current length index (if this is the first item, starts off at 0)
        let key = sha256((len, __get_storage_key()));
        store(key, value);

        // Incrementing the length
        store(__get_storage_key(), len + 1);
    }

    /// Removes the last element of the vector
    #[storage(read, write)]
    pub fn pop(self) -> Option<V> {
        let len = get::<u64>(__get_storage_key());
        // if the length is 0, there is no item to pop from the vec
        if len == 0 {
            return Option::None;
        }
    
        // reduces len by 1, effectively removing the last item in the vec
        store(__get_storage_key(), len - 1);

        let key = sha256((len, __get_storage_key()));
        Option::Some(get(key))
    }

    /// Gets the value in the given index
    #[storage(read)]
    pub fn get(self, index: u64) -> Option<V> {
        let len = get::<u64>(__get_storage_key());
        // if the index is larger or equal to len, there is no item to return
        if len <= index {
            return Option::None;
        }

        let key = sha256((index, __get_storage_key()));
        Option::Some(get(key))
    }

    /// Removes the value in the given index and moves all the values in the following indexes
    /// Down one index
    /// WARNING: Expensive for larger vecs
    #[storage(read, write)]
    pub fn remove(self, index: u64) -> Result<V, StorageVecError> {
        let len = get::<u64>(__get_storage_key());
        // if the index is larger or equal to len, there is no item to remove
        if len <= index {
            return Result::Err(StorageVecError::IndexOutOfBounds);
        }

        // gets the element before removing it, so it can be returned
        let element_to_be_removed = get(sha256((index, __get_storage_key())));

        // for every element in the vec with an index greater than the input index,
        // shifts the index for that element down one
        let mut count = index + 1;
        while count < len {
            // gets the storage location for the previous index
            let key = sha256((count - 1, __get_storage_key()));
            // moves the element of the current index into the previous index
            store(key, get(sha256((count, __get_storage_key()))));
            
            count += 1;
        }

        // decrements len by 1
        store(__get_storage_key(), len - 1);

        // returns the removed element
        Result::Ok(element_to_be_removed)
    }

    /// Removes the element at the specified index and fills it with the last element
    /// Does not preserve ordering
    #[storage(read, write)]
    pub fn swap_remove(self, index: u64) -> Result<V, StorageVecError> {
        let len = get::<u64>(__get_storage_key());
        // if the index is larger or equal to len, there is no item to remove
        if len <= index {
            return Result::Err(StorageVecError::IndexOutOfBounds);
        }

        // if the index is the last one, the function should not try to remove and then swap the same element
        if index == len - 1 {
            return Result::Err(StorageVecError::CannotSwapAndRemoveTheSameElement);
        }

        // gets the element before removing it, so it can be returned
        let element_to_be_removed = get(sha256((index, __get_storage_key())));

        let last_element = get(sha256(len - 1, __get_storage_key()));
        store(sha256(index, __get_storage_key()), last_element);

        // decrements len by 1
        store(__get_storage_key(), len - 1);

        Result::Ok(element_to_be_removed))
    }

    /// Inserts the value at the given index, moving the current index's value aswell as the following's
    /// Up one index
    /// WARNING: Expensive for larger vecs
    #[storage(read, write)]
    pub fn insert(self, index: u64, value: V) -> Result<(), StorageVecError> {
        let len = get::<u64>(__get_storage_key());
        // if the index is larger or equal to len, there is no space to insert
        if index >= len {
            return Result::Err(StorageVecError::IndexOutOfBounds);
        }

        // for every element in the vec with an index larger than the input index,
        // move the element up one index.
        // performed in reverse to prevent data overwriting
        let mut count = len;
        while count > index {
            let key = sha256((count + 1, __get_storage_key()));
            // shifts all the values up one index
            store(key, get(sha256((count, __get_storage_key()))));

            count -= 1
        }

        // inserts the value into the now unused index
        let key = sha256((index, __get_storage_key()));
        store(key, value);

        // increments len by 1
        store(__get_storage_key(), len + 1);
        Result::Ok(())
    }

    /// Returns the length of the vector
    #[storage(read)]
    pub fn len(self) -> u64 {
        get::<u64>(__get_storage_key())
    }

    /// Checks whether the len is 0 or not
    #[storage(read)]
    pub fn is_empty(self) -> bool {
        let len = get::<u64>(__get_storage_key());

        len == 0
    }

    /// Sets the len to 0
    #[storage(write)]
    pub fn clear(self) {
        store(__get_storage_key(), 0);
    }
}
