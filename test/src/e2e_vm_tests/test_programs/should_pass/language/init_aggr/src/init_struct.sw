//! Initialization of structs, with and without nesting, all-zeros and not-all-zeros.
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

    assert_no_nesting(s, 42, true, 42u256, b256::zero());
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

    assert_no_nesting_all_zeros(s);
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

    assert_no_nesting_all_zeros(s.n1);
    assert_no_nesting(s.n2, 42, true, 42u256, b256::zero());
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

    assert_no_nesting_all_zeros(s.n1);
    assert_no_nesting_all_zeros(s.n2);
}
