library;

use ::lib::return_me;

use ::lib::LIB_X;
use ::lib::LIB_Y as ALIAS_LIB_Y;

const MOD_X: u64 = 8;

const NUMBER_1: u64 = 7;
const NUMBER_2: u64 = 14;
const NUMBER_3: u64 = 5;

const TRUE: bool = true;
const FALSE: bool = false;

struct S {
    x: u64,
    b: bool,
}

struct StructWithConstNames {
    MOD_X: u64,
    LOCAL_X: u64,
    LIB_X: u64,
    ALIAS_LIB_Y: u64,
}

pub fn test() {
    let a = match return_me(S { x: NUMBER_3, b: true }) {
        S { x: NUMBER_1, b: _ } => 1,
        S { x: NUMBER_2, b: _ } => 2,
        S { x: NUMBER_3, b: _ } => 42,
        _ => 1111,
    };

    assert_eq(a, 42);

    let b = match return_me(S { x: 0, b: TRUE }) {
        S { x: _, b: TRUE } => 42,
        S { x: _, b: FALSE } => 3,
    };
    
    assert_eq(b, 42);

    const LOCAL_X: u64 = 13;

    let c = match return_me(S { x: MOD_X, b: true }) {
        S { x: MOD_X, b: _ } => {
            42
        },
        S { x: LOCAL_X, b: _ } => {
            4
        },
        S { x: LIB_X, b: _ } => {
            5
        },
        S { x: ALIAS_LIB_Y, b: _ } => {
            6
        },
        _ => 2222,
    };
    
    assert_eq(c, 42);

    let c = match return_me(S { x: LOCAL_X, b: true }) {
        S { x: MOD_X, b: _ } => {
            7
        },
        S { x: LOCAL_X, b: _ } => {
            42
        },
        S { x: LIB_X, b: _ } => {
            8
        },
        S { x: ALIAS_LIB_Y, b: _ } => {
            9
        },
        _ => 3333,
    };
    
    assert_eq(c, 42);

    let c = match return_me(S { x: LIB_X, b: true }) {
        S { x: MOD_X, b: _ } => {
            10
        },
        S { x: LOCAL_X, b: _ } => {
            11
        },
        S { x: LIB_X, b: _ } => {
            42
        },
        S { x: ALIAS_LIB_Y, b: _ } => {
            12
        },
        _ => 4444,
    };
    
    assert_eq(c, 42);

    let c = match return_me(S { x: ALIAS_LIB_Y, b: true }) {
        S { x: MOD_X, b: _ } => {
            13
        },
        S { x: LOCAL_X, b: _ } => {
            14
        },
        S { x: LIB_X, b: _ } => {
            15
        },
        S { x: ALIAS_LIB_Y, b: _ } => {
            42
        },
        _ => 5555,
    };
    
    assert_eq(c, 42);

    let c = match return_me(S { x: MOD_X, b: true }) {
        S { x: MOD_X, b: _ } | S { x: LOCAL_X, b: _ } | S { x: LIB_X, b: _ } => {
            42
        },
        S { x: ALIAS_LIB_Y, b: _ } => {
            16
        },
        _ => 6666,
    };
    
    assert_eq(c, 42);

    let c = match return_me(S { x: LOCAL_X, b: true }) {
        S { x: MOD_X, b: _ } | S { x: LOCAL_X, b: _ } | S { x: LIB_X, b: _ } => {
            42
        },
        S { x: ALIAS_LIB_Y, b: _ } => {
            17
        },
        _ => 7777,
    };
    
    assert_eq(c, 42);

    let c = match return_me(S { x: LIB_X, b: true }) {
        S { x: MOD_X, b: _ } | S { x: LOCAL_X, b: _ } | S { x: LIB_X, b: _ } => {
            42
        },
        S { x: ALIAS_LIB_Y, b: _ } => {
            18
        },
        _ => 8888,
    };
    
    assert_eq(c, 42);

    let c = match return_me(S { x: ALIAS_LIB_Y, b: true }) {
        S { x: MOD_X, b: _ } | S { x: LOCAL_X, b: _ } | S { x: LIB_X, b: _ } => {
            19
        },
        S { x: ALIAS_LIB_Y, b: _ } => {
            42
        },
        _ => 9999,
    };
    
    assert_eq(c, 42);

    let s = StructWithConstNames {
        MOD_X,
        LOCAL_X,
        LIB_X,
        ALIAS_LIB_Y,
    };

    let c = match return_me(s) {
        StructWithConstNames { MOD_X: 0u64, LOCAL_X: _, LIB_X: _, ALIAS_LIB_Y: _ } => {
            20
        },
        StructWithConstNames { MOD_X: MOD_X, LOCAL_X: LOCAL_X, LIB_X: LIB_X, ALIAS_LIB_Y: ALIAS_LIB_Y } => {
            42
        },
        _ => 9999,
    };

    let s = StructWithConstNames {
        MOD_X: 0,
        LOCAL_X: 0,
        LIB_X: 0,
        ALIAS_LIB_Y: 0,
    };

    let c = match return_me(s) {
        StructWithConstNames { MOD_X: MOD_X, LOCAL_X: LOCAL_X, LIB_X: LIB_X, ALIAS_LIB_Y: ALIAS_LIB_Y } => {
            21
        },
        StructWithConstNames { MOD_X: 0u64, LOCAL_X: 0u64, LIB_X: 0u64, ALIAS_LIB_Y: 0u64 } => {
            42
        },
        _ => 10000,
    };
    
    assert_eq(c, 42);
}
