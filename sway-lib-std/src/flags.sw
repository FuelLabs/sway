//! Functionality for setting and unsetting FuelVM flags to modify behavior related to the `$err` and `$of` registers.
library;

use ::registers::flags;

/// Allows overflowing operations to occur without a FuelVM panic.
///
/// > **_WARNING:_**
/// >
/// > Don't forget to call `enable_panic_on_overflow` after performing the operations for which you disabled the default `panic-on-overflow` behavior in the first place!
///
/// ### Examples
///
/// ```sway
/// use std::flags::{disable_panic_on_overflow, enable_panic_on_overflow};
///
/// fn main() {
///     disable_panic_on_overflow();
///      
///     let bar = u64::max() + 1;
///
///     enable_panic_on_overflow();
/// }
/// ```
pub fn disable_panic_on_overflow() {
    // Mask second bit, which is `F_WRAPPING`.
    let mask = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000010;

    // Get the current value of the flags register and mask it, setting the
    // masked bit. Flags are inverted, so set = off.
    let flag_val = flags() | mask;
    asm(flag_val: flag_val) {
        flag flag_val;
    }
}

/// Enables the default `panic-on-overflow` behavior in the FuelVM.
///
/// > **_Note:_**
/// >
/// > `panic-on-overflow` is the default, so there is no need to use this function unless you have previously called `disable_panic_on_overflow`.
///
/// ### Examples
///
/// ```sway
/// use std::flags::{disable_panic_on_overflow, enable_panic_on_overflow};
///
/// fn main() {
///     disable_panic_on_overflow();
///      
///     let bar = u64::max() + 1;
///
///     enable_panic_on_overflow();
/// }
/// ```
pub fn enable_panic_on_overflow() {
    // Mask second bit, which is `F_WRAPPING`.
    let mask = 0b11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111101;

    // Get the current value of the flags register and mask it, unsetting the
    // masked bit. Flags are inverted, so unset = on.
    let flag_val = flags() & mask;
    asm(flag_val: flag_val) {
        flag flag_val;
    }
}

/// Allows overflowing operations to occur without a FuelVM panic.
/// More suitable than `disable_panic_on_overflow` for use in functions that are not the entry point of a program.
///
/// > **_WARNING:_**
/// >
/// > Don't forget to call `set_flags` after performing the operations for which you disabled the default `panic-on-overflow` behavior in the first place!
///
/// ### Examples
///
/// ```sway
/// use std::flags::{disable_panic_on_overflow_preserving, set_flags};
///
/// fn foo() {
///     let prior_flags = disable_panic_on_overflow_preserving();
///      
///     let bar = u64::max() + 1;
///
///     set_flags(prior_flags);
/// }
/// ```
pub fn disable_panic_on_overflow_preserving() -> u64 {
    let prior_flags = flags();

    // Mask second bit, which is `F_WRAPPING`.
    let mask = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000010;

    // Get the current value of the flags register and mask it, setting the
    // masked bit. Flags are inverted, so set = off.
    let flag_val = prior_flags | mask;
    asm(flag_val: flag_val) {
        flag flag_val;
    }

    prior_flags
}

/// Sets the flag register to the given value.
///
/// ### Arguments
///
/// * `flags` - Binary encoded 64 bit value representing the flags to set.
///
/// ### Examples
///
/// ```sway
/// use std::flags::{disable_panic_on_overflow_preserving, set_flags};
///
/// fn foo() {
///     let prior_flags = disable_panic_on_overflow_preserving();
///      
///     let bar = u64::max() + 1;
///
///     set_flags(prior_flags);
/// }
/// ```
pub fn set_flags(new_flags: u64) {
    asm(new_flags: new_flags) {
        flag new_flags;
    }
}

#[test]
fn test_disable_panic_on_overflow() {
    disable_panic_on_overflow();
    let _bar = u64::max() + 1;
    enable_panic_on_overflow();
}

#[test]
fn test_disable_panic_on_overflow_preserving() {
    disable_panic_on_overflow();

    let prior_flags = disable_panic_on_overflow_preserving();
    let _bar = u64::max() + 1;
    set_flags(prior_flags);

    _bar = u64::max() + 1;

    enable_panic_on_overflow();
}