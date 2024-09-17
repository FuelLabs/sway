library;

struct S {}

impl S {
    fn assoc() -> u8 {
        MOD_CONST
    }
}

const MOD_CONST: u8 = S::assoc();