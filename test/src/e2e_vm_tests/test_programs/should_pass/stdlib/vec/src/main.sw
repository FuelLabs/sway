script;

use std::{assert::assert, option::Option, revert::revert, vec::Vec};

fn main() -> bool {
    test_vector_new();
    test_vector_new_b256();
    test_vector_with_capacity();
    true
}

fn test_vector_new() {
    let mut v: Vec<u8> = ~Vec::new::<u8>();

    assert(v.len() == 0);
    assert(v.capacity() == 0);
    assert(v.is_empty() == true);

    v.push(0);
    v.push(1);
    v.push(2);
    v.push(3);
    v.push(4);

    assert(v.len() == 5);
    assert(v.capacity() == 8);
    assert(v.is_empty() == false);

    match v.get(0) {
        Option::Some(val) => assert(val == 0),
        Option::None => revert(0),
    }

    // Push after get
    v.push(5);
    v.push(6);
    v.push(7);
    v.push(8);

    match v.get(4) {
        Option::Some(val) => assert(val == 4),
        Option::None => revert(0),
    }

    match v.get(6) {
        Option::Some(val) => assert(val == 6),
        Option::None => revert(0),
    }

    assert(v.len() == 9);
    assert(v.capacity() == 16);
    assert(v.is_empty() == false);

    // Test after capacity change
    match v.get(4) {
        Option::Some(val) => assert(val == 4),
        Option::None => revert(0),
    }

    match v.get(6) {
        Option::Some(val) => assert(val == 6),
        Option::None => revert(0),
    }

    v.clear();

    // Empty after clear
    assert(v.len() == 0);
    assert(v.capacity() == 16);
    assert(v.is_empty() == true);

    match v.get(0) {
        Option::Some(val) => revert(0),
        Option::None => {},
    }

    // Make sure pushing again after clear() works
    v.push(0);
    v.push(1);
    v.push(2);
    v.push(3);
    v.push(4);

    assert(v.len() == 5);
    assert(v.capacity() == 16);
    assert(v.is_empty() == false);

    match v.get(4) {
        Option::Some(val) => assert(val == 4),
        Option::None => revert(0),
    }

    // Out of bounds access
    match v.get(5) {
        Option::Some(val) => revert(0),
        Option::None => {},
    }
}

fn test_vector_with_capacity() {
    let mut v: Vec<u64> = ~Vec::with_capacity::<u64>(8);

    assert(v.len() == 0);
    assert(v.capacity() == 8);
    assert(v.is_empty() == true);

    v.push(0);
    v.push(1);
    v.push(2);
    v.push(3);
    v.push(4);

    assert(v.len() == 5);
    assert(v.capacity() == 8);
    assert(v.is_empty() == false);

    match v.get(0) {
        Option::Some(val) => assert(val == 0),
        Option::None => revert(0),
    }

    // Push after get
    v.push(5);
    v.push(6);
    v.push(7);
    v.push(8);

    match v.get(4) {
        Option::Some(val) => assert(val == 4),
        Option::None => revert(0),
    }

    match v.get(6) {
        Option::Some(val) => assert(val == 6),
        Option::None => revert(0),
    }

    assert(v.len() == 9);
    assert(v.capacity() == 16);
    assert(v.is_empty() == false);

    v.clear();

    // Empty after clear
    assert(v.len() == 0);
    assert(v.capacity() == 16);
    assert(v.is_empty() == true);

    match v.get(0) {
        Option::Some(val) => revert(0),
        Option::None => {},
    }

    // Make sure pushing again after clear() works
    v.push(0);
    v.push(1);
    v.push(2);
    v.push(3);
    v.push(4);

    assert(v.len() == 5);
    assert(v.capacity() == 16);
    assert(v.is_empty() == false);

    match v.get(4) {
        Option::Some(val) => assert(val == 4),
        Option::None => revert(0),
    }

    // Out of bounds access
    match v.get(5) {
        Option::Some(val) => revert(0),
        Option::None => {},
    }
}

fn test_vector_new_b256() {
    let mut v: Vec<b256> = ~Vec::new::<b256>();

    assert(v.len() == 0);
    assert(v.capacity() == 0);
    assert(v.is_empty() == true);

    v.push(0x0000000000000000000000000000000000000000000000000000000000000000);
    v.push(0x0000000000000000000000000000000000000000000000000000000000000001);
    v.push(0x0000000000000000000000000000000000000000000000000000000000000002);
    v.push(0x0000000000000000000000000000000000000000000000000000000000000003);
    v.push(0x0000000000000000000000000000000000000000000000000000000000000004);

    assert(v.len() == 5);
    assert(v.capacity() == 8);
    assert(v.is_empty() == false);

    match v.get(0) {
        Option::Some(val) => assert(val == 0x0000000000000000000000000000000000000000000000000000000000000000),
        Option::None => revert(0),
    }

    // Push after get
    v.push(0x0000000000000000000000000000000000000000000000000000000000000005);
    v.push(0x0000000000000000000000000000000000000000000000000000000000000006);
    v.push(0x0000000000000000000000000000000000000000000000000000000000000007);
    v.push(0x0000000000000000000000000000000000000000000000000000000000000008);

    match v.get(4) {
        Option::Some(val) => assert(val == 0x0000000000000000000000000000000000000000000000000000000000000004),
        Option::None => revert(0),
    }

    match v.get(6) {
        Option::Some(val) => assert(val == 0x0000000000000000000000000000000000000000000000000000000000000006),
        Option::None => revert(0),
    }

    assert(v.len() == 9);
    assert(v.capacity() == 16);
    assert(v.is_empty() == false);

    // Test after capacity change
    match v.get(4) {
        Option::Some(val) => assert(val == 0x0000000000000000000000000000000000000000000000000000000000000004),
        Option::None => revert(0),
    }

    match v.get(6) {
        Option::Some(val) => assert(val == 0x0000000000000000000000000000000000000000000000000000000000000006),
        Option::None => revert(0),
    }

    v.clear();

    // Empty after clear
    assert(v.len() == 0);
    assert(v.capacity() == 16);
    assert(v.is_empty() == true);

    match v.get(0) {
        Option::Some(val) => revert(0),
        Option::None => {},
    }

    // Make sure pushing again after clear() works
    v.push(0x0000000000000000000000000000000000000000000000000000000000000000);
    v.push(0x0000000000000000000000000000000000000000000000000000000000000001);
    v.push(0x0000000000000000000000000000000000000000000000000000000000000002);
    v.push(0x0000000000000000000000000000000000000000000000000000000000000003);
    v.push(0x0000000000000000000000000000000000000000000000000000000000000004);

    assert(v.len() == 5);
    assert(v.capacity() == 16);
    assert(v.is_empty() == false);

    match v.get(4) {
        Option::Some(val) => assert(val == 0x0000000000000000000000000000000000000000000000000000000000000004),
        Option::None => revert(0),
    }

    // Out of bounds access
    match v.get(5) {
        Option::Some(val) => revert(0),
        Option::None => {},
    }
}
