//! Regression tests for issues found while developing the `init_aggr`
//! optimization. These reproduce const-evaluation and lowering bugs around
//! initialization of tuples containing large scalars (`u256`) built from
//! runtime values, including through `std::crypto::point2d`.
library;

use ::types::*;
use std::crypto::point2d::*;

// Reproduces a const-eval issue where a tuple of `u256`s was initialized from
// values copied out of a `Point2D` (i.e. from runtime pointers), wrapped in an
// `Option`.
#[test]
fn test_const_eval_issue_extracted() {
    // Use `zero()` (not `new()`): `zero()` produces valid 32-byte zero buffers,
    // whereas `new()` produces empty `Bytes` and copying from them reads out of
    // bounds. The initialization pattern being exercised (a tuple of `u256`s
    // built from runtime pointer copies, wrapped in an `Option`) is the same.
    let point = Point2D::zero();

    let mut value_x = 0x0000000000000000000000000000000000000000000000000000000000000000_u256;
    let mut value_y = 0x0000000000000000000000000000000000000000000000000000000000000000_u256;
    let ptr_x = __addr_of(value_x);
    let ptr_y = __addr_of(value_y);

    point.x().ptr().copy_to::<u256>(ptr_x, 1);
    point.y().ptr().copy_to::<u256>(ptr_y, 1);

    let x = Some((value_x, value_y));

    assert_eq(x.unwrap().0, value_x);
    assert_eq(x.unwrap().1, value_y);
    assert_eq(x.unwrap().0, 0u256);
    assert_eq(x.unwrap().1, 0u256);
}

// Reproduces an issue where a `(u256, u256)` tuple produced by `TryFrom` for a
// `Point2D` was incorrectly initialized.
#[test]
fn test_point2d_u256_tuple_try_from() {
    let max = Point2D::from((
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
    ));

    let (x, y) = <(u256, u256) as TryFrom<Point2D>>::try_from(max).unwrap();

    assert_eq(x, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256);
    assert_eq(y, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256);
}

// The tests below reproduce edge cases stumbled upon while developing the
// mostly-zeroed aggregates lowering. They stress the size/zero-ratio analysis
// and the recursive lowering on unusual shapes: `u8`s laid out in tuples
// (word-aligned) vs. arrays (packed), non-zero leaves at non-leading positions,
// single-element arrays/tuples, and deeply nested single-element aggregates like
// `((([0], ), ), )` and `[[[[0]]]]`.

// A mostly-zeroed tuple whose first element is a repeat array of `u8`, with a
// nested all-zero tuple in the middle and many trailing zero scalars.
#[test]
fn test_mostly_zeroed_repeat_array_and_nested_tuple() {
    mostly_zeroed_repeat_array_and_nested_tuple();
}

#[inline(never)]
pub fn mostly_zeroed_repeat_array_and_nested_tuple() {
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

// Same shape, but the nested all-zero tuple comes from a local variable (a
// runtime-zeroed aggregate initializer) rather than an inline nested `init_aggr`.
#[test]
fn test_mostly_zeroed_nested_tuple_from_variable() {
    mostly_zeroed_nested_tuple_from_variable();
}

#[inline(never)]
pub fn mostly_zeroed_nested_tuple_from_variable() {
    let inner_tuple = (0u64, 0u64, 0u64);
    let t = (
        [42u8; 8],
        0u64,
        inner_tuple,
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

// The eight leading non-zero `u8`s are laid out as a *tuple*. In a tuple each
// `u8` is aligned to a word, so the non-zero part is 8 * 8 bytes, giving a
// different zero-ratio than the packed-array variant below.
#[test]
fn test_mostly_zeroed_leading_u8_tuple() {
    mostly_zeroed_leading_u8_tuple();
}

#[inline(never)]
pub fn mostly_zeroed_leading_u8_tuple() {
    let t = (
        (42u8, 42u8, 42u8, 42u8, 42u8, 42u8, 42u8, 42u8),
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

    assert_eq(t.0.0, 42u8);
    assert_eq(t.0.1, 42u8);
    assert_eq(t.0.2, 42u8);
    assert_eq(t.0.3, 42u8);
    assert_eq(t.0.4, 42u8);
    assert_eq(t.0.5, 42u8);
    assert_eq(t.0.6, 42u8);
    assert_eq(t.0.7, 42u8);
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

// The same eight leading non-zero `u8`s, but laid out as an *array*. In an array
// the `u8` elements are packed (1 byte each), so the counting of zero/non-zero
// sizes differs from the tuple variant above.
#[test]
fn test_mostly_zeroed_leading_u8_array() {
    mostly_zeroed_leading_u8_array();
}

#[inline(never)]
pub fn mostly_zeroed_leading_u8_array() {
    let t = (
        [42u8, 42u8, 42u8, 42u8, 42u8, 42u8, 42u8, 42u8],
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

// A tiny tuple sitting right at the zero-ratio boundary (a zero `u8` aligned to a
// word, plus one non-zero `u64`).
#[test]
fn test_half_zeroed_u8_and_u64_tuple() {
    half_zeroed_u8_and_u64_tuple();
}

#[inline(never)]
pub fn half_zeroed_u8_and_u64_tuple() {
    let t = (0u8, 42u64);

    assert_eq(t.0, 0u8);
    assert_eq(t.1, 42u64);
}

// Same as above, but the zero `u8` is wrapped in a single-element array, so it is
// counted as packed (1 byte) rather than word-aligned.
#[test]
fn test_half_zeroed_single_elem_u8_array_and_u64() {
    half_zeroed_single_elem_u8_array_and_u64();
}

#[inline(never)]
pub fn half_zeroed_single_elem_u8_array_and_u64() {
    let t = ([0u8], 42u64);

    assert_eq(t.0[0], 0u8);
    assert_eq(t.1, 42u64);
}

// A mostly-zeroed aggregate whose non-zero leaves are *not* at the leading
// positions: one in the middle of the array, one in the middle of the nested
// tuple. Exercises storing non-zero values at arbitrary offsets after the
// `mem_clear_val`.
#[test]
fn test_mostly_zeroed_non_zero_in_the_middle() {
    mostly_zeroed_non_zero_in_the_middle();
}

#[inline(never)]
pub fn mostly_zeroed_non_zero_in_the_middle() {
    let t = (
        [0u8, 0u8, 0u8, 0u8, 0u8, 42u8, 0u8, 0u8],
        0u64,
        (0u64, 42u64, 0u64),
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
        if i == 5 {
            assert_eq(t.0[i], 42u8);
        } else {
            assert_eq(t.0[i], 0u8);
        }
        i += 1;
    }
    assert_eq(t.1, 0u64);
    assert_eq(t.2.0, 0u64);
    assert_eq(t.2.1, 42u64);
    assert_eq(t.2.2, 0u64);
    assert_eq(t.3, 0u64);
    assert_eq(t.4, 0u64);
    assert_eq(t.5, 0u64);
    assert_eq(t.6, 0u64);
    assert_eq(t.7, 0u64);
    assert_eq(t.8, 0u64);
    assert_eq(t.9, 0u64);
}

// A single-element array. Such arrays are treated as non-repeat by the lowering.
#[test]
fn test_single_element_array_zero() {
    single_element_array_zero();
}

#[inline(never)]
pub fn single_element_array_zero() {
    let a = [0];

    assert_eq(a[0], 0);
}

// A single-element tuple.
#[test]
fn test_single_element_tuple_zero() {
    single_element_tuple_zero();
}

#[inline(never)]
pub fn single_element_tuple_zero() {
    let t = (0,);

    assert_eq(t.0, 0);
}

// A single-element tuple whose only element is a single-element array.
#[test]
fn test_single_element_tuple_of_single_element_array() {
    single_element_tuple_of_single_element_array();
}

#[inline(never)]
pub fn single_element_tuple_of_single_element_array() {
    let t = ([0],);

    assert_eq(t.0[0], 0);
}

// Deeply nested single-element tuples ending in a single-element array:
// `((([0], ), ), )`.
#[test]
fn test_deeply_nested_single_element_tuples_and_array() {
    deeply_nested_single_element_tuples_and_array();
}

#[inline(never)]
pub fn deeply_nested_single_element_tuples_and_array() {
    let t = ((([0],),),);

    assert_eq(t.0.0.0[0], 0);
}

// Same deeply nested shape, but with a `u256` leaf, i.e. a large scalar.
#[test]
fn test_deeply_nested_single_element_tuples_and_array_u256() {
    deeply_nested_single_element_tuples_and_array_u256();
}

#[inline(never)]
pub fn deeply_nested_single_element_tuples_and_array_u256() {
    let t = ((([0u256],),),);

    assert_eq(t.0.0.0[0], 0u256);
}

// Four levels of nested single-element arrays: `[[[[0u64]]]]`.
#[test]
fn test_deeply_nested_single_element_arrays() {
    deeply_nested_single_element_arrays();
}

#[inline(never)]
pub fn deeply_nested_single_element_arrays() {
    let a = [[[[0u64]]]];

    assert_eq(a[0][0][0][0], 0u64);
}

// Four levels of nested arrays where the innermost array has two elements:
// `[[[[0u64, 0u64]]]]`.
#[test]
fn test_deeply_nested_arrays() {
    deeply_nested_arrays();
}

#[inline(never)]
pub fn deeply_nested_arrays() {
    let a = [[[[0u64, 0u64]]]];

    assert_eq(a[0][0][0][0], 0u64);
    assert_eq(a[0][0][0][1], 0u64);
}

// Same as above, but with `u256` leaves.
#[test]
fn test_deeply_nested_arrays_u256() {
    deeply_nested_arrays_u256();
}

#[inline(never)]
pub fn deeply_nested_arrays_u256() {
    let a = [[[[0u256, 0u256]]]];

    assert_eq(a[0][0][0][0], 0u256);
    assert_eq(a[0][0][0][1], 0u256);
}
