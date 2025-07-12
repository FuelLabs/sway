library;

use ::alloc::{alloc, alloc_bytes, realloc_bytes};
use ::hash::*;
use ::option::Option::{self, *};
use ::storage::storage_api::*;
use ::codec::*;
use ::debug::*;

/// Store a raw_slice from the heap into storage.
///
/// # Arguments
///
/// * `key`: [b256] - The storage slot at which the variable will be stored.
/// * `slice`: [raw_slice] - The raw_slice to be stored.
///
/// # Number of Storage Accesses
///
/// * Writes: `2`
///
/// # Examples
///
/// ```sway
/// use std::{alloc::alloc_bytes, storage::{write_slice, read_slice}};
///
/// fn foo() {
///     let slice = asm(ptr: (alloc_bytes(1), 1)) { ptr: raw_slice };
///     assert(read_slice(b256::zero()).is_none());
///     write_slice(b256::zero(), slice);
///     let stored_slice = read_slice(b256::zero()).unwrap();
///     assert(slice == stored_slice);
/// }
/// ```
#[storage(read, write)]
pub fn write_slice(key: b256, slice: raw_slice) {
    // Get the number of storage slots needed based on the size of bytes.
    let number_of_bytes = slice.number_of_bytes();
    let number_of_slots = (number_of_bytes + 31) >> 5;
    let mut ptr = slice.ptr();

    // The capacity needs to be a multiple of 32 bytes so we can
    // make the 'quad' storage instruction store without accessing unallocated heap memory.
    ptr = realloc_bytes(ptr, number_of_bytes, number_of_slots * 32);

    // Store `number_of_slots * 32` bytes starting at `sha256(key)`.
    let _ = __state_store_quad(sha256(key), ptr, number_of_slots);

    // Store the length of the bytes at `key`.
    write(key, 0, number_of_bytes);
}

/// Load a raw_slice from storage.
///
/// # Arguments
///
/// * `key`: [b256] - The storage slot to load the value from.
///
/// # Returns
///
/// - [Option<raw_slice>] - If no value was previously stored at `key`, `None` is returned. Otherwise,
/// `Some(value)` is returned, where `value` is the value stored at `key`.
///
/// # Number of Storage Accesses
///
/// * Reads: `2`
///
/// # Examples
///
/// ```sway
/// use std::{alloc::alloc_bytes, storage::{write_slice, read_slice}};
///
/// fn foo {
///     let slice = asm(ptr: (alloc_bytes(1), 1)) { ptr: raw_slice };
///     assert(read_slice(b256::zero()).is_none());
///     write_slice(b256::zero(), slice);
///     let stored_slice = read_slice(b256::zero()).unwrap();
///     assert(slice == stored_slice);
/// }
/// ```
#[storage(read)]
pub fn read_slice(key: b256) -> Option<raw_slice> {
    // Get the length of the slice that is stored.
    match read::<u64>(key, 0).unwrap_or(0) {
        0 => None,
        len => {
            // Get the number of storage slots needed based on the size.
            let number_of_slots = (len + 31) >> 5;
            let ptr = alloc_bytes(number_of_slots * 32);
            // Load the stored slice into the pointer.
            let _ = __state_load_quad(sha256(key), ptr, number_of_slots);
            Some(asm(ptr: (ptr, len)) {
                ptr: raw_slice
            })
        }
    }
}

/// Clear a sequence of storage slots starting at a some key.
///
/// # Arguments
///
/// * `key`: [b256] - The key of the first storage slot that will be cleared
///
/// # Returns
///
/// * [bool] - Indicates whether all of the storage slots cleared were previously set.
///
/// # Number of Storage Accesses
///
/// * Reads: `1`
/// * Clears: `2`
///
/// # Examples
///
/// ```sway
/// use std::{alloc::alloc_bytes, storage::{clear_slice, write_slice, read_slice}};
///
/// fn foo() {
///     let slice = asm(ptr: (alloc_bytes(1), 1)) { ptr: raw_slice };
///     write_slice(b256::zero(), slice);
///     assert(read_slice(b256::zero()).is_some());
///     let cleared = clear_slice(b256::zero());
///     assert(cleared);
///     assert(read_slice(b256::zero()).is_none());
/// }
/// ```
#[storage(read, write)]
pub fn clear_slice(key: b256) -> bool {
    // Get the number of storage slots needed based on the ceiling of `len / 32`
    let len = read::<u64>(key, 0).unwrap_or(0);
    let number_of_slots = (len + 31) >> 5;

    // Clear length and `number_of_slots` bytes starting at storage slot `sha256(key)`
    let _ = __state_clear(key, 1);
    __state_clear(sha256(key), number_of_slots)
}

/// A general way to persistently store heap types.
pub trait StorableSlice<T> {
    #[storage(read, write)]
    fn write_slice(self, argument: T);
    #[storage(read)]
    fn read_slice(self) -> Option<T>;
    #[storage(read, write)]
    fn clear(self) -> bool;
    #[storage(read)]
    fn len(self) -> u64;
}
