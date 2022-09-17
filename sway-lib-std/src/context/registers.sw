//! Functions to expose 14 of the reserved FuelVM registers for ease of use.
//! Ref: https://github.com/FuelLabs/fuel-specs/blob/master/specs/vm/main.md#semantics
library registers;

/// Contains overflow/underflow of addition, subtraction, and multiplication.
pub fn overflow() -> u64 {
    asm() { of }
}

/// The program counter. Memory address of the current instruction.
pub fn program_counter() -> u64 {
    asm() { pc }
}

/// Memory address of bottom of current writable stack area.
pub fn stack_start_ptr() -> u64 {
    asm() { ssp }
}

/// Memory address on top of current writable stack area (points to free memory).
pub fn stack_ptr() -> u64 {
    asm() { sp }
}

/// Memory address of beginning of current call frame.
pub fn frame_ptr() -> u64 {
    asm() { fp }
}

/// Memory address below the current bottom of the heap (points to free memory).
pub fn heap_ptr() -> u64 {
    asm() { hp }
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
pub fn instrs_start() -> u64 {
    asm() { is }
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
