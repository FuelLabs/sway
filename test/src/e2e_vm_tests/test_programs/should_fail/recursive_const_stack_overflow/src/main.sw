// This test proves that https://github.com/FuelLabs/sway/issues/6540 is fixed.

library;

pub const MOD_FN: u8 = mod_fn();

fn mod_fn() -> u8 {
    MOD_FN
}

struct S {}

impl S {
    const S_ASSOC: u8 = MOD_CONST;
}

const MOD_CONST: u8 = S::S_ASSOC;
