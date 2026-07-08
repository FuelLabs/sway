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
