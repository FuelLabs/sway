//! Initialization of tuples, with and without nesting, all-zeros and
//! not-all-zeros. Tuples are lowered as structs by the `init_aggr` lowering.
library;

use ::types::*;

#[test]
fn test_no_nesting_not_all_zeros() {
    no_nesting_not_all_zeros();
}

#[inline(never)]
pub fn no_nesting_not_all_zeros() {
    let t = (42u64, true, 42u256, b256::zero(), ());

    assert_no_nesting_tuple(t, 42, true, 42u256, b256::zero());
}

#[test]
fn test_no_nesting_all_zeros() {
    no_nesting_all_zeros();
}

#[inline(never)]
pub fn no_nesting_all_zeros() {
    let t = (0u64, false, 0u256, b256::zero(), ());

    assert_no_nesting_tuple_all_zeros(t);
}

#[test]
fn test_nested_not_all_zeros() {
    nested_not_all_zeros();
}

#[inline(never)]
pub fn nested_not_all_zeros() {
    let t = (
        (0u64, false, 0u256, b256::zero(), ()),
        (42u64, true, 42u256, b256::zero(), ()),
    );

    assert_no_nesting_tuple_all_zeros(t.0);
    assert_no_nesting_tuple(t.1, 42, true, 42u256, b256::zero());
}

#[test]
fn test_nested_all_zeros() {
    nested_all_zeros();
}

#[inline(never)]
pub fn nested_all_zeros() {
    let t = (
        (0u64, false, 0u256, b256::zero(), ()),
        (0u64, false, 0u256, b256::zero(), ()),
    );

    assert_no_nesting_tuple_all_zeros(t.0);
    assert_no_nesting_tuple_all_zeros(t.1);
}

// Mostly-zeroed tuples where a single element is non-zero, so the tuple hits the
// mostly-zeroed lowering. Each test makes a *different* primitive element
// non-zero. The `init_mostly_zeroed` module already covers the `u64` non-zero
// case, so here we cover `bool`, `u256` and `b256`.

#[test]
fn test_mostly_zeros_bool_non_zero() {
    mostly_zeros_bool_non_zero();
}

#[inline(never)]
pub fn mostly_zeros_bool_non_zero() {
    let t = (0u64, true, 0u256, b256::zero(), ());

    assert_no_nesting_tuple(t, 0, true, 0u256, b256::zero());
}

#[test]
fn test_mostly_zeros_u256_non_zero() {
    mostly_zeros_u256_non_zero();
}

#[inline(never)]
pub fn mostly_zeros_u256_non_zero() {
    let t = (0u64, false, 123u256, b256::zero(), ());

    assert_no_nesting_tuple(t, 0, false, 123u256, b256::zero());
}

#[test]
fn test_mostly_zeros_b256_non_zero() {
    mostly_zeros_b256_non_zero();
}

#[inline(never)]
pub fn mostly_zeros_b256_non_zero() {
    let d = 0x00000000000000000000000000000000000000000000000000000000000000FF;
    let t = (0u64, false, 0u256, d, ());

    assert_no_nesting_tuple(t, 0, false, 0u256, d);
}

#[test]
fn test_single_element_tuple() {
    single_element_tuple();
}

#[inline(never)]
pub fn single_element_tuple() {
    let t = (42u64,);
    assert_eq(t.0, 42u64);

    let z = (0u64,);
    assert_eq(z.0, 0u64);
}
