script;

use std::{assert::assert, hash::sha256, option::Option, revert::revert, vec::Vec};

struct SimpleStruct {
    x: u32,
    y: b256,
}

enum SimpleEnum {
    X: (),
    Y: b256,
    Z: (b256, b256),
}

fn main() -> bool {
    let number0 = 0u16;
    let number1 = 1u16;
    let number2 = 2u16;
    let number3 = 3u16;
    let number4 = 4u16;
    let number5 = 5u16;
    let number6 = 6u16;
    let number7 = 7u16;
    let number8 = 8u16;
    let number9 = 9u16;

    let b0 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let b1 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let b2 = 0x0000000000000000000000000000000000000000000000000000000000000002;
    let b3 = 0x0000000000000000000000000000000000000000000000000000000000000003;
    let b4 = 0x0000000000000000000000000000000000000000000000000000000000000004;
    let b5 = 0x0000000000000000000000000000000000000000000000000000000000000005;
    let b6 = 0x0000000000000000000000000000000000000000000000000000000000000006;
    let b7 = 0x0000000000000000000000000000000000000000000000000000000000000007;
    let b8 = 0x0000000000000000000000000000000000000000000000000000000000000008;
    let b9 = 0x0000000000000000000000000000000000000000000000000000000000000009;
    // test u8s
    test_vector([
        0u8,
        1u8,
        2u8,
        3u8,
        4u8,
        5u8,
        6u8,
        7u8,
        8u8,
        9u8,
    ]);
    // b256s
    test_vector([
        b0,
        b1,
        b2,
        b3,
        b4,
        b5,
        b6,
        b7,
        b8,
        b9,
    ]);
    // structs
    test_vector([
        SimpleStruct { x: 0, y: b0 },
        SimpleStruct { x: 1, y: b1 },
        SimpleStruct { x: 2, y: b2 },
        SimpleStruct { x: 3, y: b3 },
        SimpleStruct { x: 4, y: b4 },
        SimpleStruct { x: 5, y: b5 },
        SimpleStruct { x: 6, y: b6 },
        SimpleStruct { x: 8, y: b7 },
        SimpleStruct { x: 9, y: b8 },
        SimpleStruct { x: 10, y: b9 },
    ]);
    test_vector([
        SimpleEnum::X(),
        SimpleEnum::Y(b0),
        SimpleEnum::Z((b1, b0)),
        SimpleEnum::Y(b1),
        SimpleEnum::Y(b2),
        SimpleEnum::Y(b3),
        SimpleEnum::Y(b4),
        SimpleEnum::Z((b8, b8)),
        SimpleEnum::Z((b9, b9)),
        SimpleEnum::Z((b1, b1)),
    ]);
    test_vector([
        (number0, b0),
        (number1, b1),
        (number2, b2),
        (number3, b3),
        (number4, b4),
        (number5, b5),
        (number6, b6),
        (number7, b7),
        (number8, b8),
        (number9, b9),
    ]);
    test_vector([
        "aaaa",
        "bbbb",
        "cccc",
        "dddd",
        "eeee",
        "ffff",
        "gggg",
        "hhhh",
        "iiii",
        "jjjj",
    ]);
    test_vector([
        [0, 0, 0],
        [0, 0, 1],
        [0, 0, 2],
        [0, 1, 0],
        [0, 1, 1],
        [0, 1, 2],
        [1, 0, 0],
        [1, 0, 1],
        [1, 0, 2],
        [1, 1, 0],
    ]);
    test_vector_with_capacity_u64();
    true
}

// TODO remove me when we have trait constraints
fn eq<T>(v1: T, v2: T) -> bool {
    asm(r1: v1, r2: v2, r3) {
        eq r3 r2 r1;
        r3: bool
    }
}

fn test_vector<T>(v: [T; 10]) {
    let mut vector = ~Vec::new();

    require(vector.len() == 0, "Vector was initialized to a non-empty state");
    require(vector.capacity() == 0, "Vector was initialized with non-zero capacity");
    require(vector.is_empty(), "Vector was initialized to a non-empty state");

    // check that test values are all distinct
    let mut x = 0;
    while x < 10 {
        let mut y = 0;
        while y < 10 {
            if x != y {
                require(!eq(v[x], v[y]), "Two test values of the same value provided to vector test");
            }
        }
    }

    vector.push(v[0]);
    vector.push(v[1]);
    vector.push(v[2]);
    vector.push(v[3]);
    vector.push(v[4]);

    require(vector.len() == 5, "After pushing five values, vector did not have a length of five");
    require(vector.capacity() == 8, "After pushing five values, vector did not have a capacity of 8");
    require(vector.is_empty() == false, "After pushing five values, vector was considered empty");

    match vector.get(0) {
        Option::Some(val) => require(eq(val, v[0]), "First value of vector was not first test value"),
        Option::None => require(false, "After pushing five values, vector[0] was empty"),
    };
    require(vector.get(9).is_none(), "sixth value of vec was not empty after pushing only five values");

    // Push after get
    vector.push(v[5]);
    vector.push(v[6]);
    vector.push(v[7]);
    vector.push(v[8]);

    match vector.get(4) {
        Option::Some(val) => require(eq(val, v[4]), "fourth value in vector was not the fourth test value"),
        Option::None => require(false, "After pushing nine values, vector[4] was empty"),
    }

    match vector.get(8) {
        Option::Some(val) => require(eq(val, v[8]), "ninth value was not the ninth test value"),
        Option::None => require(false, "After pushing nine values, vector[8] was empty"),
    }
    require(vector.get(9).is_none(), "tenth value of vec was not empty after pushing only nine values");

    vector.clear();

    // Empty after clear
    require(vector.len() == 0, "Vector had non-zero length after clearing");
    require(vector.capacity() == 16, "Vector did not have cap of 16 after clearning 9 values");
    require(vector.is_empty() == true, "Vector was not empty after clearing");

    require(vector.get(0).is_none(), "After clearing, vector[0] should be empty");

    // Make sure pushing again after clear() works
    vector.push(v[0]);
    vector.push(v[1]);
    vector.push(v[2]);
    vector.push(v[3]);
    vector.push(v[4]);

    require(vector.len() == 5, "after pushing five elements, vector len was not 5");
    require(vector.capacity() == 16, "vector cap should have remained 16 after pushing 5 elems");
    require(vector.is_empty() == false, "after pushing five elements, vector should have been empty");

    match vector.get(4) {
        Option::Some(val) => require(eq(val, v[4]), "vector[4] should be v[4]"),
        Option::None => require(false, "after pushing after clearning, vector[4] was empty"),
    }

    // Out of bounds access
    require(vector.get(5).is_none(), "Getting the fifth element in a vec of len 5 should not have worked"); 

    // Remove the first
    let val = vector.remove(0);
    require(eq(val, v[0]), "first element removed from vec was not v[0]");
    require(vector.len() == 4, "length of vector after removing one element did not shrink to 4");
    require(vector.capacity() == 16, "vec cap should stay 16");

    // Remove the last
    let val = vector.remove(3);
    require(eq(val, v[4]), "last elem popped was not v[4]");
    require(vector.len() == 3, "vector len should shrink to 3 after removing a value");
    require(vector.capacity() == 16, "vector capacity should remain 16");

    // Remove the middle
    let val = vector.remove(1);
    require(eq(val, v[2]), "middle elem should be v[2]");
    require(vector.len() == 2, "vec len should shrink to 2");
    require(vector.capacity() == 16, "vec cap needs to be 16");

    // Check what's left
    match vector.get(0) {
        Option::Some(val) => require(eq(val, v[1]), "remaining value should be v[1]"),
        Option::None => require(false, "should have been a remaining value"),
    }

    // Check what's left
    match vector.get(1) {
        Option::Some(val) => require(eq(val, v[3]), "remaining value should be v[3]"),
        Option::None => require(false, "should have been a remaining value"),
    }

    // Renew a `Vec` instead of `vector.clear()` to test the change of capacity after `insert`
    let mut vector = ~Vec::new();

    // Insert to empty
    vector.insert(0, v[2]);
    require(vector.len() == 1, "vector len was not 1 after inserting one value");
    require(vector.capacity() == 1, "vector cap did not update correctly after inserting one value");
    match vector.get(0) {
        Option::Some(val) => require(eq(val, v[2]), "inserted value should be v[2]"),
        Option::None => require(false, "after inserting, value was not v[2]"),
    }

    // Insert at the first
    vector.insert(0, v[0]);
    require(vector.len() == 2, "vector len was not 2 after inserting two values");
    require(vector.capacity() == 2, "vector cap did not update correctly after inserting two values");
    match vector.get(0) {
        Option::Some(val) => require(eq(val, v[0]), "inserted value should be v[0]"),
        Option::None => require(false, "after inserting, value was not v[0]"),
    }
    match vector.get(1) {
        Option::Some(val) => require(eq(val, v[2]), "inserted value at [1] should be v[2]"),
        Option::None => require(false, "after inserting twice, value was not v[2]"),
    }

    // Insert at the middle
    vector.insert(1, v[1]);
    require(vector.len() == 3, "insert middle: vec len was not 3");
    require(vector.capacity() == 4, "insert middle: cap update failed");

    match vector.get(0) {
        Option::Some(val) => require(eq(val, v[0]), "insert: 0 should be v[0]"),
        Option::None => require(false, "insert: vec was empty at pos 0"),
    }

    match vector.get(1) {
        Option::Some(val) => require(eq(val, v[1]), "insert: 1 should be v[1]"),
        Option::None => require(false, "insert: vec was empty at pos 1"),
    }

    match vector.get(2) {
        Option::Some(val) => require(eq(val, v[2]), "insert: 2 should be v[2]"),
        Option::None => require(false, "insert: vec was empty at pos 2"),
    }
    require(vector.get(3).is_none(), "after inserting 3 values, getting the fourth index should be None");

    // Insert at the last
    vector.insert(3, v[3]);
    require(vector.len() == 4, "after inserting last elem, len should be 3");
    require(vector.capacity() == 4, "after inserting 3 elements, cap should be 4");
    match vector.get(0) {
        Option::Some(val) => require(eq(val, v[0]), "insert: 0 should be v[0]"),
        Option::None => require(false, "insert: vec was empty at pos 0"),
    }

    match vector.get(1) {
        Option::Some(val) => require(eq(val, v[1]), "insert: 1 should be v[1]"),
        Option::None => require(false, "insert: vec was empty at pos 1"),
    }

    match vector.get(2) {
        Option::Some(val) => require(eq(val, v[2]), "insert: 2 should be v[2]"),
        Option::None => require(false, "insert: vec was empty at pos 2"),
    }
    match vector.get(3) {
        Option::Some(val) => require(eq(val, v[3]), "insert: 3 should be v[3]"),
        Option::None => require(false, "insert: vec was empty at pos 3"),
    }
    require(vector.get(4).is_none(), "after inserting 4 values, getting the fourth index should be None");

    // Test for `pop`
    vector.clear();
    vector.push(v[0]);
    vector.push(v[1]);
    require(vector.len() == 2, "pop: len should be 2");
    require(vector.capacity() == 4, "pop: cap should be 4");

    match vector.pop() {
        Option::Some(val) => require(eq(val, v[1]), "pop: val should be v[1]"),
        Option::None => require(false, "after popping, data was none"),
    }
    require(vector.len() == 1, "pop: vec len 1");
    require(vector.capacity() == 4, "pop: vec cap 4");

    match vector.pop() {
        Option::Some(val) => require(eq(val, v[0]), "pop: val should be v[0]"),
        Option::None => require(false, "after popping, data was none"),
    }

    assert(vector.len() == 0);
    assert(vector.capacity() == 4);

    assert(vector.pop().is_none());
}

fn test_vector_with_capacity_u64() {
    let mut vector = ~Vec::with_capacity(8);

    let number0 = 0;
    let number1 = 1;
    let number2 = 2;
    let number3 = 3;
    let number4 = 4;
    let number5 = 5;
    let number6 = 6;
    let number7 = 7;
    let number8 = 8;

    assert(vector.len() == 0);
    assert(vector.capacity() == 8);
    assert(vector.is_empty() == true);

    vector.push(number0);
    vector.push(number1);
    vector.push(number2);
    vector.push(number3);
    vector.push(number4);

    assert(vector.len() == 5);
    assert(vector.capacity() == 8);
    assert(vector.is_empty() == false);

    match vector.get(0) {
        Option::Some(val) => assert(val == number0),
        Option::None => revert(0),
    }

    // Push after get
    vector.push(number5);
    vector.push(number6);
    vector.push(number7);
    vector.push(number8);

    match vector.get(4) {
        Option::Some(val) => assert(val == number4),
        Option::None => revert(0),
    }

    match vector.get(6) {
        Option::Some(val) => assert(val == number6),
        Option::None => revert(0),
    }

    assert(vector.len() == 9);
    assert(vector.capacity() == 16);
    assert(vector.is_empty() == false);

    vector.clear();

    // Empty after clear
    assert(vector.len() == 0);
    assert(vector.capacity() == 16);
    assert(vector.is_empty() == true);

    match vector.get(0) {
        Option::Some(val) => revert(0),
        Option::None => (),
    }

    // Make sure pushing again after clear() works
    vector.push(number0);
    vector.push(number1);
    vector.push(number2);
    vector.push(number3);
    vector.push(number4);

    assert(vector.len() == 5);
    assert(vector.capacity() == 16);
    assert(vector.is_empty() == false);

    match vector.get(4) {
        Option::Some(val) => assert(val == number4),
        Option::None => revert(0),
    }

    // Out of bounds access
    match vector.get(5) {
        Option::Some(val) => revert(0),
        Option::None => (),
    }
}
