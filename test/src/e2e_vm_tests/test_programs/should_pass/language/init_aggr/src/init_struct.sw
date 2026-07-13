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

// Mostly-zeroed struct variants. Only a single field is non-zero, so the struct
// hits the mostly-zeroed lowering (`mem_clear_val` for the whole struct followed
// by a `store` for the non-zero field). The `init_mostly_zeroed` module already
// covers the `u256` and `b256` non-zero cases for `NoNesting`; here we complement
// it with the `u64` and `bool` non-zero cases, so that storing each nested
// primitive type on top of a zero-cleared struct is covered.

#[test]
fn test_no_nesting_mostly_zeros_u64() {
    no_nesting_mostly_zeros_u64();
}

#[inline(never)]
pub fn no_nesting_mostly_zeros_u64() {
    let s = NoNesting {
        a: 42,
        b: false,
        c: 0u256,
        d: b256::zero(),
        u: (),
    };

    assert_no_nesting(s, 42, false, 0u256, b256::zero());
}

#[test]
fn test_no_nesting_mostly_zeros_bool() {
    no_nesting_mostly_zeros_bool();
}

#[inline(never)]
pub fn no_nesting_mostly_zeros_bool() {
    let s = NoNesting {
        a: 0,
        b: true,
        c: 0u256,
        d: b256::zero(),
        u: (),
    };

    assert_no_nesting(s, 0, true, 0u256, b256::zero());
}

// A nested struct that is mostly zeroed, with a non-zero leaf in each of the two
// nested structs, exercising different primitive types (`bool` and `u256`).
#[test]
fn test_nested_mostly_zeros() {
    nested_mostly_zeros();
}

#[inline(never)]
pub fn nested_mostly_zeros() {
    let s = Nested {
        n1: NoNesting {
            a: 0,
            b: true,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
        n2: NoNesting {
            a: 0,
            b: false,
            c: 99u256,
            d: b256::zero(),
            u: (),
        },
    };

    assert_no_nesting(s.n1, 0, true, 0u256, b256::zero());
    assert_no_nesting(s.n2, 0, false, 99u256, b256::zero());
}
