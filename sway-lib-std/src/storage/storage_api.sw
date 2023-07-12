library;

use ::alloc::alloc;
use ::option::Option::{self, *};

/// Store a stack value in storage. Will not work for heap values.
///
/// ### Arguments
///
/// * `slot` - The storage slot at which the variable will be stored.
/// * `value` - The value to be stored.
/// * `offset` - An offset, in words, from the beginning of `slot`, at which `value` should be
///              stored.
///
/// ### Examples
///
/// ```sway
/// let five = 5_u64;
/// write(ZERO_B256, 2, five);
/// let stored_five = read::<u64>(ZERO_B256, 2).unwrap();
/// assert(five == stored_five);
/// ```
#[storage(read, write)]
pub fn write<T>(slot: b256, offset: u64, value: T) {
    let size_of_t = __size_of::<T>();
    if size_of_t == 0 {
        return;
    }

    // Get the last storage slot needed based on the size of `T`
    let last_slot = match __is_reference_type::<T>() {
        true => ((offset * size_of_t) + size_of_t + 31) >> 5,
        false => ((offset * 8) + 8 + 31) >> 5,
    };

    // Where in the storage slot to align `T` in order to pack word-aligned
    let place_in_slot = match __is_reference_type::<T>() {
        true => (offset * (size_of_t / 8)) % 4,
        false => offset % 4,
    };
 
    // Get the number of slots `T` spans based on it's packed position
    let number_of_slots = match __is_reference_type::<T>() {
        true => ((place_in_slot + (size_of_t / 8) - 1) / 4) + 1,
        false => 1,
    };

    // Determine which slot `T` will be stored based on the offset
    let mut offset_slot = slot;
    offset_slot.increment(last_slot - number_of_slots + 1);

    // Allocate enough memory on the heap for `value` as well as any potential padding required due 
    // to `offset`.
    let padded_value = alloc::<u64>(number_of_slots * 32);

    // Read the values that currently exist in the affected storage slots.
    let _ = __state_load_quad(offset_slot, padded_value, number_of_slots);

    // Copy the value to be stored to `padded_value + offset`.
    // padded_value.add::<u64>(offset).write::<T>(value);
    padded_value.add::<u64>(place_in_slot).write::<T>(value);

    // Now store back the data at `padded_value` which now contains the old data but partially 
    // overwritten by the new data in the desired locations.
    let _ = __state_store_quad(offset_slot, padded_value, number_of_slots);
}

/// Reads a value of type `T` starting at the location specified by `slot` and `offset`. If the
/// value crosses the boundary of a storage slot, reading continues at the following slot.
///
/// Returns `Option(value)` if the storage slots read were valid and contain `value`. Otherwise,
/// return `None`.
///
/// ### Arguments
///
/// * `slot` - The storage slot to load the value from.
/// * `offset` - An offset, in words, from the start of `slot`, from which the value should be read.
///
/// ### Examples
///
/// ```sway
/// let five = 5_u64;
/// write(ZERO_B256, 2, five);
/// let stored_five = read::<u64>(ZERO_B256, 2);
/// assert(five == stored_five);
/// ```
#[storage(read)]
pub fn read<T>(slot: b256, offset: u64) -> Option<T> {
    let size_of_t = __size_of::<T>();
    if size_of_t == 0 {
        return None;
    }

    // Get the last storage slot needed based on the size of `T`
    let last_slot = match __is_reference_type::<T>() {
        true => ((offset * size_of_t) + size_of_t + 31) >> 5,
        false => ((offset * 8) + 8 + 31) >> 5,
    };

    // Where in the storage slot to align `T` in order to pack word-aligned
    let place_in_slot = match __is_reference_type::<T>() {
        true => (offset * (size_of_t / 8)) % 4,
        false => offset % 4,
    };
 
    // Get the number of slots `T` spans based on it's packed position
    let number_of_slots = match __is_reference_type::<T>() {
        true => ((place_in_slot + (size_of_t / 8) - 1) / 4) + 1,
        false => 1,
    };

    let mut offset_slot = slot;
    offset_slot.increment(last_slot - number_of_slots + 1);

    // Allocate a buffer for the result. Its size needs to be a multiple of 32 bytes so we can 
    // make the 'quad' storage instruction read without overflowing.
    let result_ptr = alloc::<u64>(number_of_slots * 32);

    // Read `number_of_slots * 32` bytes starting at storage slot `slot` and return an `Option` 
    // wrapping the value stored at `result_ptr + offset` if all the slots are valid. Otherwise, 
    // return `None`.
    if __state_load_quad(offset_slot, result_ptr, number_of_slots) {
        Some(result_ptr.add::<u64>(place_in_slot).read::<T>())
    } else {
        None
    }
}

/// Clear a sequence of consecutive storage slots starting at a some slot with an offset. Returns a Boolean
/// indicating whether all of the storage slots cleared were previously set.
///
/// ### Arguments
///
/// * `slot` - The key of the first storage slot that will be cleared
/// * `offset` - An offset, in words, from the start of `slot`, from which the value should be cleared.
///
/// ### Examples
///
/// ```sway
/// let five = 5_u64;
/// write(ZERO_B256, 0, five);
/// let cleared = clear::<u64>(ZERO_B256, 0);
/// assert(cleared);
/// assert(read::<u64>(ZERO_B256, 0).is_none());
/// ```
#[storage(write)]
pub fn clear<T>(slot: b256, offset: u64) -> bool {
    // Get the last storage slot needed based on the size of `T`
    let last_slot = match __is_reference_type::<T>() {
        true => ((offset * __size_of::<T>()) + __size_of::<T>() + 31) >> 5,
        false => ((offset * 8) + 8 + 31) >> 5,
    };

    // Where in the storage slot to align `T` in order to pack word-aligned
    let place_in_slot = match __is_reference_type::<T>() {
        true => (offset * (__size_of::<T>() / 8)) % 4,
        false => offset % 4,
    };
 
    // Get the number of slots `T` spans based on it's packed position
    let number_of_slots = match __is_reference_type::<T>() {
        true => ((place_in_slot + (__size_of::<T>() / 8) - 1) / 4) + 1,
        false => 1,
    };

    let mut offset_slot = slot;
    offset_slot.increment(last_slot - number_of_slots + 1);

    // Clear `number_of_slots * 32` bytes starting at storage slot `slot`.
    __state_clear(offset_slot, number_of_slots)
}
