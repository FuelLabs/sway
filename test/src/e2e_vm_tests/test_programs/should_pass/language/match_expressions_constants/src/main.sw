script;

mod lib;

use ::lib::LIB_X;
use ::lib::LIB_Y as ALIAS_LIB_Y;

const MOD_X = 8;

const NUMBER_1: u64 = 7;
const NUMBER_2: u64 = 14;
const NUMBER_3: u64 = 5;

const TRUE: bool = true;
const FALSE: bool = false;

fn main() -> u64 {
    let a = match return_me(NUMBER_3) {
        NUMBER_1 => 1,
        NUMBER_2 => 1,
        NUMBER_3 => 42,
        other => other,
    };

    assert(a == 42);

    let b = match return_me(TRUE) {
        TRUE => 42,
        FALSE => 1,
    };
    
    assert(b == 42);

    const MAIN_X = 13;

    let c = match return_me(MOD_X) {
        MOD_X => {
            42
        },
        MAIN_X => {
            1
        },
        LIB_X => {
            1
        },
        ALIAS_LIB_Y => {
            1
        },
        _ => 9999,
    };
    
    assert(c == 42);

    let c = match return_me(MAIN_X) {
        MOD_X => {
            1
        },
        MAIN_X => {
            42
        },
        LIB_X => {
            1
        },
        ALIAS_LIB_Y => {
            1
        },
        _ => 9999,
    };
    
    assert(c == 42);

    let c = match return_me(LIB_X) {
        MOD_X => {
            1
        },
        MAIN_X => {
            1
        },
        LIB_X => {
            42
        },
        ALIAS_LIB_Y => {
            1
        },
        _ => 9999,
    };
    
    assert(c == 42);

    let c = match return_me(ALIAS_LIB_Y) {
        MOD_X => {
            1
        },
        MAIN_X => {
            1
        },
        LIB_X => {
            1
        },
        ALIAS_LIB_Y => {
            42
        },
        _ => 9999,
    };
    
    assert(c == 42);

    let c = match return_me(MOD_X) {
        MOD_X | MAIN_X | LIB_X => {
            42
        },
        ALIAS_LIB_Y => {
            1
        },
        _ => 9999,
    };
    
    assert(c == 42);

    let c = match return_me(MAIN_X) {
        MOD_X | MAIN_X | LIB_X => {
            42
        },
        ALIAS_LIB_Y => {
            1
        },
        _ => 9999,
    };
    
    assert(c == 42);

    let c = match return_me(LIB_X) {
        MOD_X | MAIN_X | LIB_X => {
            42
        },
        ALIAS_LIB_Y => {
            1
        },
        _ => 9999,
    };
    
    assert(c == 42);

    let c = match return_me(ALIAS_LIB_Y) {
        MOD_X | MAIN_X | LIB_X => {
            1
        },
        ALIAS_LIB_Y => {
            42
        },
        _ => 9999,
    };
    
    assert(c == 42);

    a + b + c
}

fn return_me<T>(x: T) -> T {
    x
}