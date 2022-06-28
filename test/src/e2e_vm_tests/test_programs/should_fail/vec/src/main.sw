script;

use std::{assert::assert, hash::sha256, option::Option, revert::revert, vec::Vec};

struct SimpleStruct {
    x: u32,
    y: b256,
}

enum SimpleEnum {
    X: (),
    Y: b256,
    Z: (b256,
    b256), 
}

fn main() -> bool {
    test_vector_new_u8();
    true
}

fn test_vector_new_u8() {
    let mut vector = ~Vec::new();

    let number0 = 0u8;
    let number1 = 1u8;
    let number2 = 2u8;
    let number3 = 3u8;
    let number4 = 4u8;
    let number5 = 5u8;
    let number6 = 6u8;
    let number7 = 7u8;
    let number8 = 8u8;

    assert(vector.len() == 0);
    assert(vector.capacity() == 0);
    assert(vector.is_empty());

    vector.push(number0);
    vector.push(number1);
    vector.push(number2);
    vector.push(number3);
    vector.push(number4);
    vector.push(false);

    assert(vector.len() == 5);
    assert(vector.capacity() == 8);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => assert(val == number0), Option::None => revert(0), 
    }

    // Push after get
    vector.push(number5);
    vector.push(number6);
    vector.push(number7);
    vector.push(number8);
    vector.push("this should break it 1");

    match vector.get(4) {
        Option::Some(val) => assert(val == number4), Option::None => revert(0), 
    }

    match vector.get(number6) {
        Option::Some(val) => assert(val == number6), Option::None => revert(0), 
    }

    assert(vector.len() == 9);
    assert(vector.capacity() == 16);
    assert(!vector.is_empty());

    // Test after capacity change
    match vector.get(4) {
        Option::Some(val) => assert(val == number4), Option::None => revert(0), 
    }

    match vector.get(6) {
        Option::Some(val) => assert(val == number6), Option::None => revert(0), 
    }

    vector.clear();

    // Empty after clear
    assert(vector.len() == 0);
    assert(vector.capacity() == 16);
    assert(vector.is_empty() == true);

    match vector.get(0) {
        Option::Some(val) => revert(0), Option::None => (), 
    }

    // Make sure pushing again after clear() works
    vector.push(number0);
    vector.push(number1);
    vector.push(number2);
    vector.push(number3);
    vector.push(number4);
    vector.push("this should break it 2");

    assert(vector.len() == 5);
    assert(vector.capacity() == 16);
    assert(vector.is_empty() == false);

    match vector.get(4) {
        Option::Some(val) => assert(val == number4), Option::None => revert(0), 
    }

    // Out of bounds access
    match vector.get(5) {
        Option::Some(val) => revert(0), Option::None => (), 
    }
}
