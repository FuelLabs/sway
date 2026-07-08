//! Initialization of "mostly zeroed" aggregates, i.e. aggregates where a large
//! fraction of the bytes are zero-initialized. The optimization lowers such
//! aggregates to a single `mem_clear_val` for the whole aggregate followed by
//! `store`s for only the non-zero fields.
//!
//! These tests cover:
//! - the common `(non_zero, 0, 0, 0, ...)` shapes for tuples, arrays and structs,
//! - nesting (arrays in arrays, tuples in tuples, structs in structs),
//! - values around the "is it mostly zeroed?" ratio threshold,
//! - fully zeroed aggregates (100% zero), and
//! - aggregates that are *not* mostly zeroed (below the threshold), which must
//!   still be initialized correctly by the default lowering.
library;

use ::types::*;

// A tuple with a single non-zero leading element followed by zeros.
#[test]
fn test_tuple_leading_non_zero() {
    tuple_leading_non_zero();
}

#[inline(never)]
pub fn tuple_leading_non_zero() {
    let t = (42u64, 0u64, 0u64, 0u64);

    assert_eq(t.0, 42u64);
    assert_eq(t.1, 0u64);
    assert_eq(t.2, 0u64);
    assert_eq(t.3, 0u64);
}

// A tuple with a single non-zero trailing element preceded by zeros. This is
// the classic `(0, 0, 0, some_variable)` case.
#[test]
fn test_tuple_trailing_non_zero() {
    tuple_trailing_non_zero();
}

#[inline(never)]
pub fn tuple_trailing_non_zero() {
    let x = opaque_u64(7);
    let t = (0u64, 0u64, 0u64, x);

    assert_eq(t.0, 0u64);
    assert_eq(t.1, 0u64);
    assert_eq(t.2, 0u64);
    assert_eq(t.3, 7u64);
}

// An array with a single non-zero leading element.
#[test]
fn test_array_leading_non_zero() {
    array_leading_non_zero();
}

#[inline(never)]
pub fn array_leading_non_zero() {
    let a = [42u64, 0, 0, 0];

    assert_eq(a[0], 42u64);
    assert_eq(a[1], 0u64);
    assert_eq(a[2], 0u64);
    assert_eq(a[3], 0u64);
}

// Nested arrays where the outer array is mostly zeroed: one non-zero inner
// array followed by all-zero inner arrays.
#[test]
fn test_nested_arrays_mostly_zeroed() {
    nested_arrays_mostly_zeroed();
}

#[inline(never)]
pub fn nested_arrays_mostly_zeroed() {
    let a = [[42u64; 10], [0u64; 10], [0u64; 10], [0u64; 10]];

    let mut i = 0;
    while i < 10 {
        assert_eq(a[0][i], 42u64);
        assert_eq(a[1][i], 0u64);
        assert_eq(a[2][i], 0u64);
        assert_eq(a[3][i], 0u64);
        i += 1;
    }
}

// A tuple whose first element is a non-zero array, followed by many zero
// scalars. The non-zero array is a small fraction of the whole aggregate.
#[test]
fn test_tuple_array_then_many_zeros() {
    tuple_array_then_many_zeros();
}

#[inline(never)]
pub fn tuple_array_then_many_zeros() {
    let t = (
        [42u8; 8],
        0u64,
        0u64,
        0u64,
        0u64,
        0u64,
        0u64,
        0u64,
        0u64,
        0u64,
        0u64,
        0u64,
    );

    let mut i = 0;
    while i < 8 {
        assert_eq(t.0[i], 42u8);
        i += 1;
    }
    assert_eq(t.1, 0u64);
    assert_eq(t.2, 0u64);
    assert_eq(t.3, 0u64);
    assert_eq(t.4, 0u64);
    assert_eq(t.5, 0u64);
    assert_eq(t.6, 0u64);
    assert_eq(t.7, 0u64);
    assert_eq(t.8, 0u64);
    assert_eq(t.9, 0u64);
    assert_eq(t.10, 0u64);
    assert_eq(t.11, 0u64);
}

// A deeply nested mostly-zeroed tuple: an array, then a zero scalar, then a
// nested all-zero tuple, then more zero scalars.
#[test]
fn test_deeply_nested_mostly_zeroed() {
    deeply_nested_mostly_zeroed();
}

#[inline(never)]
pub fn deeply_nested_mostly_zeroed() {
    let t = (
        [42u8; 8],
        0u64,
        (0u64, 0u64, 0u64),
        0u64,
        0u64,
        0u64,
        0u64,
        0u64,
        0u64,
        0u64,
    );

    let mut i = 0;
    while i < 8 {
        assert_eq(t.0[i], 42u8);
        i += 1;
    }
    assert_eq(t.1, 0u64);
    assert_eq(t.2.0, 0u64);
    assert_eq(t.2.1, 0u64);
    assert_eq(t.2.2, 0u64);
    assert_eq(t.3, 0u64);
    assert_eq(t.4, 0u64);
    assert_eq(t.5, 0u64);
    assert_eq(t.6, 0u64);
    assert_eq(t.7, 0u64);
    assert_eq(t.8, 0u64);
    assert_eq(t.9, 0u64);
}

// A struct that is mostly zeroed but has one large non-zero field.
#[test]
fn test_struct_mostly_zeroed_large_non_zero() {
    struct_mostly_zeroed_large_non_zero();
}

#[inline(never)]
pub fn struct_mostly_zeroed_large_non_zero() {
    let s = NoNesting {
        a: 0,
        b: false,
        c: 123u256,
        d: b256::zero(),
        u: (),
    };

    assert_no_nesting(s, 0, false, 123u256, b256::zero());
}

// A struct that is mostly zeroed but with a non-zero `b256` field.
#[test]
fn test_struct_mostly_zeroed_non_zero_b256() {
    struct_mostly_zeroed_non_zero_b256();
}

#[inline(never)]
pub fn struct_mostly_zeroed_non_zero_b256() {
    let d = 0x00000000000000000000000000000000000000000000000000000000000000FF;
    let s = NoNesting {
        a: 0,
        b: false,
        c: 0u256,
        d,
        u: (),
    };

    assert_no_nesting(s, 0, false, 0u256, d);
}

// Nested struct where only a single leaf field is non-zero.
#[test]
fn test_nested_struct_single_non_zero_leaf() {
    nested_struct_single_non_zero_leaf();
}

#[inline(never)]
pub fn nested_struct_single_non_zero_leaf() {
    let s = Nested {
        n1: NoNesting::default(),
        n2: NoNesting {
            a: 5,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
    };

    assert_no_nesting_all_zeros(s.n1);
    assert_no_nesting(s.n2, 5, false, 0u256, b256::zero());
}

// A fully zeroed aggregate (100% zero). Should lower to just a `mem_clear_val`.
#[test]
fn test_fully_zeroed() {
    fully_zeroed();
}

#[inline(never)]
pub fn fully_zeroed() {
    let t = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
    assert_eq(t.0, 0u64);
    assert_eq(t.7, 0u64);

    let a = [0u64, 0, 0, 0];
    assert_eq(a[0], 0u64);
    assert_eq(a[3], 0u64);

    let s = NoNesting::default();
    assert_no_nesting_all_zeros(s);
}

// A tuple that is NOT mostly zeroed (well below the threshold). This must still
// be initialized correctly by the default lowering.
#[test]
fn test_not_mostly_zeroed() {
    not_mostly_zeroed();
}

#[inline(never)]
pub fn not_mostly_zeroed() {
    let t = (1u64, 2u64, 3u64, 0u64);

    assert_eq(t.0, 1u64);
    assert_eq(t.1, 2u64);
    assert_eq(t.2, 3u64);
    assert_eq(t.3, 0u64);
}

// A tuple sitting right around the "half zeroed" mark.
#[test]
fn test_half_zeroed() {
    half_zeroed();
}

#[inline(never)]
pub fn half_zeroed() {
    let t = (1u64, 0u64, 2u64, 0u64);

    assert_eq(t.0, 1u64);
    assert_eq(t.1, 0u64);
    assert_eq(t.2, 2u64);
    assert_eq(t.3, 0u64);
}

#[inline(never)]
fn opaque_u64(x: u64) -> u64 {
    asm() {}; // To forbid const-eval.
    x
}
