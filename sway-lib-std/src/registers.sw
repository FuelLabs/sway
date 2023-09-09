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
///     assert(!pc.is_null());
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
///     assert(!ssp.is_null());
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
///     assert(!sp.is_null());
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
///     assert(!fp.is_null());
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
///     assert(!hp.is_null());
/// }
/// ```
pub fn heap_ptr() -> raw_ptr {
    asm() { hp: raw_ptr }
}

/// Error codes for particular operations.
///
/// # Additional Information
///
/// Normally, if the result of an ALU operation is mathematically undefined (e.g. dividing by zero), the VM Reverts.
/// However, if the `F_UNSAFEMATH` flag is set, $err is set to `true` and execution continues.
///
/// # Returns
///
/// * [u64] - A VM error code.
///
/// # Examples
///
/// ```sway
/// use std::{registers::error, flags::{disable_panic_on_unsafe_math, enable_panic_on_unsafe_math}};
///
/// fn foo() {
///     disable_panic_on_unsafe_math();
///     let bar = 1 / 0;
///     assert(error() == 1);
///     enable_panic_on_unsafe_math();
/// }
/// ```
pub fn error() -> u64 {
    asm() { err }
}

/// Remaining gas globally.
///
/// # Returns
///
/// * [u64] - The remaining gas.
///
/// # Examples
///
/// ```sway
/// use std::registers::global_gas;
///
/// fn foo() {
///     let gas = global_gas();
///     assert(gas != 0);
///     bar();
///
///     let gas_2 = global_gas();
///     assert(gas_2 < gas);
/// }
///
/// fn bar() {
///     let val = 0;
/// }
/// ```
pub fn global_gas() -> u64 {
    asm() { ggas }
}

/// Remaining gas in the context.
///
/// # Returns
///
/// * [u64] - The remaining gas for the curren context.
///
/// # Examples
///
/// ```sway
/// use std::registers::context_gas;
///
/// fn foo() {
///     let gas = context_gas();
///     let gas_2 = bar();
///     assert(gas_2 < gas);
/// }
///
/// fn bar() -> u64 {
///     context_gas();
/// }
/// ```
pub fn context_gas() -> u64 {
    asm() { cgas }
}

/// Get the amount of units of `call_frames::msg_asset_id()` being sent.
///
/// # Returns
///
/// * [u64] - The forwarded coins in the context.
///
/// # Examples
/// ```sway
/// use std::register::balance;
///
/// fn foo() {
///     let bal = balance();
///     assert(bal == 0);
/// }
/// ```
pub fn balance() -> u64 {
    asm() { bal }
}

/// Pointer to the start of the currently-executing code.
///
/// # Returns
///
/// * [raw_ptr] - The memory location of the start of the currently-executing code.
///
/// # Examples
///
/// ```sway
/// use std::registers::instrs_start;
///
/// fn foo() {
///     let is = instrs_start();
///     assert(!is.is_null());
/// }
/// ```
pub fn instrs_start() -> raw_ptr {
    asm() { is: raw_ptr }
}

/// Return value or pointer.
///
/// # Returns
///
/// * [u64] - The value or pointer stored in the return register of the VM for the current context.
///
/// # Examples
///
/// ```sway
/// use std::registers::return_value;
///
/// fn foo() {
///     let ret = return_value();
///     assert(ret == 0);
/// }
/// ```
pub fn return_value() -> u64 {
    asm() { ret }
}

/// Return value length in bytes.
///
/// # Returns
///
/// * [u64] - The length in bytes of the value stored in the return register of the VM for the current context.
///
/// # Examples
///
/// ```sway
/// use std::registers::return_length;
///
/// fn foo() {
///     let ret = return_length();
///     assert(ret == 0);
/// }
/// ```
pub fn return_length() -> u64 {
    asm() { retl }
}

/// Flags register.
///
/// # Returns
///
/// * [u64] - The current flags set within the VM.
///
/// # Examples
///
/// ```sway
/// use std::{registers::flags, flags::disable_panic_on_overflow};
///
/// const F_WRAPPING_DISABLE_MASK: u64 = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000010;
///
/// fn foo() {
///     let flag = flags();
///     assert(flag == 0);
///     disable_panic_on_overflow();
///     let flag_2 = flags();
///     assert(flag_2 == F_WRAPPING_DISABLE_MASK);
/// }
/// ```
pub fn flags() -> u64 {
    asm() { flag }
}
