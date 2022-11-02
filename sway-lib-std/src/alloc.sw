//! Library for allocating memory.
//! Inspired from: https://doc.rust-lang.org/std/alloc/index.html
library alloc;

/// Allocates zeroed memory on the heap
///
/// In FuelVM, the heap begins at `VM_MAX_RAM - 1` and grows downward.
/// Heap pointer `$hp` will always point to unallocated space.
///
/// Initially the heap will look like this:
/// ... 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |
///                                               $hp^  ^VM_MAX_RAM
///
/// After allocating with `let ptr = alloc::<u64>(1)`:
/// ... 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |
///                       $hp^  ^ptr                    ^VM_MAX_RAM
///
/// After writing with `sw(ptr, u64::max())`:
/// ... 00 00 00 00 00 00 00 00 FF FF FF FF FF FF FF FF |
///                       $hp^  ^ptr                    ^VM_MAX_RAM
///
/// See: https://fuellabs.github.io/fuel-specs/master/vm#vm-initialization
/// See: https://fuellabs.github.io/fuel-specs/master/vm/instruction_set.html#aloc-allocate-memory
pub fn alloc<T>(count: u64) -> raw_slice {
    let size = __size_of::<T>() * count;
    let ptr = asm(size: size, ptr) {
        aloc size;
        // `$hp` points to unallocated space and heap grows downward so
        // our newly allocated space will be right after it
        addi ptr hp i1;
        ptr: raw_ptr
    };
    raw_slice::from_raw_parts(ptr, size)
}

/// Reallocates the given area of memory
pub fn realloc<T>(buf: raw_slice, new_count: u64) -> raw_slice {
    let count = buf.len::<T>();
    if new_count > count {
        let new_buf = alloc::<T>(new_count);
        if count > 0 {
            buf.copy_to::<T>(new_buf);
        }
        new_buf
    } else {
        buf
    }
}
