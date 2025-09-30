library;

use ::alloc::alloc;
use ::option::Option::{self, *};
use ::ops::*;
use ::primitive_conversions::{b256::*, u256::*, u64::*};

/// Stores a stack value in storage. Will not work for heap values.
///
/// # Additional Information
///
/// If the value crosses the boundary of a storage slot, writing continues at the following slot.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot at which the variable will be stored.
/// * `offset`: [u64] - An offset starting at the beginning of `slot` at which `value` should be stored.
/// * `value`: [T] - The value to be stored.
///
/// # Number of Storage Accesses
///
/// * Reads: `1`
/// * Writes: `1`
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
///     assert(five == stored_five);
/// }
/// ```
#[storage(read, write)]
pub fn write<T>(slot: b256, offset: u64, value: T) {
    if __size_of::<T>() == 0 {
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

/// Reads a value of type `T` starting at the location specified by `slot` and `offset`. If the
/// value crosses the boundary of a storage slot, reading continues at the following slot.
///
/// # Arguments
///
/// * `slot`: [b256] - The storage slot to load the value from.
/// * `offset`: [u64] - An offset, in words, from the start of `slot`, from which the value should be read.
///
/// # Returns
///
/// * [Option<T>] - `Option(value)` if the storage slots read were valid and contain `value`. Otherwise,
/// returns `None`.
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
///     let stored_five = read::<u64>(b256::zero(), 2);
///     assert(five == stored_five.unwrap());
/// }
/// ```
#[storage(read)]
pub fn read<T>(slot: b256, offset: u64) -> Option<T> {
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

/// Clear a value starting at some slot with an offset.
///
/// # Arguments
///
/// * `slot` - The key of the stored value that will be cleared
/// * `offset` - An offset, in words, from the start of `slot`, from which the value should be cleared.
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
///     let cleared = clear::<u64>(b256::zero());
///     assert(cleared);
///     assert(read::<u64>(b256::zero(), 0).is_none());
/// }
/// ```
#[storage(write)]
pub fn clear<T>(slot: b256, offset: u64) -> bool {
    if __size_of::<T>() == 0 {
        return true;
    }

    // Determine how many slots and where the value is to be cleared.
    let (offset_slot, number_of_slots, _place_in_slot) = slot_calculator::<T>(slot, offset);

    // Clear `number_of_slots * 32` bytes starting at storage slot `slot`.
    __state_clear(offset_slot, number_of_slots)
}

/// Given a slot, offset, and type this function determines where something should be stored.
///
/// # Arguments
///
/// * `slot`: [b256] - The starting address at which something should be stored.
/// * `offset`: [u64] - The offset from `slot` to store the value.
///
/// # Returns
///
/// [b256] - The calculated offset slot to store the value.
/// [u64] - The number of slots the value will occupy in storage.
/// [u64] - The word in the slot where the value will start.
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
    let number_of_slots = match __is_reference_type::<T>() {
        true => ((place_in_slot * 8) + size_of_t + 31) >> 5,
        false => 1,
    };

    // Determine which starting slot `T` will be stored based on the offset.
    let mut offset_slot = slot.as_u256();
    offset_slot += last_slot.as_u256() - number_of_slots.as_u256();
    (offset_slot.as_b256(), number_of_slots, place_in_slot)
}
