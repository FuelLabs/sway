//! This module provides:
//! - a `StorableSlice<T>` trait for storing and loading types in storage, whose content can be represented as a slice of bytes.
//! - helper functions for storing and loading slices of bytes in storage.
//!
//! The helper functions with prefix `quads`, e.g., `write_slice_quads` or `read_slice_quads`,
//! store the slice content in contiguous storage slots of 32 bytes, starting at the storage slot
//! determined by the `sha256` hash of the provided `slot`.
//! The `slot` itself is used to store the length of the slice.
//!
//! The helper functions with prefix `slot`, e.g., `write_slice_slot` or `read_slice_slot`,
//! store the slice content in a single dynamic storage slot of variable size, at the provided `slot`.
library;

use ::alloc::{alloc, alloc_bytes, realloc_bytes};
use ::hash::*;
use ::option::Option::{self, *};
use ::storage::storage_api::*;
use ::codec::*;
use ::debug::*;

/// A trait for storing types in storage, whose content can be represented as a slice of bytes.
///
/// Note that although a type `T` can have a semantic of being "empty" (e.g., an empty `String`),
/// its `StorableSlice` implementation (e.g., `StorageString`) cannot be empty.
/// In other words, if the `write_slice` method of a `StorableSlice<T>` implementation is called
/// with an argument of type `T` that is semantically empty, the `read_slice` method of the
/// `StorableSlice<T>` implementation will return `None` and not a `Some(empty_value)`,
/// where `empty_value` is the semantically empty `T`.
#[cfg(experimental_dynamic_storage = false)]
pub trait StorableSlice<T> {
    #[storage(read, write)]
    fn write_slice(self, argument: T);
    #[storage(read)]
    fn read_slice(self) -> Option<T>;
    #[storage(read, write)]
    fn clear(self) -> bool;
    /// The length of the slice in storage, in bytes,
    /// or `0` if `read_slice` would return `None`.
    #[storage(read)]
    fn len(self) -> u64;
}

/// A trait for storing types in storage, whose content can be represented as a slice of bytes.
///
/// Note that although a type `T` can have a semantic of being "empty" (e.g., an empty `String`),
/// its `StorableSlice` implementation (e.g., `StorageString`) cannot be empty.
/// In other words, if the `write_slice` method of a `StorableSlice<T>` implementation is called
/// with an argument of type `T` that is semantically empty, the `read_slice` method of the
/// `StorableSlice<T>` implementation will return `None` and not a `Some(empty_value)`,
/// where `empty_value` is the semantically empty `T`.
#[cfg(experimental_dynamic_storage = true)]
pub trait StorableSlice<T> {
    #[storage(write)]
    fn write_slice(self, argument: T);
    #[storage(read)]
    fn read_slice(self) -> Option<T>;
    #[storage(read, write)]
    fn clear(self);
    #[storage(read, write)]
    fn clear_existed(self) -> bool;
    /// The length of the slice in storage, in bytes,
    /// or `0` if `read_slice` would return `None`.
    #[storage(read)]
    fn len(self) -> u64;
}

/// Stores `slice` into storage, in slots of 32 bytes, starting at the `slot`.
///
/// # Additional Information
///
/// If `slice` is empty (i.e., has a length of zero), storage access will still occur,
/// but no content slots will be written to. The length of the slice in the storage will be set
/// to zero, but eventual existing content of the slice in storage will not be cleared.
///
/// # Arguments
///
/// * `slot`: [b256] - The starting storage slot at which `slice` will be stored.
/// * `slice`: [raw_slice] - The `raw_slice` to be stored.
///
/// # Number of Storage Accesses
///
/// * Writes: `2` (one for the length of the slice, and one for the slice content)
///
/// # Examples
///
/// ```sway
/// use std::{alloc::alloc_bytes, storage::{write_slice_quads, read_slice_quads}};
///
/// fn foo() {
///     let slice = asm(ptr: (alloc_bytes(1), 1)) { ptr: raw_slice };
///     assert(read_slice_quads(b256::zero()).is_none());
///     write_slice_quads(b256::zero(), slice);
///     let stored_slice = read_slice_quads(b256::zero()).unwrap();
///     assert_eq(slice, stored_slice);
/// }
/// ```
#[storage(read, write)]
pub fn write_slice_quads(slot: b256, slice: raw_slice) {
    // Get the number of storage slots needed based on the size of bytes.
    let number_of_bytes = slice.number_of_bytes();
    let number_of_slots = (number_of_bytes + 31) >> 5;
    let mut ptr = slice.ptr();

    // The capacity needs to be a multiple of 32 bytes so we can
    // make the 'quad' storage instruction store without accessing unallocated heap memory.
    ptr = realloc_bytes(ptr, number_of_bytes, number_of_slots * 32);

    // Store `number_of_slots * 32` bytes starting at `sha256(slot)`.
    let _ = __state_store_quad(sha256(slot), ptr, number_of_slots);

    // Store the length of the bytes at `slot`.
    write_quads::<u64>(slot, 0, number_of_bytes);
}

/// Stores `slice` into storage, in slots of 32 bytes, starting at the `slot`.
///
/// # Deprecation Notice
///
/// This function is deprecated in favor of `write_slice_quads` and `write_slice_slot`.
/// To preserve exactly the same behavior as `write`, use `write_slice_quads`. To store the `slice` into
/// a single dynamic slot of a variable size, use `write_slice_slot`.
///
/// # Additional Information
///
/// If `slice` is empty (i.e., has a length of zero), storage access will still occur,
/// but no content slots will be written to. The length of the slice in the storage will be set
/// to zero, but eventual existing content of the slice in storage will not be cleared.
///
/// # Arguments
///
/// * `slot`: [b256] - The starting storage slot at which `slice` will be stored.
/// * `slice`: [raw_slice] - The `raw_slice` to be stored.
///
/// # Number of Storage Accesses
///
/// * Writes: `2` (one for the length of the slice, and one for the slice content)
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
///     assert_eq(slice, stored_slice);
/// }
/// ```
#[deprecated(note = "Use `write_slice_quads` or `write_slice_slot` instead.")]
#[storage(read, write)]
pub fn write_slice(slot: b256, slice: raw_slice) {
    write_slice_quads(slot, slice);
}

/// Stores `slice` into storage, in a single dynamic storage `slot`.
///
/// # Additional Information
///
/// If `slice` is empty (i.e., has a length of zero), storage access will still occur.
/// The `slot` will be marked as occupied and storing a content of length zero.
/// Any eventual existing content of the slice in storage will be deleted.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot at which `slice` will be stored.
/// * `slice`: [raw_slice] - The `raw_slice` to be stored.
///
/// # Number of Storage Accesses
///
/// * Writes: `1` (for storing the slice content)
///
/// # Examples
///
/// ```sway
/// use std::{alloc::alloc_bytes, storage::{write_slice_slot, read_slice_slot}};
///
/// fn foo() {
///     let slice = asm(ptr: (alloc_bytes(1), 1)) { ptr: raw_slice };
///     assert(read_slice_slot(b256::zero()).is_none());
///     write_slice_slot(b256::zero(), slice);
///     let stored_slice = read_slice_slot(b256::zero()).unwrap();
///     assert_eq(slice, stored_slice);
/// }
/// ```
#[storage(write)]
pub fn write_slice_slot(slot: b256, slice: raw_slice) {
    __state_store_slot(slot, slice.ptr(), slice.number_of_bytes());
}

/// Loads a `raw_slice` from storage, stored in slots of 32 bytes, starting at the `slot`.
///
/// # Additional Information
///
/// Loading does not distinguish between a slot that has never been written to
/// and a slot that contains a slice of length zero. In both cases, `None` is returned.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot to begin loading a slice from.
///
/// # Returns
///
/// - [Option<raw_slice>] - If no value was previously stored at `slot`, or the stored slice was empty, `None` is returned. Otherwise,
/// `Some(value)` is returned, where `value` is the `raw_slice` stored at `slot`.
///
/// # Number of Storage Accesses
///
/// * Reads: `2` (one for the length of the slice, and one for the slice content)
///
/// # Examples
///
/// ```sway
/// use std::{alloc::alloc_bytes, storage::{write_slice_quads, read_slice_quads}};
///
/// fn foo {
///     let slice = asm(ptr: (alloc_bytes(1), 1)) { ptr: raw_slice };
///     assert(read_slice_quads(b256::zero()).is_none());
///     write_slice_quads(b256::zero(), slice);
///     let stored_slice = read_slice_quads(b256::zero()).unwrap();
///     assert_eq(slice, stored_slice);
/// }
/// ```
#[storage(read)]
pub fn read_slice_quads(slot: b256) -> Option<raw_slice> {
    // Get the length of the stored slice.
    match read_quads::<u64>(slot, 0).unwrap_or(0) {
        0 => None,
        len => {
            // Get the number of storage slots needed based on the size.
            let number_of_slots = (len + 31) >> 5;
            let ptr = alloc_bytes(number_of_slots * 32);
            // Load the slice content of `number_of_slots * 32` bytes starting at `sha256(slot)`.
            let _ = __state_load_quad(sha256(slot), ptr, number_of_slots);
            Some(asm(ptr: (ptr, len)) {
                ptr: raw_slice
            })
        }
    }
}

/// Loads a `raw_slice` from storage, stored in slots of 32 bytes, starting at the `slot`.
///
/// # Deprecation Notice
///
/// This function is deprecated in favor of `read_slice_quads` and `read_slice_slot`.
/// To preserve exactly the same behavior as `read`, use `read_slice_quads`. To read the `slice` from
/// a single dynamic slot of a variable size, use `read_slice_slot`.
///
/// # Additional Information
///
/// Loading does not distinguish between a slot that has never been written to
/// and a slot that contains a slice of length zero. In both cases, `None` is returned.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot to begin loading a slice from.
///
/// # Returns
///
/// - [Option<raw_slice>] - If no value was previously stored at `slot`, or the stored slice was empty, `None` is returned. Otherwise,
/// `Some(value)` is returned, where `value` is the `raw_slice` stored at `slot`.
///
/// # Number of Storage Accesses
///
/// * Reads: `2` (one for the length of the slice, and one for the slice content)
///
/// # Examples
///
/// ```sway
/// use std::{alloc::alloc_bytes, storage::{write_slice_quads, read_slice_quads}};
///
/// fn foo {
///     let slice = asm(ptr: (alloc_bytes(1), 1)) { ptr: raw_slice };
///     assert(read_slice_quads(b256::zero()).is_none());
///     write_slice_quads(b256::zero(), slice);
///     let stored_slice = read_slice_quads(b256::zero()).unwrap();
///     assert_eq(slice, stored_slice);
/// }
/// ```
#[deprecated(note = "Use `read_slice_quads` or `read_slice_slot` instead.")]
#[storage(read)]
pub fn read_slice(slot: b256) -> Option<raw_slice> {
    read_slice_quads(slot)
}

/// Loads a `raw_slice` from storage, stored in a single dynamic `slot`.
///
/// # Additional Information
///
/// Loading does not distinguish between a slot that has never been written to
/// and a slot that contains a slice of length zero. In both cases, `None` is returned.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot to load a slice from.
///
/// # Returns
///
/// - [Option<raw_slice>] - If no value was previously stored at `slot`, or the stored slice was empty, `None` is returned. Otherwise,
/// `Some(value)` is returned, where `value` is the `raw_slice` stored at `slot`.
///
/// # Number of Storage Accesses
///
/// * Preloads: `1` (for the length of the slice)
/// * Reads: `1` (for the slice content)
///
/// # Examples
///
/// ```sway
/// use std::{alloc::alloc_bytes, storage::{write_slice_slot, read_slice_slot}};
///
/// fn foo {
///     let slice = asm(ptr: (alloc_bytes(1), 1)) { ptr: raw_slice };
///     assert(read_slice_slot(b256::zero()).is_none());
///     write_slice_slot(b256::zero(), slice);
///     let stored_slice = read_slice_slot(b256::zero()).unwrap();
///     assert_eq(slice, stored_slice);
/// }
/// ```
#[storage(read)]
pub fn read_slice_slot(slot: b256) -> Option<raw_slice> {
    // Get the length of the stored slice.
    match __state_preload(slot) {
        0 => None,
        len => {
            let ptr = alloc_bytes(len);
            let _ = __state_load_slot(slot, ptr, 0, len);
            Some(asm(ptr: (ptr, len)) {
                ptr: raw_slice
            })
        }
    }
}

/// Clears a slice, stored in slots of 32 bytes, from storage, starting at the `slot`.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot to begin clearing the slice from.
///
/// # Returns
///
/// * [bool] - `true` if _all_ of the cleared storage slots were previously set. Otherwise, `false`.
///
/// # Number of Storage Accesses
///
/// * Reads: `1` (to determine the length of the slice)
/// * Clears: `2` (one for the length of the slice, and one for the slice content)
///
/// # Examples
///
/// ```sway
/// use std::{alloc::alloc_bytes, storage::{clear_slice_quads, write_slice_quads, read_slice_quads}};
///
/// fn foo() {
///     let slice = asm(ptr: (alloc_bytes(1), 1)) { ptr: raw_slice };
///     write_slice_quads(b256::zero(), slice);
///     assert(read_slice_quads(b256::zero()).is_some());
///     let cleared = clear_slice_quads(b256::zero());
///     assert(cleared);
///     assert(read_slice_quads(b256::zero()).is_none());
/// }
/// ```
#[storage(read, write)]
pub fn clear_slice_quads(slot: b256) -> bool {
    // Get the number of storage slots needed based on the ceiling of `len / 32`.
    let len = read_quads::<u64>(slot, 0).unwrap_or(0);
    let number_of_slots = (len + 31) >> 5;

    // Clear length and `number_of_slots` content slots starting at storage slot `sha256(slot)`.
    let _ = __state_clear(slot, 1);
    __state_clear(sha256(slot), number_of_slots)
}

/// Clears a slice, stored in slots of 32 bytes, from storage, starting at the `slot`.
///
/// # Deprecation Notice
///
/// This function is deprecated in favor of `clear_slice_quads`, `clear_slice_slot`, and `clear_slice_slot_existed`.
/// To preserve exactly the same behavior as `clear_slice`, use `clear_slice_quads`.
/// To clear a slice contained in a single dynamic slot of variable sizes, use `clear_slice_slot`.
/// To clear a slice contained in a single dynamic slot of variable sizes, and obtain information
/// about whether the cleared slot was previously set, use `clear_slice_slot_existed`.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot to begin clearing the slice from.
///
/// # Returns
///
/// * [bool] - `true` if _all_ of the cleared storage slots were previously set. Otherwise, `false`.
///
/// # Number of Storage Accesses
///
/// * Reads: `1` (to determine the length of the slice)
/// * Clears: `2` (one for the length of the slice, and one for the slice content)
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
#[deprecated(note = "Use `clear_slice_quads`, `clear_slice_slot`, or `clear_slice_slot_existed` instead.")]
#[storage(read, write)]
pub fn clear_slice(slot: b256) -> bool {
    clear_slice_quads(slot)
}

/// Clears a slice from storage, stored in a single dynamic `slot`.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot to clear the slice from.
///
/// # Number of Storage Accesses
///
/// * Clears: `1` (for the slice content)
///
/// # Examples
///
/// ```sway
/// use std::{alloc::alloc_bytes, storage::{clear_slice_slot, write_slice_slot, read_slice_slot}};
///
/// fn foo() {
///     let slice = asm(ptr: (alloc_bytes(1), 1)) { ptr: raw_slice };
///     write_slice_slot(b256::zero(), slice);
///     assert(read_slice_slot(b256::zero()).is_some());
///     clear_slice_slot(b256::zero());
///     assert(read_slice_slot(b256::zero()).is_none());
/// }
/// ```
#[storage(write)]
pub fn clear_slice_slot(slot: b256) {
    __state_clear_slots(slot, 1);
}

/// Clears a slice from storage, stored in a single dynamic `slot`, and returns whether the cleared slot was previously set.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot to clear the slice from.
///
/// # Returns
///
/// * [bool] - `true` if the cleared storage slot was previously set. Otherwise, `false`.
///
/// # Number of Storage Accesses
///
/// * Preload: `1` (to determine if the `slot` was previously set)
/// * Clears: `1` (for the slice content)
///
/// # Examples
///
/// ```sway
/// use std::{alloc::alloc_bytes, storage::{clear_slice_slot_existed, write_slice_slot, read_slice_slot}};
///
/// fn foo() {
///     let slice = asm(ptr: (alloc_bytes(1), 1)) { ptr: raw_slice };
///     write_slice_slot(b256::zero(), slice);
///     assert(read_slice_slot(b256::zero()).is_some());
///     let cleared = clear_slice_slot_existed(b256::zero());
///     assert(cleared);
///     assert(read_slice_slot(b256::zero()).is_none());
/// }
/// ```
#[storage(read, write)]
pub fn clear_slice_slot_existed(slot: b256) -> bool {
    let existed = __state_preload(slot) != 0;
    __state_clear_slots(slot, 1);
    existed
}
