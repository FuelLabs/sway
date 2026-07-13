//! Initialization of non-repeat arrays, i.e. arrays whose elements are given
//! individually (even if they happen to hold equal values). This tests the
//! per-element lowering path (as opposed to the repeat-array path).
library;

use ::types::*;

#[test]
fn test_u64_all_zeros() {
    u64_all_zeros();
}

#[inline(never)]
pub fn u64_all_zeros() {
    let a = [0u64, 0, 0_0, 0, 0, 0, 0, 0u64, 0, 0];

    assert_all_elems_equal(a, 0u64);
}

#[test]
fn test_u64_all_42s() {
    u64_all_42s();
}

#[inline(never)]
pub fn u64_all_42s() {
    let a = [42u64, 42, 4_2, 42, 42, 42, 42, 42u64, 42, 42];

    assert_all_elems_equal(a, 42u64);
}

#[test]
fn test_u64_mixed_values() {
    u64_mixed_values();
}

#[inline(never)]
pub fn u64_mixed_values() {
    let a = [0u64, 1, 2, 3, 4, 5, 6, 7, 8, 9];

    let mut i = 0;
    while i < 10 {
        assert_eq(a[i], i);
        i += 1;
    }
}

#[test]
fn test_no_nesting_all_zeros() {
    no_nesting_all_zeros();
}

#[inline(never)]
pub fn no_nesting_all_zeros() {
    let a = [
        NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
    ];

    assert_all_elems_equal(a, NoNesting {
        a: 0,
        b: false,
        c: 0u256,
        d: b256::zero(),
        u: (),
    });

    // Additionally validate each element field-by-field against primitive
    // literals. The aggregate-level assert above compares against a `NoNesting`
    // that is itself created via `init_aggr`, so a bug shared between the
    // element initialization and the expected value could otherwise be masked.
    let mut i = 0;
    while i < 10 {
        assert_no_nesting_all_zeros(a[i]);
        i += 1;
    }
}

#[test]
fn test_no_nesting_not_all_zeros() {
    no_nesting_not_all_zeros();
}

#[inline(never)]
pub fn no_nesting_not_all_zeros() {
    let a = [
        NoNesting {
            a: 42,
            b: true,
            c: 42u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 42,
            b: true,
            c: 42u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 42,
            b: true,
            c: 42u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 42,
            b: true,
            c: 42u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 42,
            b: true,
            c: 42u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 42,
            b: true,
            c: 42u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 42,
            b: true,
            c: 42u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 42,
            b: true,
            c: 42u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 42,
            b: true,
            c: 42u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 42,
            b: true,
            c: 42u256,
            d: b256::zero(),
            u: (),
        },
    ];

    assert_all_elems_equal(a, NoNesting {
        a: 42,
        b: true,
        c: 42u256,
        d: b256::zero(),
        u: (),
    });

    // Additionally validate each element field-by-field against primitive literals.
    let mut i = 0;
    while i < 10 {
        assert_no_nesting(a[i], 42, true, 42u256, b256::zero());
        i += 1;
    }
}

#[test]
fn test_no_nesting_mostly_zeros() {
    no_nesting_mostly_zeros();
}

/// A mostly-zeroed array of structs. Most elements are fully zero, and the few
/// non-zero elements each make a *different* single primitive field non-zero, so
/// that on top of the zero-cleared aggregate we exercise storing each nested
/// primitive type (`u64`, `bool`, `u256`, `b256`).
#[inline(never)]
pub fn no_nesting_mostly_zeros() {
    let a = [
        // index 0: `u64` field non-zero.
        NoNesting {
            a: 42,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
        // index 1: all zero.
        NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
        // index 2: `bool` field non-zero.
        NoNesting {
            a: 0,
            b: true,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
        // index 3: all zero.
        NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
        // index 4: `u256` field non-zero.
        NoNesting {
            a: 0,
            b: false,
            c: 123u256,
            d: b256::zero(),
            u: (),
        },
        // index 5: all zero.
        NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
        // index 6: `b256` field non-zero.
        NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: 0x00000000000000000000000000000000000000000000000000000000000000FF,
            u: (),
        },
        // indices 7, 8, 9: all zero.
        NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
        NoNesting {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        },
    ];

    // Validate each element field-by-field against primitive literals.
    assert_no_nesting(a[0], 42, false, 0u256, b256::zero());
    assert_no_nesting_all_zeros(a[1]);
    assert_no_nesting(a[2], 0, true, 0u256, b256::zero());
    assert_no_nesting_all_zeros(a[3]);
    assert_no_nesting(a[4], 0, false, 123u256, b256::zero());
    assert_no_nesting_all_zeros(a[5]);
    assert_no_nesting(
        a[6],
        0,
        false,
        0u256,
        0x00000000000000000000000000000000000000000000000000000000000000FF,
    );
    assert_no_nesting_all_zeros(a[7]);
    assert_no_nesting_all_zeros(a[8]);
    assert_no_nesting_all_zeros(a[9]);
}

#[test]
fn test_b256_zero() {
    b256_zero();
}

#[inline(never)]
pub fn b256_zero() {
    let a = [
        b256::zero(),
        b256::zero(),
        b256::zero(),
        b256::zero(),
        b256::zero(),
        b256::zero(),
        b256::zero(),
        b256::zero(),
        b256::zero(),
        b256::zero(),
    ];

    assert_all_elems_equal(a, b256::zero());
}

#[test]
fn test_b256_zero_literal() {
    b256_zero_literal();
}

#[inline(never)]
pub fn b256_zero_literal() {
    let a: [b256; 10] = [
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    ];

    assert_all_elems_equal(a, b256::zero());
}

#[test]
fn test_empty_array() {
    empty_array();
}

#[inline(never)]
pub fn empty_array() {
    let a: [u64; 0] = [];
    assert_eq(a.len(), 0);
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
fn test_single_field_struct_not_zero() {
    single_field_struct_not_zero();
}

#[inline(never)]
pub fn single_field_struct_not_zero() {
    let a = [
        SingleFieldStruct { a: 42 },
        SingleFieldStruct { a: 42 },
        SingleFieldStruct { a: 42 },
        SingleFieldStruct { a: 42 },
        SingleFieldStruct { a: 42 },
        SingleFieldStruct { a: 42 },
        SingleFieldStruct { a: 42 },
        SingleFieldStruct { a: 42 },
        SingleFieldStruct { a: 42 },
        SingleFieldStruct { a: 42 },
    ];
    assert_all_elems_equal(a, SingleFieldStruct { a: 42 });

    // Additionally validate each element's field against a primitive literal.
    let mut i = 0;
    while i < 10 {
        assert_eq(a[i].a, 42);
        i += 1;
    }
}

#[test]
fn test_single_field_struct_not_zero_sroa() {
    single_field_struct_not_zero_sroa();
}

#[inline(never)]
pub fn single_field_struct_not_zero_sroa() {
    let a = [
        SingleFieldStruct { a: 42 },
        SingleFieldStruct { a: 42 },
        SingleFieldStruct { a: 42 },
        SingleFieldStruct { a: 42 },
        SingleFieldStruct { a: 42 },
    ];
    assert_eq(a[1].a, 42);
}

#[test]
fn test_u64_all_zeros_array_nested_in_array() {
    u64_all_zeros_array_nested_in_array();
}

#[inline(never)]
pub fn u64_all_zeros_array_nested_in_array() {
    let a = [
        [0u64, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0u64, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0u64, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0u64, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0u64, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0u64, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0u64, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0u64, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0u64, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [0u64, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    assert_all_elems_equal(a, [0u64, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

    // Additionally validate each individual element against a primitive literal.
    // The aggregate-level assert above compares against an array literal that is
    // itself created via `init_aggr`.
    let mut i = 0;
    while i < 10 {
        let mut j = 0;
        while j < 10 {
            assert_eq(a[i][j], 0u64);
            j += 1;
        }
        i += 1;
    }
}

#[test]
fn test_u64_all_42s_array_nested_in_array() {
    u64_all_42s_array_nested_in_array();
}

#[inline(never)]
pub fn u64_all_42s_array_nested_in_array() {
    let a = [
        [42u64, 42, 42, 4_2, 42, 42, 42, 42, 42, 42],
        [42u64, 42, 42, 42, 42, 42, 42, 42, 42, 42],
        [42u64, 42, 42, 42, 42, 42, 42, 42, 42, 42],
        [42u64, 42, 42, 42, 42, 42, 42, 42, 42, 42],
        [42u64, 42, 42, 42, 42, 42, 42, 42, 42, 42],
        [42u64, 42, 42, 4_2, 42, 42, 42, 42, 42, 42],
        [42u64, 42, 42, 42, 42, 42, 42, 42, 42, 42],
        [42u64, 42, 42, 42, 42, 42, 42, 42, 42, 42],
        [42u64, 42, 42, 42, 42, 42, 42, 42, 42, 42],
        [42u64, 42, 42, 42, 42, 42, 42, 42, 42, 42],
    ];
    assert_all_elems_equal(a, [42u64, 42, 42, 42, 42, 42, 42, 42, 42, 42]);

    // Additionally validate each individual element against a primitive literal.
    let mut i = 0;
    while i < 10 {
        let mut j = 0;
        while j < 10 {
            assert_eq(a[i][j], 42u64);
            j += 1;
        }
        i += 1;
    }
}

#[test]
fn test_u64_all_zeros_array_nested_in_tuple() {
    u64_all_zeros_array_nested_in_tuple();
}

#[inline(never)]
pub fn u64_all_zeros_array_nested_in_tuple() {
    let a = (0u64, [0u64, 0, 0, 0, 0, 0, 0, 0, 0u64, 0], false);
    assert_eq(a.0, 0u64);
    assert_all_elems_equal(a.1, 0u64);
    assert_eq(a.2, false);
}

#[test]
fn test_u64_all_42s_array_nested_in_tuple() {
    u64_all_42s_array_nested_in_tuple();
}

#[inline(never)]
pub fn u64_all_42s_array_nested_in_tuple() {
    let a = (42u64, [42u64, 42, 42, 42, 4_2, 42, 42, 42, 42u64, 42], true);
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
