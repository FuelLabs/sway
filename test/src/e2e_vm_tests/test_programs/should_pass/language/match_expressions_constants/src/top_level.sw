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

pub fn test() {
    let a = match return_me(NUMBER_3) {
        NUMBER_1 => 1,
        NUMBER_2 => 2,
        NUMBER_3 => 42,
        other => other,
    };

    assert_eq(a, 42);

    let b = match return_me(TRUE) {
        TRUE => 42,
        FALSE => 3,
    };
    
    assert_eq(b, 42);

    const LOCAL_X: u64 = 13;

    let c = match return_me(MOD_X) {
        MOD_X => {
            42
        },
        LOCAL_X => {
            4
        },
        LIB_X => {
            5
        },
        ALIAS_LIB_Y => {
            6
        },
        _ => 1111,
    };
    
    assert_eq(c, 42);

    let c = match return_me(LOCAL_X) {
        MOD_X => {
            7
        },
        LOCAL_X => {
            42
        },
        LIB_X => {
            8
        },
        ALIAS_LIB_Y => {
            9
        },
        _ => 2222,
    };
    
    assert_eq(c, 42);

    let c = match return_me(LIB_X) {
        MOD_X => {
            10
        },
        LOCAL_X => {
            11
        },
        LIB_X => {
            42
        },
        ALIAS_LIB_Y => {
            12
        },
        _ => 3333,
    };
    
    assert_eq(c, 42);

    let c = match return_me(ALIAS_LIB_Y) {
        MOD_X => {
            13
        },
        LOCAL_X => {
            14
        },
        LIB_X => {
            15
        },
        ALIAS_LIB_Y => {
            42
        },
        _ => 4444,
    };
    
    assert_eq(c, 42);

    let c = match return_me(MOD_X) {
        MOD_X | LOCAL_X | LIB_X => {
            42
        },
        ALIAS_LIB_Y => {
            16
        },
        _ => 5555,
    };
    
    assert_eq(c, 42);

    let c = match return_me(LOCAL_X) {
        MOD_X | LOCAL_X | LIB_X => {
            42
        },
        ALIAS_LIB_Y => {
            17
        },
        _ => 6666,
    };
    
    assert_eq(c, 42);

    let c = match return_me(LIB_X) {
        MOD_X | LOCAL_X | LIB_X => {
            42
        },
        ALIAS_LIB_Y => {
            18
        },
        _ => 7777,
    };
    
    assert_eq(c, 42);

    let c = match return_me(ALIAS_LIB_Y) {
        MOD_X | LOCAL_X | LIB_X => {
            19
        },
        ALIAS_LIB_Y => {
            42
        },
        _ => 8888,
    };
    
    assert_eq(c, 42);
}
