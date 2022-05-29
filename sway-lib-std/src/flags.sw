//! Functionality for setting & unsetting Fuel-VM flags to modify behavior related to the $err and $of registers
library flags;

use ::context::registers::flags;

pub fn disable_overflow() {
    // Mask second bit, which is `F_WRAPPING`.
    // TODO can't use binary literal: https://github.com/FuelLabs/sway/issues/1664
    // 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000010
    let mask = 2;
    // Get the current value of the flags register and mask it, setting the
    // masked bit. Flags are inverted, so set = off.
    let flag_val = flags() | mask;
    asm(flag_val: flag_val) {
        flag flag_val;
    }
}

pub fn enable_overflow() {
    // Mask second bit, which is `F_WRAPPING`.
    // TODO can't use binary literal: https://github.com/FuelLabs/sway/issues/1664
    // 0b11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111101
    let mask = 18446744073709551613;
    // Get the current value of the flags register and mask it, unsetting the
    // masked bit. Flags are inverted, so unset = on.
    let flag_val = flags() & mask;
    asm(flag_val: flag_val) {
        flag flag_val;
    }
}