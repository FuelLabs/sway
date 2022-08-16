//! Functionality for setting and unsetting FuelVM flags to modify behavior related to the `$err` and `$of` registers
library flags;

use ::context::registers::flags;

/// Call this function to allow overflowing operations to occur without a FuelVM panic.
/// IMPORTANT !!! Don't forget to call enable_panic_on_overflow() after performing the operations for which you disabled the default panic-on-overflow behavior in the first place !
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

/// Call this function to re-enable the panic-on-overflow behavior in the FuelVM. Note that panic-on-overflow is the default, so there is no need to use this function unless you have previously called disable_panic_on_overflow().
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
