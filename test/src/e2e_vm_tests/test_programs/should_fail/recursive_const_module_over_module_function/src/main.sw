library;

fn mod_fn() -> u8 {
    MOD_CONST
}

const MOD_CONST: u8 = mod_fn();