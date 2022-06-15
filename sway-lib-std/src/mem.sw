//! Library for working with memory.
library mem;

use ::revert::revert;
use ::intrinsics::{is_reference_type, size_of_val};

/// Returns the address of the given value.
pub fn addr_of<T>(val: T) -> u64 {
    if !__is_reference_type::<T>() {
        revert(0);
    }
    asm(ptr: val) {
        ptr: u64
    }
}

/// Copies bytes from src to dst.
pub fn copy(dst: u64, src: u64, size: u64) {
    asm(dst: dst, src: src, size: size) {
        mcp dst src size;
    };
}

/// Determines whether the raw bytes at two points of memory are equal.
pub fn eq(first: u64, second: u64, len: u64) -> bool {
    asm(first: first, second: second, len: len, result) {
        meq result first second len;
        result: bool
    }
}
