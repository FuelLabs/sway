//! Initialization of empty / zero-sized aggregates.
library;

use ::types::*;

#[test]
fn test_empty_struct() {
    empty_struct();
}

#[inline(never)]
pub fn empty_struct() {
    let s = EmptyStruct {};
    assert_eq(s, EmptyStruct {});
}

#[test]
fn test_empty_struct_container() {
    empty_struct_container();
}

#[inline(never)]
pub fn empty_struct_container() {
    let s = EmptyStructContainer { e: EmptyStruct {} };
    assert_eq(s, EmptyStructContainer { e: EmptyStruct {} });
}

#[test]
fn test_empty_struct_container_with_side_effect() {
    empty_struct_container_with_side_effect();
}

#[inline(never)]
pub fn empty_struct_container_with_side_effect() {
    let s = EmptyStructContainer {
        e: return_empty_struct(),
    };
    assert_eq(s, EmptyStructContainer { e: EmptyStruct {} });
}

#[test]
fn test_empty_tuple_like_aggregates() {
    empty_tuple_like_aggregates();
}

#[inline(never)]
pub fn empty_tuple_like_aggregates() {
    // A tuple of unit and empty struct values is zero-sized.
    let t = ((), EmptyStruct {}, ());
    assert_eq(t.0, ());
    assert_eq(t.1, EmptyStruct {});
    assert_eq(t.2, ());

    let a: [u64; 0] = [];
    assert_eq(a.len(), 0);
}
