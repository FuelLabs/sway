//! Functionality for setting and unsetting FuelVM flags to modify behavior related to the `$err` and `$of` registers.
library;

use ::{assert::assert, logging::log, registers::{error, flags}};

// Mask second bit, which is `F_WRAPPING`.
const F_WRAPPING_DISABLE_MASK: u64 = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000010;
// Mask second bit, which is `F_WRAPPING`.
const F_WRAPPING_ENABLE_MASK: u64 = 0b11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111101;
// Mask first bit, which is `F_UNSAFEMATH`.
const F_UNSAFEMATH_DISABLE_MASK: u64 = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000001;
// Mask first bit, which is `F_UNSAFEMATH`.
const F_UNSAFEMATH_ENABLE_MASK: u64 = 0b11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111110;

/// Sets the flag register to the given value.
///
/// # Arguments
///
/// * `new_flags`: [u64] - Binary encoded 64 bit value representing the flags to set.
///
/// # Examples
///
/// ```sway
/// use std::flags::{disable_panic_on_overflow_preserving, set_flags};
///
/// fn foo() {
///     let prior_flags = disable_panic_on_overflow_preserving();
///
///     // Adding 1 to the max value of a u64 is considered an overflow.
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

/// Allows overflowing operations to occur without a FuelVM panic.
///
/// # Additional Information
///
/// > **_WARNING:_**
/// >
/// > Don't forget to call `enable_panic_on_overflow` or `set_flags` after performing the operations for which you disabled the default `panic-on-overflow` behavior in the first place!
///
/// # Returns
///
/// * [u64] - The flag prior to disabling panic on overflow.
///
/// # Examples
///
/// ```sway
/// use std::flags::{disable_panic_on_overflow, enable_panic_on_overflow};
///
/// fn main() {
///     disable_panic_on_overflow();
///
///     // Adding 1 to the max value of a u64 is considered an overflow.
///     let bar = u64::max() + 1;
///
///     enable_panic_on_overflow();
/// }
/// ```
///
/// ```sway
/// use std::flags::{disable_panic_on_overflow, set_flags};
///
/// fn foo() {
///     let prior_flags = disable_panic_on_overflow();
///
///     // Adding 1 to the max value of a u64 is considered an overflow.
///     let bar = u64::max() + 1;
///
///     set_flags(prior_flags);
/// }
/// ```
pub fn disable_panic_on_overflow() -> u64 {
    let prior_flags = flags();

    // Get the current value of the flags register and mask it, setting the
    // masked bit. Flags are inverted, so set = off.
    let flag_val = prior_flags | F_WRAPPING_DISABLE_MASK;
    asm(flag_val: flag_val) {
        flag flag_val;
    }

    prior_flags
}

/// Enables the default `panic-on-overflow` behavior in the FuelVM.
///
/// # Additional Information
///
/// > **_Note:_**
/// >
/// > `panic-on-overflow` is the default, so there is no need to use this function unless you have previously called `disable_panic_on_overflow`.
///
/// # Examples
///
/// ```sway
/// use std::flags::{disable_panic_on_overflow, enable_panic_on_overflow};
///
/// fn main() {
///     disable_panic_on_overflow();
///
///     // Adding 1 to the max value of a u64 is considered an overflow.
///     let bar = u64::max() + 1;
///
///     enable_panic_on_overflow();
/// }
/// ```
pub fn enable_panic_on_overflow() {
    // Get the current value of the flags register and mask it, unsetting the
    // masked bit. Flags are inverted, so unset = on.
    let flag_val = flags() & F_WRAPPING_ENABLE_MASK;
    asm(flag_val: flag_val) {
        flag flag_val;
    }
}

/// Allows unsafe math operations to occur without a FuelVM panic.
/// Sets the `$err` register to `true` whenever unsafe math occurs.
///
/// # Additional Information
///
/// > **_WARNING:_**
/// >
/// > Don't forget to call `enable_panic_on_unsafe_math` or `set_flags` after performing the operations for which you disabled the default `panic-on-unsafe-math` behavior in the first place!
///
/// # Returns
///
/// * [u64] - The flag prior to disabling panic on overflow.
///
/// # Examples
///
/// ```sway
/// use std::{assert::assert, flags::{disable_panic_on_unsafe_math, enable_panic_on_unsafe_math}, registers::error};
///
/// fn main() {
///     disable_panic_on_unsafe_math();
///
///     // Division by zero is considered unsafe math.
///     let bar = 1 / 0;
///     // Error flag is set to true whenever unsafe math occurs. Here represented as 1.
///     assert(error() == 1);
///
///     enable_panic_on_unsafe_math();
/// }
/// ```
///
/// ```sway
/// use std::{assert::assert, flags::{disable_panic_on_unsafe_math, set_flags}, registers::error};
///
/// fn foo() {
///     let prior_flags = disable_panic_on_unsafe_math();
///
///     // Division by zero is considered unsafe math.
///     let bar = 1 / 0;
///     // Error flag is set to true whenever unsafe math occurs. Here represented as 1.
///     assert(error() == 1);
///
///     set_flags(prior_flags);
/// }
/// ```
pub fn disable_panic_on_unsafe_math() -> u64 {
    let prior_flags = flags();

    // Get the current value of the flags register and mask it, setting the
    // masked bit. Flags are inverted, so set = off.
    let flag_val = prior_flags | F_UNSAFEMATH_DISABLE_MASK;
    asm(flag_val: flag_val) {
        flag flag_val;
    }

    prior_flags
}

/// Enables the default `panic-on-unsafe-math` behavior in the FuelVM.
///
/// # Additional Information
///
/// > **_Note:_**
/// >
/// > `panic-on-unsafe-math` is the default, so there is no need to use this function unless you have previously called `disable_panic_on_unsafe_math`.
///
/// # Examples
///
/// ```sway
/// use std::{assert::assert, flags::{disable_panic_on_unsafe_math, enable_panic_on_unsafe_math}, registers::error};
///
/// fn main() {
///     disable_panic_on_unsafe_math();
///
///     // Division by zero is considered unsafe math.
///     let bar = 1 / 0;
///     // Error flag is set to true whenever unsafe math occurs. Here represented as 1.
///     assert(error() == 1);
///
///     enable_panic_on_unsafe_math();
/// }
/// ```
pub fn enable_panic_on_unsafe_math() {
    // Get the current value of the flags register and mask it, unsetting the
    // masked bit. Flags are inverted, so unset = on.
    let flag_val = flags() & F_UNSAFEMATH_ENABLE_MASK;
    asm(flag_val: flag_val) {
        flag flag_val;
    }
}

#[test]
fn test_disable_panic_on_overflow() {
    let _ = disable_panic_on_overflow();
    let _bar = u64::max() + 1;
    enable_panic_on_overflow();
}

#[test]
fn test_disable_panic_on_overflow_preserving() {
    let _ = disable_panic_on_overflow();

    let prior_flags = disable_panic_on_overflow();
    let _bar = u64::max() + 1;
    set_flags(prior_flags);

    let _bar = u64::max() + 1;

    enable_panic_on_overflow();
}

#[test]
fn test_disable_panic_on_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();

    let _bar = asm(r2: 1, r3: 0, r1) {
        div  r1 r2 r3;
        r1: u64
    };

    assert(error() == 1);

    enable_panic_on_unsafe_math();
}

#[test]
fn test_disable_panic_on_unsafe_math_preserving() {
    let _ = disable_panic_on_unsafe_math();

    let prior_flags = disable_panic_on_unsafe_math();
    let _bar = asm(r2: 1, r3: 0, r1) {
        div  r1 r2 r3;
        r1: u64
    };
    assert(error() == 1);
    set_flags(prior_flags);

    let _bar = asm(r2: 1, r3: 0, r1) {
        div  r1 r2 r3;
        r1: u64
    };
    assert(error() == 1);

    enable_panic_on_unsafe_math();
}
