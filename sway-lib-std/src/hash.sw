library hash;

use ::core::num::*;

/// Returns the SHA-2-256 hash of `param`.
pub fn sha256<T>(param: T) -> b256 {
    let mut result_buffer: b256 = ~b256::min();
    if !__is_reference_type::<T>() {
        asm(buffer, ptr: param, eight_bytes: 8, hash: result_buffer) {
            move buffer sp; // Make `buffer` point to the current top of the stack
            cfei i8; // Grow stack by 1 word
            sw buffer ptr i0; // Save value in register at "ptr" to memory at "buffer"
            s256 hash buffer eight_bytes; // Hash the next eight bytes starting from "buffer" into "hash"
            cfsi i8; // Shrink stack by 1 word
            hash: b256 // Return
        }
    } else {
        let size = __size_of::<T>();
        asm(hash: result_buffer, ptr: param, bytes: size) {
            s256 hash ptr bytes; // Hash the next "size" number of bytes starting from "ptr" into "hash"
            hash: b256 // Return
        }
    }
}

/// Returns the KECCAK-256 hash of `param`.
pub fn keccak256<T>(param: T) -> b256 {
    let mut result_buffer: b256 = ~b256::min();
    if !__is_reference_type::<T>() {
        asm(buffer, ptr: param, eight_bytes: 8, hash: result_buffer) {
            move buffer sp; // Make `buffer` point to the current top of the stack
            cfei i8; // Grow stack by 1 word
            sw buffer ptr i0; // Save value in register at "ptr" to memory at "buffer"
            k256 hash buffer eight_bytes; // Hash the next eight bytes starting from "buffer" into "hash"
            cfsi i8; // Shrink stack by 1 word
            hash: b256 // Return
        }
    } else {
        let size = __size_of::<T>();
        asm(hash: result_buffer, ptr: param, bytes: size) {
            k256 hash ptr bytes; // Hash the next "size" number of bytes starting from "ptr" into "hash"
            hash: b256 // Return
        }
    }
}
