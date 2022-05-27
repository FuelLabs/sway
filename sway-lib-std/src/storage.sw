library storage;

use ::hash::sha256;

/// Store a stack variable in storage.
pub fn store<T>(key: b256, value: T) {
    if !__is_reference_type::<T>() {
        // If copy type, then it's a single word and can be stored with a single SWW.
        asm(r1: key, r2: value) {
            sww r1 r2;
        };
    } else {
        // If reference type, then it's more than a word. Loop over every 32 bytes and
        // store sequentially.
        let mut size_left = __size_of::<T>();
        let mut local_key = key;
        let mut ptr = asm(r1, r2: value) {
            move r1 r2;
            r1
        };
        while size_left > 32 {
            asm(r1: local_key, r2: ptr) {
                swwq r1 r2;
            };
            ptr = ptr + 32;
            size_left = size_left - 32;
            local_key = sha256(local_key);
        }
        asm(r1: local_key, r2: ptr) {
            swwq r1 r2;
        };
    };
}

/// Load a stack variable from storage.
pub fn get<T>(key: b256) -> T {
    let result = if !__is_reference_type::<T>() {
        // If copy type, then it's a single word and can be read with a single SRW.
        asm(r1: key, r2) {
            srw r2 r1;
            r2: T
        }
    } else {
        // If reference type, then it's more than a word. Loop over every 32 bytes and
        // read sequentially.
        let mut size_left = __size_of::<T>();
        let mut local_key = key;
        let result_ptr = asm(r1: size_left, r2) {
            move r2 sp;
            r2: u64 
        };
        while size_left > 32 {
            asm(r1: local_key, r2) {
                move r2 sp;
                cfei i32;
                srwq r2 r1;
            };
            size_left = size_left - 32;
            local_key = sha256(local_key);
        }
        asm(r1: local_key, r2: result_ptr, r3) {
            move r3 sp;
            cfei i32;
            srwq r3 r1;
            r2: T
        }
    };
    result
}
