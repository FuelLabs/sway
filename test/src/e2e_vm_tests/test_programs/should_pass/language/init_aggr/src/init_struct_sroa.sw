//! Initialization of structs where only a single field is later accessed.
//! This exercises the interaction between `init_aggr` lowering and SROA,
//! which can eliminate the aggregate entirely once its fields are
//! lowered to individual stores.
library;

use ::types::*;

#[test]
fn test_no_nesting_not_all_zeros() {
    no_nesting_not_all_zeros();
}

#[inline(never)]
pub fn no_nesting_not_all_zeros() {
    let s = NoNesting {
        a: 42,
        b: true,
        c: 42u256,
        d: b256::zero(),
        u: (),
    };

    assert_eq(s.c, 42u256);
}

#[test]
fn test_no_nesting_all_zeros() {
    no_nesting_all_zeros();
}

#[inline(never)]
pub fn no_nesting_all_zeros() {
    let s = NoNesting {
        a: 0,
        b: false,
        c: 0u256,
        d: b256::zero(),
        u: (),
    };

    assert_eq(s.c, 0u256);
}

#[test]
fn test_nested_not_all_zeros() {
    nested_not_all_zeros();
}

#[inline(never)]
pub fn nested_not_all_zeros() {
    let s = Nested {
        n1: NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
        n2: NoNesting {
            a: 42,
            b: true,
            c: 42u256,
            d: b256::zero(),
            u: (),
        },
    };

    assert_eq(s.n1.c, 0u256);
    assert_eq(s.n2.c, 42u256);
}

#[test]
fn test_nested_all_zeros() {
    nested_all_zeros();
}

#[inline(never)]
pub fn nested_all_zeros() {
    let s = Nested {
        n1: NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
        n2: NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
    };

    assert_eq(s.n1.c, 0u256);
    assert_eq(s.n2.c, 0u256);
}
