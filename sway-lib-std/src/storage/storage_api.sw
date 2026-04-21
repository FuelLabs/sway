library;

use ::alloc::alloc_bytes;
use ::option::Option::{self, *};
use ::ops::*;
use ::primitive_conversions::{b256::*, u256::*, u64::*};

/// Stores `value` in storage, in slots of 32 bytes, starting at `slot` and `offset` given in words.
///
/// # Additional Information
///
/// The `value` can be stored in the `slot` or the following slots depending on the `offset` and size of `value`.
/// If the `value` crosses the boundary of a storage slot, writing continues at the following slot.
///
/// The `offset` is given in words and can be outside of the `slot` boundary. For example, offset `4` means
/// the beginning of the next slot, offset `5` means the second word of the next slot, and so on.
///
/// If `T` is a zero-sized type, no storage access will occur. Storage API does not store zero-sized types in storage,
/// so reading from the slot and offset where a zero-sized type would be stored will return `None`.
///
/// **The `value` is memory-copied into the storage slots. If it contains any pointers or references,
/// the data they point to will not be stored in storage.**
///
/// To store dynamic types like `Vec`, `String`, or `Bytes`, use the dedicated storage types provided in the `storage` module,
/// like `StorageVec`, `StorageString`, and `StorageBytes`.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot from which to count the `offset`. The value can be stored in this or the following slots.
/// * `offset`: [u64] - An offset, *in words*, starting at the beginning of `slot` at which `value` should be stored.
/// * `value`: [T] - The value to be stored.
///
/// # Number of Storage Accesses
///
/// * Reads: `0` if the `value` occupies full slots, `1` otherwise (to read the existing data that will be partially overwritten)
/// * Writes: `1`
///
/// # Reverts
///
/// * When the currently existing storage slots being read before writing the `value` have size different than 32 bytes.
///
/// # Examples
///
/// ```sway
/// use std::storage::storage_api::{read_quads, write_quads};
///
/// fn foo() {
///     let five = 5_u64;
///     write_quads(b256::zero(), 2, five);
///     let stored_five = read_quads::<u64>(b256::zero(), 2).unwrap();
///     assert_eq(five, stored_five);
/// }
/// ```
#[storage(read, write)]
pub fn write_quads<T>(slot: b256, offset: u64, value: T) {
    if __size_of::<T>() == 0 {
        return;
    }

    if __size_of::<T>() % 32 == 0 && offset == 0 {
        // If the value is aligned to the start of a slot and occupies full slots, we can store it directly.
        let value_addr = __addr_of::<T>(value);
        let _ = __state_store_quad(slot, value_addr, __size_of::<T>() / 32);
        return;
    }

    // Determine how many slots and where the value is to be stored.
    let (offset_slot, number_of_slots, place_in_slot) = slot_calculator::<T>(slot, offset);

    // Allocate enough memory on the heap for `value` as well as any potential padding required due
    // to `offset`.
    let padded_value = alloc_bytes(number_of_slots * 32);

    // Read the values that currently exist in the affected storage slots.
    let _ = __state_load_quad(offset_slot, padded_value, number_of_slots);

    // Copy the value to be stored to `padded_value + offset`.
    padded_value.add::<u64>(place_in_slot).write::<T>(value);

    // Now store back the data at `padded_value` which now contains the old data but partially
    // overwritten by the new data in the desired locations.
    let _ = __state_store_quad(offset_slot, padded_value, number_of_slots);
}

/// Stores `value` in storage, in slots of 32 bytes, starting at `slot` and `offset` given in words.
///
/// # Deprecation Notice
///
/// This function is deprecated in favor of `write_quads`, `write_slot`, and `update_slot`.
/// To preserve exactly the same behavior as `write`, use `write_quads`. To store the `value` into
/// a single dynamic slot of a variable size, use `write_slot`. To update a portion of a dynamic slot,
/// or append to it, use `update_slot`.
///
/// # Additional Information
///
/// The `value` can be stored in the `slot` or the following slots depending on the `offset` and size of `value`.
/// If the `value` crosses the boundary of a storage slot, writing continues at the following slot.
///
/// The `offset` is given in words and can be outside of the `slot` boundary. For example, offset `4` means
/// the beginning of the next slot, offset `5` means the second word of the next slot, and so on.
///
/// If `T` is a zero-sized type, no storage access will occur. Storage API does not store zero-sized types in storage,
/// so reading from the slot and offset where a zero-sized type would be stored will return `None`.
///
/// **The `value` is memory-copied into the storage slots. If it contains any pointers or references,
/// the data they point to will not be stored in storage.**
///
/// To store dynamic types like `Vec`, `String`, or `Bytes`, use the dedicated storage types provided in the `storage` module,
/// like `StorageVec`, `StorageString`, and `StorageBytes`.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot from which to count the `offset`. The value can be stored in this or the following slots.
/// * `offset`: [u64] - An offset, *in words*, starting at the beginning of `slot` at which `value` should be stored.
/// * `value`: [T] - The value to be stored.
///
/// # Number of Storage Accesses
///
/// * Reads: `0` if the `value` occupies full slots, `1` otherwise (to read the existing data that will be partially overwritten)
/// * Writes: `1`
///
/// # Reverts
///
/// * When the currently existing storage slots being read before writing the `value` have size different than 32 bytes.
///
/// # Examples
///
/// ```sway
/// use std::storage::storage_api::{read_quads, write_quads};
///
/// fn foo() {
///     let five = 5_u64;
///     write_quads(b256::zero(), 2, five);
///     let stored_five = read_quads::<u64>(b256::zero(), 2).unwrap();
///     assert_eq(five, stored_five);
/// }
/// ```
#[deprecated(note = "Use `write_quads`, `write_slot`, or `update_slot` instead.")]
#[storage(read, write)]
pub fn write<T>(slot: b256, offset: u64, value: T) {
    write_quads(slot, offset, value);
}

/// Stores a `value` in storage in a single dynamic `slot`.
///
/// # Additional Information
///
/// The `value` is entirely stored in the `slot` and never crosses into another slot.
///
/// If `T` is a zero-sized type, no storage access will occur. Storage API does not store zero-sized types in storage,
/// so reading from the slot and offset where a zero-sized type would be stored will return `None`.
///
/// **The `value` is memory-copied into the storage slot. If it contains any pointers or references,
/// the data they point to will not be stored in storage.**
///
/// To store dynamic types like `Vec`, `String`, or `Bytes`, use the dedicated storage types provided in the `storage` module,
/// like `StorageVec`, `StorageString`, and `StorageBytes`.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot at which the `value` will be stored.
/// * `value`: [T] - The value to be stored.
///
/// # Number of Storage Accesses
///
/// * Writes: `1`
///
/// # Examples
///
/// ```sway
/// use std::storage::storage_api::{read_slot, write_slot};
///
/// fn foo() {
///     let five = 5_u64;
///     write_slot(b256::zero(), five);
///     let stored_five = read_slot::<u64>(b256::zero(), 0).unwrap();
///     assert_eq(five, stored_five);
/// }
/// ```
#[storage(write)]
pub fn write_slot<T>(slot: b256, value: T) {
    if __size_of::<T>() == 0 {
        return;
    }

    __state_store_slot(slot, __addr_of::<T>(value), __size_of::<T>());
}

/// Updates a `value` in storage in a single dynamic `slot`, placing it at the `offset` given in bytes.
///
/// # Additional Information
///
/// The `value` is entirely stored in the `slot` and never crosses into another slot.
/// The `offset`, given in bytes, only determines where in the `slot` the `value` is stored.
///
/// If the slot already has data stored in it, the `value` will be written on top of the existing data starting at the `offset`,
/// overwriting the existing data at the `offset`. If the `value` does not fit in the remaining space in the slot after the `offset`,
/// the slot will be expanded to accommodate the entire `value`.
///
/// The `offset` must be a valid existing offset in the `slot` or `u64::max()`.
/// `u64::max()` is used to store at the end of the currently used portion of the slot, i.e., to append to the slot.
/// Valid existing offsets are from `0` to the size of the currently used portion of the slot in bytes.
/// For example, if the slot currently has 10 bytes used, valid offsets are from `0` to `10` and `u64::max()`.
/// Offsets `0` to `9` are used to store within the currently used portion of the slot.
/// Offsets `10` and `u64::max()` will store starting right after the currently used portion of the slot.
///
/// An offset greater than the currently used portion of the slot but less than `u64::max()` is invalid and will cause a revert.
///
/// To append to the slot, instead of using `update_slot` with `u64::max()`, the more idiomatic way is to use the `append_slot` function.
///
/// If `T` is a zero-sized type, no storage access will occur. Storage API does not store zero-sized types in storage,
/// so reading from the slot and offset where a zero-sized type would be stored will return `None`.
///
/// **The `value` is memory-copied into the storage slot. If it contains any pointers or references,
/// the data they point to will not be stored in storage.**
///
/// To store dynamic types like `Vec`, `String`, or `Bytes`, use the dedicated storage types provided in the `storage` module,
/// like `StorageVec`, `StorageString`, and `StorageBytes`.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot at which the `value` will be stored.
/// * `offset`: [u64] - An offset, *in bytes*, starting at the beginning of `slot` at which `value` should be stored.
/// * `value`: [T] - The value to be stored.
///
/// # Number of Storage Accesses
///
/// * Internal preloads: `1`
/// * Writes: `1`
///
/// # Reverts
///
/// * If the `offset` is greater than the currently used portion of the slot but less than `u64::max()`.
///
/// # Examples
///
/// ```sway
/// use std::storage::storage_api::{read_slot, update_slot, write_slot};
///
/// fn foo() {
///     let five = 5_u64;
///     write_slot(b256::zero(), five);
///     update_slot(b256::zero(), 0, five + 1);
///     update_slot(b256::zero(), 1, five + 2); // Append 7.
///     update_slot(b256::zero(), u64::max(), five + 3); // Append 8.
///     let stored_six = read_slot::<u64>(b256::zero(), 0).unwrap();
///     assert_eq(five + 1, stored_six);
///     let stored_seven = read_slot::<u64>(b256::zero(), 1).unwrap();
///     assert_eq(five + 2, stored_seven);
///     let stored_eight = read_slot::<u64>(b256::zero(), 2).unwrap();
///     assert_eq(five + 3, stored_eight);
/// }
/// ```
#[storage(write)]
pub fn update_slot<T>(slot: b256, offset: u64, value: T) {
    if __size_of::<T>() == 0 {
        return;
    }

    __state_update_slot(slot, __addr_of::<T>(value), offset, __size_of::<T>());
}

/// Appends a `value` to the end of the currently used portion of a single dynamic `slot`.
///
/// # Additional Information
///
/// The `value` is stored at the end of the currently used portion of the `slot` and never crosses into another slot.
/// This is equivalent to calling `update_slot` with `u64::max()` as the `offset`.
///
/// If `T` is a zero-sized type, no storage access will occur. Storage API does not store zero-sized types in storage,
/// so reading from the slot and offset where a zero-sized type would be stored will return `None`.
///
/// **The `value` is memory-copied into the storage slot. If it contains any pointers or references,
/// the data they point to will not be stored in storage.**
///
/// To store dynamic types like `Vec`, `String`, or `Bytes`, use the dedicated storage types provided in the `storage` module,
/// like `StorageVec`, `StorageString`, and `StorageBytes`.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot to which the `value` will be appended.
/// * `value`: [T] - The value to be appended.
///
/// # Number of Storage Accesses
///
/// * Preloads: `1`
/// * Writes: `1`
///
/// # Examples
///
/// ```sway
/// use std::storage::storage_api::{read_slot, write_slot, append_slot};
///
/// fn foo() {
///     let five = 5_u64;
///     write_slot(b256::zero(), five);
///     append_slot(b256::zero(), five + 1);
///     append_slot(b256::zero(), five + 2);
///     let stored_five = read_slot::<u64>(b256::zero(), 0).unwrap();
///     assert_eq(five, stored_five);
///     let stored_six = read_slot::<u64>(b256::zero(), 1).unwrap();
///     assert_eq(five + 1, stored_six);
///     let stored_seven = read_slot::<u64>(b256::zero(), 2).unwrap();
///     assert_eq(five + 2, stored_seven);
/// }
/// ```
#[storage(write)]
pub fn append_slot<T>(slot: b256, value: T) {
    update_slot(slot, u64::max(), value);
}

/// Reads a value of type `T` from slots of 32 bytes each, starting at the location specified by `slot` and `offset` given in words.
///
/// # Additional Information
///
/// If the stored value crosses the boundary of a 32-byte-long storage slot, reading continues at the following slot.
///
/// If `T` is a zero-sized type, no storage access will occur. Storage API does not store zero-sized types in storage,
/// so reading from the slot and offset where a zero-sized type would be stored will return `None`.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot from which to count the `offset`. The value can be read from this or the following slots.
/// * `offset`: [u64] - An offset, *in words*, from the start of `slot`, from which the value should be read.
///
/// # Returns
///
/// * [Option<T>] - `Option(value)` if the storage slots read were valid and contain `value`. Otherwise, `None`.
///
/// # Number of Storage Accesses
///
/// * Reads: `1`
///
/// # Examples
///
/// ```sway
/// use std::storage::storage_api::{read_quads, write_quads};
///
/// fn foo() {
///     let five = 5_u64;
///     write_quads(b256::zero(), 2, five);
///     let stored_five = read_quads::<u64>(b256::zero(), 2).unwrap();
///     assert_eq(five, stored_five);
/// }
/// ```
#[storage(read)]
pub fn read_quads<T>(slot: b256, offset: u64) -> Option<T> {
    if __size_of::<T>() == 0 {
        return None;
    }

    // Determine how many slots and where the value is to be read.
    let (offset_slot, number_of_slots, place_in_slot) = slot_calculator::<T>(slot, offset);

    // Allocate a buffer for the result. Its size needs to be a multiple of 32 bytes so we can
    // make the 'quad' storage instruction read without overflowing.
    let result_ptr = alloc_bytes(number_of_slots * 32);

    // Read `number_of_slots * 32` bytes starting at storage slot `slot` and return an `Option`
    // wrapping the value stored at `result_ptr + offset` if all the slots are valid. Otherwise,
    // return `None`.
    if __state_load_quad(offset_slot, result_ptr, number_of_slots)
    {
        Some(result_ptr.add::<u64>(place_in_slot).read::<T>())
    } else {
        None
    }
}

/// Reads a value of type `T` from slots of 32 bytes each, starting at the location specified by `slot` and `offset` given in words.
///
/// # Deprecation Notice
///
/// This function is deprecated in favor of `read_quads` and `read_slot`.
/// To preserve exactly the same behavior as `read`, use `read_quads`. To read a value from
/// a single dynamic slot of a variable size, use `read_slot`.
///
/// # Additional Information
///
/// If the stored value crosses the boundary of a 32-byte-long storage slot, reading continues at the following slot.
///
/// If `T` is a zero-sized type, no storage access will occur. Storage API does not store zero-sized types in storage,
/// so reading from the slot and offset where a zero-sized type would be stored will return `None`.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot from which to count the `offset`. The value can be read from this or the following slots.
/// * `offset`: [u64] - An offset, *in words*, from the start of `slot`, from which the value should be read.
///
/// # Returns
///
/// * [Option<T>] - `Option(value)` if the storage slots read were valid and contain `value`. Otherwise, `None`.
///
/// # Number of Storage Accesses
///
/// * Reads: `1`
///
/// # Examples
///
/// ```sway
/// use std::storage::storage_api::{read, write};
///
/// fn foo() {
///     let five = 5_u64;
///     write(b256::zero(), 2, five);
///     let stored_five = read::<u64>(b256::zero(), 2).unwrap();
///     assert_eq(five, stored_five);
/// }
/// ```
#[deprecated(note = "Use `read_quads` or `read_slot` instead.")]
#[storage(read)]
pub fn read<T>(slot: b256, offset: u64) -> Option<T> {
    read_quads(slot, offset)
}

/// Reads a value of type `T` from a single dynamic `slot`, starting at the `offset` given in bytes.
///
/// # Additional Information
///
/// If `T` is a zero-sized type, no storage access will occur. Storage API does not store zero-sized types in storage,
/// so reading from the slot and offset where a zero-sized type would be stored will return `None`.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot from which to read a value.
/// * `offset`: [u64] - An offset, *in bytes*, from the start of `slot`, from which the value should be read.
///
/// # Returns
///
/// * [Option<T>] - `Option(value)` if the storage slot read was valid and contain `value`. Otherwise, `None`.
///
/// # Number of Storage Accesses
///
/// * Reads: `1`
///
/// # Reverts
///
/// * When the `offset` is out of bounds of the currently used portion of the slot, if the slot is not empty.
/// * When the storage slot is not large enough to contain a value of size of `T` at the given `offset`.
///
/// # Examples
///
/// ```sway
/// use std::storage::storage_api::{read_slot, append_slot};
///
/// fn foo() {
///     let five = 5_u64;
///     append_slot(b256::zero(), five);
///     append_slot(b256::zero(), five + 1);
///     let stored_five = read_slot::<u64>(b256::zero(), 0).unwrap();
///     assert_eq(five, stored_five);
///     let stored_six = read_slot::<u64>(b256::zero(), 1 * 8).unwrap();
///     assert_eq(five + 1, stored_six);
/// }
/// ```
#[storage(read)]
pub fn read_slot<T>(slot: b256, offset: u64) -> Option<T> {
    if __size_of::<T>() == 0 {
        return None;
    }

    let result_ptr = alloc_bytes(__size_of::<T>());

    if __state_load_slot(slot, result_ptr, offset, __size_of::<T>())
    {
        Some(result_ptr.read::<T>())
    } else {
        None
    }
}

/// Clears a value of type `T` from slots of 32 bytes each, starting at `slot` with an `offset` given in words.
///
/// # Additional Information
///
/// If `T` is a zero-sized type, no storage access will occur. Storage API does not store zero-sized types in storage,
/// so clearing a zero-sized type from the slot and offset will have no effect.
///
/// If `T` is a zero-sized type, the function always returns `true`, regardless of the `slot` and `offset`.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot from which to count the `offset`. This or the following slots can be cleared.
/// * `offset`: [u64] - An offset, *in words*, from the start of `slot`, from which the value should be cleared.
///
/// # Returns
///
/// * [bool] - `true` if _all_ the cleared storage slots were previously set. Otherwise, `false`.
///
/// # Number of Storage Accesses
///
/// * Clears: `1`
///
/// # Examples
///
/// ```sway
/// use std::storage::storage_api::{read_quads, write_quads, clear_quads};
///
/// fn foo() {
///     let five = 5_u64;
///     write_quads(b256::zero(), 0, five);
///     let cleared = clear_quads::<u64>(b256::zero(), 0);
///     assert(cleared);
///     assert(read_quads::<u64>(b256::zero(), 0).is_none());
/// }
/// ```
#[storage(write)]
pub fn clear_quads<T>(slot: b256, offset: u64) -> bool {
    if __size_of::<T>() == 0 {
        return true;
    }

    // Determine how many slots and where the value is to be cleared.
    let (offset_slot, number_of_slots, _place_in_slot) = slot_calculator::<T>(slot, offset);

    // Clear `number_of_slots * 32` bytes starting at storage slot `slot`.
    __state_clear(offset_slot, number_of_slots)
}

/// Clears a value of type `T` from slots of 32 bytes each, starting at `slot` with an `offset` given in words.
///
/// # Deprecation Notice
///
/// This function is deprecated in favor of `clear_quads`, `clear_slots`, and `clear_slots_existed`.
/// To preserve exactly the same behavior as `clear`, use `clear_quads`.
/// To clear values contained in dynamic slots of variable sizes, use `clear_slots`.
/// To clear values contained in dynamic slots of variable sizes, and obtain information
/// about whether _all_ the cleared slots were previously set, use `clear_slots_existed`.
///
/// # Additional Information
///
/// The function returns `true` if _all_ the cleared storage slots were previously set, otherwise, `false`.
/// If the information about whether the cleared storage slots were previously set is not needed,
/// consider using `clear_slots` because it is more gas efficient.
///
/// If `T` is a zero-sized type, no storage access will occur. Storage API does not store zero-sized types in storage,
/// so clearing a zero-sized type from the slot and offset will have no effect.
///
/// If `T` is a zero-sized type, the function always returns `true`, regardless of the `slot` and `offset`.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot from which to count the `offset`. This or the following slots can be cleared.
/// * `offset`: [u64] - An offset, *in words*, from the start of `slot`, from which the value should be cleared.
///
/// # Returns
///
/// * [bool] - `true` if _all_ the cleared storage slots were previously set. Otherwise, `false`.
///
/// # Number of Storage Accesses
///
/// * Clears: `1`
///
/// # Examples
///
/// ```sway
/// use std::storage::storage_api::{read, write, clear};
///
/// fn foo() {
///     let five = 5_u64;
///     write(b256::zero(), 0, five);
///     let cleared = clear::<u64>(b256::zero(), 0);
///     assert(cleared);
///     assert(read::<u64>(b256::zero(), 0).is_none());
/// }
/// ```
#[deprecated(note = "Use `clear_quads`, `clear_slots`, or `clear_slots_existed` instead.")]
#[storage(write)]
pub fn clear<T>(slot: b256, offset: u64) -> bool {
    clear_quads::<T>(slot, offset)
}

/// Clears `number_of_slots` slots of dynamic size, starting at `slot`.
///
/// # Additional Information
///
/// If `number_of_slots` is zero, storage access will still occur,
/// but no slots will be cleared.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot from which to start clearing.
/// * `number_of_slots`: [u64] - The number of slots to clear.
///
/// # Number of Storage Accesses
///
/// * Clears: `1`
///
/// # Examples
///
/// ```sway
/// use std::storage::storage_api::{read_slot, write_slot, clear_slots};
///
/// fn foo() {
///     let five = 5_u64;
///     write_slot(b256::zero(), five);
///     clear_slots(b256::zero(), 1);
///     assert(read_slot::<u64>(b256::zero(), 0).is_none());
/// }
/// ```
#[storage(write)]
pub fn clear_slots(slot: b256, number_of_slots: u64) {
    __state_clear_slots(slot, number_of_slots);
}

/// Clears `number_of_slots` slots of dynamic size, starting at `slot`,
/// and returns whether _all_ the cleared slots were previously set.
///
/// # Additional Information
///
/// If `number_of_slots` is zero, storage access will still occur,
/// but no slots will be cleared.
///
/// If `number_of_slots` is zero, function always returns `true`,
/// regardless of the `slot`.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot from which to start clearing.
/// * `number_of_slots`: [u64] - The number of slots to clear.
///
/// # Returns
///
/// * [bool] - `true` if _all_ the cleared storage slots were previously set. Otherwise, `false`.
///
/// # Number of Storage Accesses
///
/// * Preloads: `number_of_slots` (to check whether the slots were previously set)
/// * Clears: `1`
///
/// # Examples
///
/// ```sway
/// use std::storage::storage_api::{read_slot, write_slot, clear_slots_existed};
///
/// fn foo() {
///     let five = 5_u64;
///     write_slot(b256::zero(), five);
///     let cleared = clear_slots_existed(b256::zero(), 1);
///     assert(cleared);
///     assert(read_slot::<u64>(b256::zero(), 0).is_none());
/// }
/// ```
#[storage(read, write)]
pub fn clear_slots_existed(slot: b256, number_of_slots: u64) -> bool {
    let mut slot_counter = number_of_slots;
    let mut current_slot = slot;
    let mut existed = true;
    while existed && slot_counter > 0 {
        existed = __state_preload(current_slot) != 0;
        add_u64_to_b256(current_slot, 1);
        slot_counter -= 1;
    }
    __state_clear_slots(slot, number_of_slots);
    existed
}

/// Given a `slot`, `offset`, and type `T`, this function determines where
/// a value of type `T` should be stored in 32-byte storage slots,
/// how many slots it will occupy, and where in the first slot it
/// will be placed based on the `offset`.
///
/// # Arguments
///
/// * `slot`: [b256] - The starting address at which a value should be stored.
/// * `offset`: [u64] - The offset from `slot` to store the value.
///
/// # Returns
///
/// * [b256] - The calculated actual first slot to store the value.
/// * [u64] - The number of slots the value will occupy in storage.
/// * [u64] - The word in the first slot where the value will start.
fn slot_calculator<T>(slot: b256, offset: u64) -> (b256, u64, u64) {
    let size_of_t = __size_of::<T>();

    // Get the last storage slot needed based on the size of `T`.
    // ((offset * bytes_in_word) + bytes + (bytes_in_slot - 1)) >> align_to_slot = last slot
    let last_slot = ((offset * 8) + size_of_t + 31) >> 5;

    // Where in the storage slot to align `T` in order to pack word-aligned.
    // offset % number_words_in_slot = word_place_in_slot
    let place_in_slot = offset % 4;

    // Get the number of slots `T` spans based on its packed position.
    // ((place_in_slot * bytes_in_word) + bytes + (bytes_in_slot - 1)) >> align_to_slot = number_of_slots
    let number_of_slots = if __is_reference_type::<T>() {
        ((place_in_slot * 8) + size_of_t + 31) >> 5
    } else {
        1
    };

    // Determine which starting slot `T` will be stored based on the offset.
    let mut offset_slot = slot.as_u256();
    add_u64_to_u256(offset_slot, last_slot - number_of_slots);
    (__transmute::<u256, b256>(offset_slot), number_of_slots, place_in_slot)
}

#[inline(always)]
fn add_u64_to_u256(ref mut num: u256, val: u64) {
    asm(num: num, val: val) {
        wqop num num val i0;
    }
}

#[inline(always)]
fn add_u64_to_b256(ref mut num: b256, val: u64) {
    asm(num: num, val: val) {
        wqop num num val i0;
    }
}
