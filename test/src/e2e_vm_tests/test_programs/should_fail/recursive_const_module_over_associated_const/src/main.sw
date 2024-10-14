library;

struct S {}

impl S {
    const S_ASSOC: u8 = MOD_CONST;
}

const MOD_CONST: u8 = S::S_ASSOC;