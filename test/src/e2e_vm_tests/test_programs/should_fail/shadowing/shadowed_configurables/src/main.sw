script;

mod lib;

const CONST_A: u8 = 1;

use lib::LIB_X;

configurable {
    X: u8 = 10,
    Y: u8 = 11,
    A: u8 = 13,
    B: u8 = 14,
    CONST_A: u8 = 15,
    CONST_B: u8 = 16,
    LIB_X: u8 = 17,
    LIB_Y: u8 = 18,
    LIB_Z_ALIAS: u8 = 19,
    LET_A: u8 = 20,
    LET_B: u8 = 21,
    LET_C: u8 = 22,
}

const CONST_B: u8 = 2;

use lib::LIB_Y;

use lib::LIB_Z as LIB_Z_ALIAS;

struct S {
    x: u8,
}

enum E {
    A: u8,
}

fn main() {
    let X = 101u8;
    const Y: u8 = 102;

    {
        let A = 103u8;
        const B: u8 = 104;
    }

    let S { x: LET_A } = S { x: 105 };
    let (_, LET_B) = (106u8, 107u8);
}
