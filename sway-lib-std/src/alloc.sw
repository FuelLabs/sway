//! A library for allocating memory inspired by [Rust's std::alloc](https://doc.rust-lang.org/std/alloc/index.html).
library alloc;

/// Allocates zeroed memory on the heap.
///
/// In the FuelVM, the heap begins at `VM_MAX_RAM - 1` and grows downward.
/// The heap pointer, `$hp`, will always point to unallocated space.
///
/// Initially the heap will look like this:
/// ```
/// ... 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |
///                                               $hp^  ^VM_MAX_RAM
/// ```
/// After allocating with `let ptr = alloc::<u64>(1)`:
/// ```
/// ... 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |
///                       $hp^  ^ptr                    ^VM_MAX_RAM
/// ```
/// After writing with `sw(ptr, u64::max())`:
/// ```
/// ... 00 00 00 00 00 00 00 00 FF FF FF FF FF FF FF FF |
///                       $hp^  ^ptr                    ^VM_MAX_RAM
/// ```
/// For more information, see the Fuel Spec for [VM Initialization](https://fuellabs.github.io/fuel-specs/master/vm#vm-initialization)
/// and the VM Instruction Set for [Memory Allocation](https://fuellabs.github.io/fuel-specs/master/vm/instruction_set.html#aloc-allocate-memory).
pub fn alloc<T>(count: u64) -> raw_ptr {
    asm(size: __size_of::<T>() * count, ptr) {
        aloc size;
        // `$hp` points to unallocated space and heap grows downward so
        // our newly allocated space will be right after it.
        addi ptr hp i1;
        ptr: raw_ptr
    }
}

/// Reallocates the given area of memory.
pub fn realloc<T>(ptr: raw_ptr, count: u64, new_count: u64) -> raw_ptr {
    if new_count > count {
        let new_ptr = alloc::<T>(new_count);
        if count > 0 {
            ptr.copy_to::<T>(new_ptr, count);
        }
        new_ptr
    } else {
        ptr
    }
}

/// Allocates zeroed memory on the heap in individual bytes.
pub fn alloc_bytes(count: u64) -> raw_ptr {
    asm(size: count, ptr) {
        aloc size;
        addi ptr hp i1;
        ptr: raw_ptr
    }
}

/// Reallocates the given area of memory in individual bytes.
pub fn realloc_bytes(ptr: raw_ptr, count: u64, new_count: u64) -> raw_ptr {
    if new_count > count {
        let new_ptr = alloc_bytes(new_count);
        if count > 0 {
            ptr.copy_bytes_to(new_ptr, count);
        }
        new_ptr
    } else {
        ptr
    }
}
