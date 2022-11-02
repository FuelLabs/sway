//! Functions to expose 14 of the reserved FuelVM registers for ease of use.
//! Ref: https://fuellabs.github.io/fuel-specs/master/vm#semantics
library registers;

/// Contains overflow/underflow of addition, subtraction, and multiplication.
pub fn overflow() -> u64 {
    asm() { of }
}

/// The program counter. Memory address of the current instruction.
pub fn program_counter() -> raw_ptr {
    asm() { pc: raw_ptr }
}

/// Memory address of bottom of current writable stack area.
pub fn stack_start_ptr() -> raw_ptr {
    asm() { ssp: raw_ptr }
}

/// Memory address on top of current writable stack area (points to free memory).
pub fn stack_ptr() -> raw_ptr {
    asm() { sp: raw_ptr }
}

/// Memory address of beginning of current call frame.
pub fn frame_ptr() -> raw_ptr {
    asm() { fp: raw_ptr }
}

/// Memory address below the current bottom of the heap (points to free memory).
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
