//! Additional aggregate-initialization constellations that combine structs,
//! tuples and arrays in various nestings. These stress the recursive lowering
//! (structs of arrays, arrays of structs, tuples of arrays of structs, etc.).
library;

use ::types::*;

struct WithArray {
    head: u64,
    body: [u64; 4],
    tail: u64,
}

impl PartialEq for WithArray {
    fn eq(self, other: Self) -> bool {
        let mut i = 0;
        let mut body_eq = true;
        while i < 4 {
            if self.body[i] != other.body[i] {
                body_eq = false;
            }
            i += 1;
        }
        self.head == other.head && body_eq && self.tail == other.tail
    }
}
impl Eq for WithArray {}

// Struct containing an array field.
#[test]
fn test_struct_with_array() {
    struct_with_array();
}

#[inline(never)]
pub fn struct_with_array() {
    let s = WithArray {
        head: 1,
        body: [2, 3, 4, 5],
        tail: 6,
    };

    assert_eq(s.head, 1);
    assert_eq(s.body[0], 2);
    assert_eq(s.body[1], 3);
    assert_eq(s.body[2], 4);
    assert_eq(s.body[3], 5);
    assert_eq(s.tail, 6);
}

// Struct containing a repeat array field.
#[test]
fn test_struct_with_repeat_array() {
    struct_with_repeat_array();
}

#[inline(never)]
pub fn struct_with_repeat_array() {
    let s = WithArray {
        head: 1,
        body: [9; 4],
        tail: 2,
    };

    assert_eq(s.head, 1);
    let mut i = 0;
    while i < 4 {
        assert_eq(s.body[i], 9);
        i += 1;
    }
    assert_eq(s.tail, 2);
}

// Array of tuples.
#[test]
fn test_array_of_tuples() {
    array_of_tuples();
}

#[inline(never)]
pub fn array_of_tuples() {
    let a = [(1u64, true), (2u64, false), (3u64, true)];

    assert_eq(a[0].0, 1);
    assert_eq(a[0].1, true);
    assert_eq(a[1].0, 2);
    assert_eq(a[1].1, false);
    assert_eq(a[2].0, 3);
    assert_eq(a[2].1, true);
}

// Array of structs (non-repeat, distinct values).
#[test]
fn test_array_of_structs_distinct() {
    array_of_structs_distinct();
}

#[inline(never)]
pub fn array_of_structs_distinct() {
    let a = [
        NoNesting {
            a: 1,
            b: true,
            c: 1u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 2,
            b: false,
            c: 2u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 3,
            b: true,
            c: 3u256,
            d: b256::zero(),
            u: (),
        },
    ];

    assert_no_nesting(a[0], 1, true, 1u256, b256::zero());
    assert_no_nesting(a[1], 2, false, 2u256, b256::zero());
    assert_no_nesting(a[2], 3, true, 3u256, b256::zero());
}

// Tuple of an array of structs.
#[test]
fn test_tuple_of_array_of_structs() {
    tuple_of_array_of_structs();
}

#[inline(never)]
pub fn tuple_of_array_of_structs() {
    let t = (
        100u64,
        [create_some_simple(1), create_some_simple(2)],
        true,
    );

    assert_eq(t.0, 100);
    assert_simple(t.1[0], 1, 2, true, 3u256);
    assert_simple(t.1[1], 2, 2, true, 3u256);
    assert_eq(t.2, true);
}

// Struct nested in a struct nested in a struct, with a mix of values.
#[test]
fn test_deep_struct_nesting() {
    deep_struct_nesting();
}

#[inline(never)]
pub fn deep_struct_nesting() {
    let s = Struct {
        x: 10,
        simple: Simple {
            a: 1,
            b: 2,
            c: true,
            d: 3u256,
        },
        b: false,
    };

    assert_eq(s.x, 10);
    assert_simple(s.simple, 1, 2, true, 3u256);
    assert_eq(s.b, false);
}

// Array of u256 (large scalars), non-repeat.
#[test]
fn test_array_of_u256() {
    array_of_u256();
}

#[inline(never)]
pub fn array_of_u256() {
    let a = [1u256, 2u256, 3u256, 0u256];

    assert_eq(a[0], 1u256);
    assert_eq(a[1], 2u256);
    assert_eq(a[2], 3u256);
    assert_eq(a[3], 0u256);
}

// A large repeat array of a struct element (triggers the loop path for a
// non-scalar repeated value).
#[test]
fn test_large_repeat_array_of_struct() {
    large_repeat_array_of_struct();
}

#[inline(never)]
pub fn large_repeat_array_of_struct() {
    let a = [create_some_simple(7); 8];

    let mut i = 0;
    while i < 8 {
        assert_simple(a[i], 7, 2, true, 3u256);
        i += 1;
    }
}
