//! A library for allocating memory inspired by [Rust's std::alloc](https://doc.rust-lang.org/std/alloc/index.html).
library;

use ::ops::*;
use ::raw_ptr::*;

/// Allocates zeroed memory on the heap.
///
/// # Additional Information
///
/// In the FuelVM, the heap begins at `VM_MAX_RAM` and grows downward.
/// The heap pointer(`$hp`) always points to the first allocated byte.
///
/// Initially the heap will look like this:
/// ```
///                                                     ▾$hp
/// ... 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |
///                                                     ▴VM_MAX_RAM
/// ```
/// After allocating with `let ptr = alloc::<u64>(1)`:
/// ```
///                             ▾$hp
/// ... 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 |
///                             ▴ptr                    ▴VM_MAX_RAM
/// ```
/// After writing with `sw(ptr, u64::max())`:
/// ```
///                             ▾$hp
/// ... 00 00 00 00 00 00 00 00 FF FF FF FF FF FF FF FF |
///                             ▴ptr                    ▴VM_MAX_RAM
/// ```
/// For more information, see the Fuel Spec for [VM Initialization](https://fuellabs.github.io/fuel-specs/master/vm#vm-initialization)
/// and the VM Instruction Set for [Memory Allocation](https://docs.fuel.network/docs/specs/fuel-vm/instruction-set#aloc-allocate-memory).
///
/// # Arguments
///
/// * `count`: [u64] - The number of `size_of<T>` bytes to allocate onto the heap.
///
/// # Returns
///
/// * [raw_ptr] - The pointer to the newly allocated memory.
///
/// # Examples
///
/// ```sway
/// use std::alloc::alloc;
///
/// fn foo() {
///     let ptr = alloc::<u64>(2);
///     assert(!ptr.is_null());
/// }
/// ```
pub fn alloc<T>(count: u64) -> raw_ptr {
    __alloc::<T>(count)
}

/// Reallocates the given area of memory.
///
/// # Arguments
///
/// * `ptr`: [raw_ptr] - The pointer to the area of memory to reallocate.
/// * `count`: [u64] - The number of `size_of<T>` bytes kept when reallocating. These are not set to 0.
/// * `new_count`: [u64] - The number of new `size_of<T>` bytes to allocate. These are set to 0.
///
/// # Returns
///
/// * [raw_ptr] - The pointer to the newly reallocated memory.
///
/// # Examples
///
/// ```sway
/// use std::alloc::{alloc, realloc};
///
/// fn foo() {
///     let ptr = alloc::<u64>(1);
///     ptr.write(5);
///     let reallocated_ptr = realloc::<u64>(ptr, 1, 2);
///     assert(reallocated_ptr.read::<u64>() == 5);
/// }
/// ```
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
///
/// # Arguments
///
/// * `count`: [u64] - The number of bytes to allocate onto the heap.
///
/// # Returns
///
/// * [raw_ptr] - The pointer to the newly allocated memory.
///
/// # Examples
///
/// ```sway
/// use std::alloc::alloc_bytes;
///
/// fn foo() {
///     let ptr = alloc_bytes(2);
///     assert(!ptr.is_null());
/// }
/// ```
pub fn alloc_bytes(count: u64) -> raw_ptr {
    __alloc::<u8>(count)
}

/// Reallocates the given area of memory in individual bytes.
///
/// # Arguments
///
/// * `ptr`: [raw_ptr] - The pointer to the area of memory to reallocate.
/// * `count`: [u64] - The number of bytes kept when reallocating. These are not set to 0.
/// * `new_count`: [u64] - The number of new bytes to allocate. These are set to 0.
///
/// # Returns
///
/// * [raw_ptr] - The pointer to the newly reallocated memory.
///
/// # Examples
///
/// ```sway
/// use std::alloc::{alloc_bytes, realloc_bytes};
///
/// fn foo() {
///     let ptr = alloc_bytes(8);
///     ptr.write(5);
///     let reallocated_ptr = realloc_bytes(ptr, 8, 16);
///     assert(reallocated_ptr.read::<u64>() == 5);
/// }
/// ```
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
