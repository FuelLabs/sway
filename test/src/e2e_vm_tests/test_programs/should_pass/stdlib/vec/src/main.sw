script;

use core::ops::*;
use std::{assert::assert, hash::sha256, option::Option, revert::revert, vec::Vec};

// STRUCTURED DATA DEFINITIONS
struct SimpleStruct {
    x: u32,
    y: b256,
}

enum SimpleEnum {
    X: (),
    Y: b256,
    Z: (b256, b256),
}

// EQUALITY TRAIT DEFINITIONS
impl Eq for SimpleStruct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for SimpleEnum {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (SimpleEnum::X, SimpleEnum::X) => true,
            (SimpleEnum::Y(y0), SimpleEnum::Y(y1)) => y0 == y1,
            (SimpleEnum::Z(z0), SimpleEnum::Z(z1)) => z0.0 == z1.0 && z0.1 == z1.1,
            _ => false,
        }
    }
}

impl Eq for (u16, b256) {
    fn eq(self, other: Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl Eq for str[4] {
    fn eq(self, other: Self) -> bool {
        sha256(self) == sha256(other)
    }
}

impl Eq for [u64; 3] {
    fn eq(self, other: Self) -> bool {
        self[0] == other[0] && self[1] == other[1] && self[2] == other[2]
    }
}

// CONSTANTS
const B256_0 = 0x0000000000000000000000000000000000000000000000000000000000000000;
const B256_1 = 0x0000000000000000000000000000000000000000000000000000000000000001;
const B256_2 = 0x0000000000000000000000000000000000000000000000000000000000000002;
const B256_3 = 0x0000000000000000000000000000000000000000000000000000000000000003;
const B256_4 = 0x0000000000000000000000000000000000000000000000000000000000000004;
const B256_5 = 0x0000000000000000000000000000000000000000000000000000000000000005;
const B256_6 = 0x0000000000000000000000000000000000000000000000000000000000000006;
const B256_7 = 0x0000000000000000000000000000000000000000000000000000000000000007;
const B256_8 = 0x0000000000000000000000000000000000000000000000000000000000000008;

// TESTS
fn main() -> bool {
    // test Vec<u8>
    test_vector::<u8>(0_u8, 1_u8, 2_u8, 3_u8, 4_u8, 5_u8, 6_u8, 7_u8, 8_u8);

    // test Vec<b256>
    // test_vector::<b256>(B256_0, B256_1, B256_2, B256_3, B256_4, B256_5, B256_6, B256_7, B256_8);
    
    // // test Vec<SimpleStruct>
    // test_vector::<SimpleStruct>(
    //     SimpleStruct { x: 0_u32, y: B256_0 },
    //     SimpleStruct { x: 1_u32, y: B256_1 },
    //     SimpleStruct { x: 2_u32, y: B256_2 },
    //     SimpleStruct { x: 3_u32, y: B256_3 },
    //     SimpleStruct { x: 4_u32, y: B256_4 },
    //     SimpleStruct { x: 5_u32, y: B256_5 },
    //     SimpleStruct { x: 6_u32, y: B256_6 },
    //     SimpleStruct { x: 7_u32, y: B256_7 },
    //     SimpleStruct { x: 8_u32, y: B256_8 },
    // );

    // // test Vec<SimpleEnum>
    // test_vector::<SimpleEnum>(
    //     SimpleEnum::Y(B256_0),
    //     SimpleEnum::X,
    //     SimpleEnum::Z((B256_1, B256_2)),
    //     SimpleEnum::Y(B256_1),
    //     SimpleEnum::Y(B256_2),
    //     SimpleEnum::Z((B256_3, B256_4)),
    //     SimpleEnum::Z((B256_5, B256_5)),
    //     SimpleEnum::Y(B256_8),
    //     SimpleEnum::X,
    // );

    // // test Vec<u16, b256)>
    // test_vector::<(u16, b256)>(
    //     (0_u16, B256_0),
    //     (1_u16, B256_1),
    //     (2_u16, B256_2),
    //     (3_u16, B256_3),
    //     (4_u16, B256_4),
    //     (5_u16, B256_5),
    //     (6_u16, B256_6),
    //     (7_u16, B256_7),
    //     (8_u16, B256_8),
    // );

    // // test Vec<str[4]>
    // test_vector::<str[4]>(
    //     "fuel",
    //     "john",
    //     "nick",
    //     "adam",
    //     "emma",
    //     "sway",
    //     "gmgn",
    //     "kekw",
    //     "meow",
    // );

    // // test Vec<[u64; 3]>
    // test_vector::<[u64; 3]>(
    //     [0, 1, 2],
    //     [3, 4, 5],
    //     [6, 7, 8],
    //     [9, 10, 11],
    //     [12, 13, 14],
    //     [15, 16, 17],
    //     [18, 19, 20],
    //     [21, 22, 23],
    //     [24, 25, 26],
    // );

    true
}

fn assert_bounds<T>(ref mut vector: Vec<T>, expected_len: u64, expected_cap: u64) {
    assert(vector.len() == expected_len);
    assert(vector.capacity() == expected_cap);
    assert(!vector.is_empty() || expected_len == 0);
}

fn test_vector<T>(
    value0: T,
    value1: T,
    value2: T,
    value3: T,
    value4: T,
    value5: T,
    value6: T,
    value7: T,
    value8: T,
) where T: Eq {
    without_capacity(value0, value1, value2, value3, value4, value5, value6, value7, value8);
    with_capacity(value0, value1, value2, value3, value4, value5, value6,value7, value8);
}

fn without_capacity<T>(
    value0: T,
    value1: T,
    value2: T,
    value3: T,
    value4: T,
    value5: T,
    value6: T,
    value7: T,
    value8: T,
) where T: Eq {
    // create vector
    let mut vector: Vec<T> = Vec::new();

    assert_bounds(vector, 0, 0);

    // push 5 values
    vector.push(value0);
    vector.push(value1);
    vector.push(value2);
    vector.push(value3);
    vector.push(value4);

    assert_bounds(vector, 5, 8);
    assert(vector.get(0).unwrap() == value0);
    assert(vector.get(1).unwrap() == value1);
    assert(vector.get(2).unwrap() == value2);
    assert(vector.get(3).unwrap() == value3);
    assert(vector.get(4).unwrap() == value4);

    // push 4 values
    vector.push(value5);
    vector.push(value6);
    vector.push(value7);
    vector.push(value8);

    assert_bounds(vector, 9, 16);
    assert(vector.get(5).unwrap() == value5);
    assert(vector.get(6).unwrap() == value6);
    assert(vector.get(7).unwrap() == value7);
    assert(vector.get(8).unwrap() == value8);

    // clear the vector
    vector.clear();

    assert_bounds(vector, 0, 16);
    assert(vector.get(0).is_none());

    // push 5 elements
    vector.push(value0);
    vector.push(value1);
    vector.push(value2);
    vector.push(value3);
    vector.push(value4);

    assert_bounds(vector, 5, 16);
    assert(vector.get(0).unwrap() == value0);
    assert(vector.get(1).unwrap() == value1);
    assert(vector.get(2).unwrap() == value2);
    assert(vector.get(3).unwrap() == value3);
    assert(vector.get(4).unwrap() == value4);
    assert(vector.get(5).is_none());

    // remove first
    assert(vector.remove(0) == value0);

    assert_bounds(vector, 4, 16);
    assert(vector.get(0).unwrap() == value1);
    assert(vector.get(1).unwrap() == value2);
    assert(vector.get(2).unwrap() == value3);
    assert(vector.get(3).unwrap() == value4);

    // remove last
    assert(vector.remove(3) == value4);

    assert_bounds(vector, 3, 16);
    assert(vector.get(0).unwrap() == value1);
    assert(vector.get(1).unwrap() == value2);
    assert(vector.get(2).unwrap() == value3);

    // remove middle
    assert(vector.remove(1) == value2);

    assert_bounds(vector, 2, 16);
    assert(vector.get(0).unwrap() == value1);
    assert(vector.get(1).unwrap() == value3);

    // alloc new vec
    let mut vector = Vec::new();

    // insert into empty
    vector.insert(0, value2);

    assert_bounds(vector, 1, 1);
    assert(vector.get(0).unwrap() == value2);

    // insert at first
    vector.insert(0, value0);

    assert_bounds(vector, 2, 2);
    assert(vector.get(0).unwrap() == value0);
    assert(vector.get(1).unwrap() == value2);

    // insert at middle
    vector.insert(1, value1);

    assert_bounds(vector, 3, 4);
    assert(vector.get(0).unwrap() == value0);
    assert(vector.get(1).unwrap() == value1);
    assert(vector.get(2).unwrap() == value2);

    // insert at last
    vector.insert(3, value3);

    assert_bounds(vector, 4, 4);
    assert(vector.get(0).unwrap() == value0);
    assert(vector.get(1).unwrap() == value1);
    assert(vector.get(2).unwrap() == value2);
    assert(vector.get(3).unwrap() == value3);

    // test pop
    vector.clear();
    vector.push(value0);
    vector.push(value1);

    assert_bounds(vector, 2, 4);

    // pop
    assert(vector.pop().unwrap() == value1);

    assert_bounds(vector, 1, 4);

    // pop
    assert(vector.pop().unwrap() == value0);

    assert_bounds(vector, 0, 4);

    // pop empty
    assert(vector.pop().is_none());

    assert_bounds(vector, 0, 4);

    // test for set
    vector.clear();
    vector.push(value0);
    vector.push(value1);
    vector.push(value2);

    assert_bounds(vector, 3, 4);
    assert(vector.get(0).unwrap() == value0);
    assert(vector.get(1).unwrap() == value1);
    assert(vector.get(2).unwrap() == value2);

    // set first
    vector.set(0, value3);

    assert_bounds(vector, 3, 4);
    assert(vector.get(0).unwrap() == value3);
    assert(vector.get(1).unwrap() == value1);
    assert(vector.get(2).unwrap() == value2);

    // set middle
    vector.set(1, value4);

    assert_bounds(vector, 3, 4);
    assert(vector.get(0).unwrap() == value3);
    assert(vector.get(1).unwrap() == value4);
    assert(vector.get(2).unwrap() == value2);

    // set last
    vector.set(2, value5);

    assert_bounds(vector, 3, 4);
    assert(vector.get(0).unwrap() == value3);
    assert(vector.get(1).unwrap() == value4);
    assert(vector.get(2).unwrap() == value5);
}

fn with_capacity<T>(
    value0: T,
    value1: T,
    value2: T,
    value3: T,
    value4: T,
    value5: T,
    value6: T,
    value7: T,
    value8: T
) where T: Eq {
    // create vector with capacity
    let mut vector = Vec::with_capacity(8);

    assert_bounds(vector, 0, 8);

    // push 5
    vector.push(value0);
    vector.push(value1);
    vector.push(value2);
    vector.push(value3);
    vector.push(value4);

    assert_bounds(vector, 5, 8);
    assert(vector.get(0).unwrap() == value0);
    assert(vector.get(1).unwrap() == value1);
    assert(vector.get(2).unwrap() == value2);
    assert(vector.get(3).unwrap() == value3);
    assert(vector.get(4).unwrap() == value4);

    // push 4
    vector.push(value5);
    vector.push(value6);
    vector.push(value7);
    vector.push(value8);

    assert_bounds(vector, 9, 16);
    assert(vector.get(5).unwrap() == value5);
    assert(vector.get(6).unwrap() == value6);
    assert(vector.get(7).unwrap() == value7);
    assert(vector.get(8).unwrap() == value8);

    // clear
    vector.clear();

    assert_bounds(vector, 0, 16);
    assert(vector.get(0).is_none());

    // push 5
    vector.push(value0);
    vector.push(value1);
    vector.push(value2);
    vector.push(value3);
    vector.push(value4);

    assert_bounds(vector, 5, 16);
    assert(vector.get(0).unwrap() == value0);
    assert(vector.get(1).unwrap() == value1);
    assert(vector.get(2).unwrap() == value2);
    assert(vector.get(3).unwrap() == value3);
    assert(vector.get(4).unwrap() == value4);
    assert(vector.get(5).is_none());
}
