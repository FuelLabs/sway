//! Initialization of repeat arrays (`[value; N]`).
//! Covers scalar and aggregate elements, zero and non-zero values,
//! empty arrays, and arrays nested in other aggregates.
library;

use ::types::*;

#[test]
fn test_u64_all_zeros_repeat() {
    u64_all_zeros_repeat();
}

#[inline(never)]
pub fn u64_all_zeros_repeat() {
    let a = [0u64; 10];

    assert_all_elems_equal(a, 0u64);
}

#[test]
fn test_u64_all_42s_repeat() {
    u64_all_42s_repeat();
}

#[inline(never)]
pub fn u64_all_42s_repeat() {
    let a = [42u64; 10];

    assert_all_elems_equal(a, 42u64);
}

#[test]
fn test_no_nesting_all_zeros_repeat() {
    no_nesting_all_zeros_repeat();
}

#[inline(never)]
pub fn no_nesting_all_zeros_repeat() {
    let a = [
        NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        }; 10
    ];

    assert_all_elems_equal(a, NoNesting {
        a: 0,
        b: false,
        c: 0u256,
        d: b256::zero(),
        u: (),
    });
}

#[test]
fn test_no_nesting_not_all_zeros_repeat() {
    no_nesting_not_all_zeros_repeat();
}

#[inline(never)]
pub fn no_nesting_not_all_zeros_repeat() {
    let a = [
        NoNesting {
            a: 42,
            b: true,
            c: 42u256,
            d: b256::zero(),
            u: (),
        }; 10
    ];

    assert_all_elems_equal(a, NoNesting {
        a: 42,
        b: true,
        c: 42u256,
        d: b256::zero(),
        u: (),
    });
}

#[test]
fn test_b256_zero_repeat() {
    b256_zero_repeat();
}

#[inline(never)]
pub fn b256_zero_repeat() {
    let a = [b256::zero(); 10];

    assert_all_elems_equal(a, b256::zero());
}

#[test]
fn test_empty_array_repeat() {
    empty_array_repeat();
}

#[inline(never)]
pub fn empty_array_repeat() {
    let a: [u64; 0] = [42; 0];

    assert_eq(a.len(), 0);
}

#[test]
fn test_empty_array_with_side_effect_repeat() {
    empty_array_with_side_effect_repeat();
}

#[inline(never)]
pub fn empty_array_with_side_effect_repeat() {
    let mut i = 0;
    let a: [u64; 0] = [side_effect(i); 0];

    assert_eq(i, 1);
    assert_eq(a.len(), 0);
}

fn side_effect(ref mut x: u64) -> u64 {
    x += 1;
    x
}

struct SingleFieldStruct {
    pub a: u64,
}

impl PartialEq for SingleFieldStruct {
    fn eq(self, other: Self) -> bool {
        self.a == other.a
    }
}
impl Eq for SingleFieldStruct {}

#[test]
fn test_single_field_struct_not_zero_repeat() {
    single_field_struct_not_zero_repeat();
}

#[inline(never)]
pub fn single_field_struct_not_zero_repeat() {
    let a = [SingleFieldStruct { a: 42 }; 10];

    assert_all_elems_equal(a, SingleFieldStruct { a: 42 });
}

#[test]
fn test_single_field_struct_not_zero_sroa_repeat() {
    single_field_struct_not_zero_sroa_repeat();
}

#[inline(never)]
pub fn single_field_struct_not_zero_sroa_repeat() {
    let a = [SingleFieldStruct { a: 42 }; 10];
    assert_eq(a[1].a, 42);
}

#[test]
fn test_u64_all_zeros_array_nested_in_array_repeat() {
    u64_all_zeros_array_nested_in_array_repeat();
}

#[inline(never)]
pub fn u64_all_zeros_array_nested_in_array_repeat() {
    let a = [[0u64; 10]; 10];
    assert_all_elems_equal(a, [0u64; 10]);
}

#[test]
fn test_u64_all_42s_array_nested_in_array_repeat() {
    u64_all_42s_array_nested_in_array_repeat();
}

#[inline(never)]
pub fn u64_all_42s_array_nested_in_array_repeat() {
    let a = [[42u64; 10]; 10];
    assert_all_elems_equal(a, [42u64; 10]);
}

#[test]
fn test_u64_all_zeros_array_nested_in_tuple_repeat() {
    u64_all_zeros_array_nested_in_tuple_repeat();
}

#[inline(never)]
pub fn u64_all_zeros_array_nested_in_tuple_repeat() {
    let a = (0u64, [0u64; 10], false);

    assert_eq(a.0, 0u64);
    assert_all_elems_equal(a.1, 0u64);
    assert_eq(a.2, false);
}

#[test]
fn test_u64_all_42s_array_nested_in_tuple_repeat() {
    u64_all_42s_array_nested_in_tuple_repeat();
}

#[inline(never)]
pub fn u64_all_42s_array_nested_in_tuple_repeat() {
    let a = (42u64, [42u64; 10], true);

    assert_eq(a.0, 42u64);
    assert_all_elems_equal(a.1, 42u64);
    assert_eq(a.2, true);
}

fn assert_all_elems_equal<T>(array: [T; 10], val: T)
where
    T: PartialEq + AbiEncode,
{
    let mut i = 0;
    while i < 10 {
        assert_eq(array[i], val);
        i += 1;
    }
}
