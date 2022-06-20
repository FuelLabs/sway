library storage;

use ::hash::sha256;
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

        // Cast the pointer to `value` to a u64. This lets us increment
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
