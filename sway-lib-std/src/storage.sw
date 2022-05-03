library storage;

/// Store a stack variable in storage.
pub fn store<T>(mutkey: b256, value: T) {
    if !is_reference_type::<T>() {
        // If copy type, then it's a single word and can be stored with a single SWW.
        asm(r1: key, r2: value) {
            sww r1 r2;
        };
    } else {
        // If reference type, then it's more than a word. Loop over every 32 bytes and
        // store sequentially.
        let mut size_left = size_of::<T>();
        let mut ptr = asm(r1, r2: value) {
            move r1 r2;
            r1
        };
        while size_left > 32 {
            asm(r1: key, r2: ptr) {
                swwq r1 r2;
            };
            ptr = ptr + 32;
            key
            size_left = size_left - 32;
        }
        asm(r1: key, r2: ptr) {
            swwq r1 r2;
        };
    };
}

/// Load a stack variable from storage.
pub fn get<T>(key: b256) -> T {
    if !is_reference_type::<T>() {
        // If copy type, then it's a single word and can be read with a single SRW.
        asm(r1: key, r2) {
            srw r2 r1;
            r2: T
        }
    } else {
        // If reference type, then it's more than a word. Loop over every 32 bytes and
        // read sequentially.
        let mut size_left = size_of::<T>();
        while size_left > 32 {
            asm(r1: key, r2) {
                srwq r2 r1;
            };
            // increment key to next slot...
            size_left = size_left - 32;
        }
        asm(r1: key, r2) {
            srwq r2 r1;
        };
    }
}
