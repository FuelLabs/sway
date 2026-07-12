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

    // Additionally validate each element field-by-field against primitive literals.
    let mut i = 0;
    while i < 10 {
        assert_no_nesting(a[i], 42, true, 42u256, b256::zero());
        i += 1;
    }
}

#[test]
fn test_no_nesting_mostly_zeros_repeat() {
    no_nesting_mostly_zeros_repeat();
}

/// A repeat array of a mostly-zeroed struct. Every element is the same struct
/// whose only non-zero field is a large `u256`, so the whole array is mostly
/// zeroed. This exercises the interaction between the mostly-zeroed lowering
/// (`mem_clear_val` for the whole array) and the repeat-array lowering (which
/// still has to store the non-zero field into every element).
#[inline(never)]
pub fn no_nesting_mostly_zeros_repeat() {
    let a = [
        NoNesting {
            a: 0,
            b: false,
            c: 123u256,
            d: b256::zero(),
            u: (),
        }; 10
    ];

    let mut i = 0;
    while i < 10 {
        assert_no_nesting(a[i], 0, false, 123u256, b256::zero());
        i += 1;
    }
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

    // Additionally validate each element's field against a primitive literal.
    let mut i = 0;
    while i < 10 {
        assert_eq(a[i].a, 42);
        i += 1;
    }
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

    // Additionally validate each individual element against a primitive literal.
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
fn test_u64_all_42s_array_nested_in_array_repeat() {
    u64_all_42s_array_nested_in_array_repeat();
}

#[inline(never)]
pub fn u64_all_42s_array_nested_in_array_repeat() {
    let a = [[42u64; 10]; 10];
    assert_all_elems_equal(a, [42u64; 10]);

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
