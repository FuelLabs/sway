//! Initialization of aggregates that reduce to a single stored value, e.g.
//! single-field structs and chains of single-field wrapper structs. These are
//! interesting because the lowering should avoid unnecessary temporaries and
//! `memcpy`s (especially for large scalars like `u256`), and cooperate with SROA.
library;

use ::types::*;

struct SingleField<T> {
    field: T,
}

impl<T> PartialEq for SingleField<T>
where
    T: PartialEq,
{
    fn eq(self, other: Self) -> bool {
        self.field == other.field
    }
}
impl<T> Eq for SingleField<T>
where
    T: PartialEq,
{}

struct InnerWrapper<T> {
    value: SingleField<T>,
}

struct OuterWrapper<T> {
    inner: InnerWrapper<T>,
}

#[test]
fn test_single_field_u8_zero() {
    single_field_u8_zero();
}

#[inline(never)]
pub fn single_field_u8_zero() {
    let t = SingleField { field: 0u8 };
    assert_eq(t.field, 0u8);
}

#[test]
fn test_single_field_u256_zero() {
    single_field_u256_zero();
}

#[inline(never)]
pub fn single_field_u256_zero() {
    let t = SingleField { field: 0u256 };
    assert_eq(t.field, 0u256);
}

#[test]
fn test_single_field_u256_non_zero() {
    single_field_u256_non_zero();
}

#[inline(never)]
pub fn single_field_u256_non_zero() {
    let t = SingleField { field: 42u256 };
    assert_eq(t.field, 42u256);
}

#[test]
fn test_single_field_sroa() {
    single_field_sroa();
}

#[inline(never)]
pub fn single_field_sroa() {
    let t = SingleField { field: 0u256 };
    assert_eq(t.field, 0u256);

    let t = SingleField { field: 42u256 };
    assert_eq(t.field, 42u256);
}

#[test]
fn test_nested_wrappers_zero() {
    nested_wrappers_zero();
}

#[inline(never)]
pub fn nested_wrappers_zero() {
    let t = OuterWrapper {
        inner: InnerWrapper {
            value: SingleField { field: 0u256 },
        },
    };
    assert_eq(t.inner.value.field, 0u256);
}

#[test]
fn test_nested_wrappers_non_zero() {
    nested_wrappers_non_zero();
}

#[inline(never)]
pub fn nested_wrappers_non_zero() {
    let t = OuterWrapper {
        inner: InnerWrapper {
            value: SingleField { field: 42u256 },
        },
    };
    assert_eq(t.inner.value.field, 42u256);
}

#[test]
fn test_nested_wrappers_sroa() {
    nested_wrappers_sroa();
}

#[inline(never)]
pub fn nested_wrappers_sroa() {
    let t = OuterWrapper {
        inner: InnerWrapper {
            value: SingleField { field: 0u256 },
        },
    };
    assert_eq(t.inner.value.field, 0u256);

    let t = OuterWrapper {
        inner: InnerWrapper {
            value: SingleField { field: 42u256 },
        },
    };
    assert_eq(t.inner.value.field, 42u256);
}

#[test]
fn test_single_element_tuple_u8() {
    single_element_tuple_u8();
}

#[inline(never)]
pub fn single_element_tuple_u8() {
    let t = (0u8,);
    assert_eq(t.0, 0u8);

    let t = (42u8,);
    assert_eq(t.0, 42u8);
}
