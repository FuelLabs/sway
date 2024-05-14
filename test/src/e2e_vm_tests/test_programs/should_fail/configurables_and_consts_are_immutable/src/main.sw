script;

struct S {
    x: u8,
}

const CONST: u64 = 0;
const CONST_S: S = S { x: 0 };

configurable {
    C: u64 = 0,
    C_S: S = S { x: 0 },
}

fn main() { 
    C = 1;
    C_S.x = 1;

    CONST = 1;
    CONST_S.x = 1;
}
