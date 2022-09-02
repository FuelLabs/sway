//! Library for working with memory.
library mem;

use ::intrinsics::{is_reference_type, size_of_val};
use ::revert::revert;

/// Returns the address of the given value.
pub fn addr_of<T>(val: T) -> raw_ptr {
    if !__is_reference_type::<T>() {
        revert(0);
    }
    asm(ptr: val) {
        ptr: raw_ptr
    }
}

/// Offsets the pointer by the given number of bytes
pub fn ptr_offset(ptr: raw_ptr, amount: u64) -> raw_ptr {
    asm(r1: ptr, r2: amount, r3) {
        add r3 r2 r1;
        r3: raw_ptr
    }
}

/// Copies `size` bytes from `src` to `dst`.
pub fn copy(src: raw_ptr, dst: raw_ptr, size: u64) {
    asm(dst: dst, src: src, size: size) {
        mcp dst src size;
    };
}

/// Compares `len` raw bytes in memory at addresses `first` and `second`.
pub fn eq(first: raw_ptr, second: raw_ptr, len: u64) -> bool {
    asm(first: first, second: second, len: len, result) {
        meq result first second len;
        result: bool
    }
}

/// Reads the given type of value from the address.
pub fn read<T>(ptr: raw_ptr) -> T {
    if is_reference_type::<T>() {
        asm(ptr: ptr) {
            ptr: T
        }
    } else {
        asm(ptr: ptr, val) {
            lw val ptr i0;
            val: T
        }
    }
}

/// Writes the given value to the address.
pub fn write<T>(ptr: raw_ptr, val: T) {
    if is_reference_type::<T>() {
        copy(addr_of(val), ptr, size_of_val(val));
    } else {
        asm(ptr: ptr, val: val) {
            sw ptr val i0;
        };
    }
}
