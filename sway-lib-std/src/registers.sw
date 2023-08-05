//! Functions to expose 14 of the reserved FuelVM registers for ease of use.
//! Ref: https://fuellabs.github.io/fuel-specs/master/vm#semantics
library;

/// Contains overflow & underflow of addition, subtraction, and multiplication.
///
/// # Addtional Information
///
/// In order to use this function, panic on overflow must be disabled.
///
/// # Returns
///
/// * [u64] - The overflow or underflow remaining value.
///
/// # Examples
///
/// ```sway
/// use std::{registers::overflow, flags::{disable_panic_on_overflow, enable_panic_on_overflow}};
///
/// fn foo() {
///    disable_panic_on_overflow();
///    let max = u64::max();
///    let result = max + 1;
///    let overflow_val = overflow();
///
///    assert(result == 0);
///    assert(overflow_val == 1);
///    enable_panic_on_overflow();
/// }
/// ```
pub fn overflow() -> u64 {
    asm() { of }
}

/// The program counter. Memory address of the current instruction.
///
/// # Returns
///
/// * [raw_ptr] - The location in memory of the current instruction.
///
/// # Examples
///
/// ```sway
/// use std::registers::program_counter;
///
/// fn foo() {
///     let pc = program_counter();
///     assert(pc.is_null() == false);
/// }
/// ```
pub fn program_counter() -> raw_ptr {
    asm() { pc: raw_ptr }
}

/// Memory address of bottom of current writable stack area.
///
/// # Returns
///
/// * [raw_ptr] - The location in memory of the bottom of the stack.
///
/// # Examples
///
/// ```sway
/// use std::registers::stack_start_ptr;
///
/// fn foo() {
///     let ssp = stack_start_ptr();
///     assert(ssp.is_null() == false);
/// }
/// ```
pub fn stack_start_ptr() -> raw_ptr {
    asm() { ssp: raw_ptr }
}

/// Memory address on top of current writable stack area (points to free memory).
///
/// # Returns
///
/// * [raw_ptr] - The location in memory of the top of the stack.
///
/// # Examples
///
/// ```sway
/// use std::registers::stack_ptr;
///
/// fn foo() {
///     let sp = stack_ptr();
///     assert(sp.is_null() == false);
/// }
/// ```
pub fn stack_ptr() -> raw_ptr {
    asm() { sp: raw_ptr }
}

/// Memory address of beginning of current call frame.
///
/// # Returns
///
/// * [raw_ptr] - The location in memory of the start of the call frame.
///
/// # Examples
///
/// ```sway
/// use std::registers::frame_ptr;
///
/// fn foo() {
///     let fp = frame_ptr();
///     assert(fp.is_null() == false);
/// }
/// ```
pub fn frame_ptr() -> raw_ptr {
    asm() { fp: raw_ptr }
}

/// Memory address below the current bottom of the heap (points to free memory).
///
/// # Returns
///
/// * [raw_ptr] - The location in memory of the bottom of the heap.
///
/// # Examples
///
/// ```sway
/// use std::registers::heap_ptr;
///
/// fn foo() {
///     let hp = heap_ptr();
///     assert(hp.is_null() == false);
/// }
/// ```
pub fn heap_ptr() -> raw_ptr {
    asm() { hp: raw_ptr }
}

/// Error codes for particular operations.
pub fn error() -> u64 {
    asm() { err }
}

/// Remaining gas globally.
pub fn global_gas() -> u64 {
    asm() { ggas }
}

/// Remaining gas in the context.
pub fn context_gas() -> u64 {
    asm() { cgas }
}

/// Get the amount of units of `call_frames::msg_asset_id()` being sent.
pub fn balance() -> u64 {
    asm() { bal }
}

/// Pointer to the start of the currently-executing code.
pub fn instrs_start() -> raw_ptr {
    asm() { is: raw_ptr }
}

/// Return value or pointer.
pub fn return_value() -> u64 {
    asm() { ret }
}

/// Return value length in bytes.
pub fn return_length() -> u64 {
    asm() { retl }
}

/// Flags register.
pub fn flags() -> u64 {
    asm() { flag }
}
