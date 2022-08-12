//! Library for allocating memory.
//! Inspired from: https://doc.rust-lang.org/std/alloc/index.html
library alloc;

use ::mem::copy;
use ::context::registers::stack_ptr;

/// Allocates zeroed memory on the heap
///
/// In FuelVM, the heap begins at `VM_MAX_RAM - 1` and grows downward.
/// Heap pointer `$hp` will always point to unallocated space.
///
/// Initially the heap will look like this:
/// ... 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |
///                                               $hp^  ^VM_MAX_RAM
///
/// After allocating with `let ptr = alloc(8)`:
/// ... 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |
///                       $hp^  ^ptr                    ^VM_MAX_RAM
///
/// After writing with `sw(ptr, u64::max())`:
/// ... 00 00 00 00 00 00 00 00 FF FF FF FF FF FF FF FF |
///                       $hp^  ^ptr                    ^VM_MAX_RAM
///
/// See: https://github.com/FuelLabs/fuel-specs/blob/master/specs/vm/main.md#vm-initialization
/// See: https://github.com/FuelLabs/fuel-specs/blob/master/specs/vm/opcodes.md#aloc-allocate-memory
pub fn alloc(size: u64) -> u64 {
    asm(size: size, ptr) {
        aloc size;
        // `$hp` points to unallocated space and heap grows downward so
        // our newly allocated space will be right after it
        addi ptr hp i1;
        ptr: u64
    }
}

/// Reallocates the given area of memory
pub fn realloc(ptr: u64, size: u64, new_size: u64) -> u64 {
    if new_size > size {
        let new_ptr = alloc(new_size);
        if size > 0 {
            copy(ptr, new_ptr, size);
        }
        new_ptr
    } else {
        ptr
    }
}

// Allocate a type on the stack.
pub fn alloca<T>() -> u64 {
    let current_pointer = stack_ptr();
    asm() {
        cfei i32;
    };
    current_pointer
}
