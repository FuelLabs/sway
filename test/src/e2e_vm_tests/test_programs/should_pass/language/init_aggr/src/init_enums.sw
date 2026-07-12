//! Initialization of aggregates that contain enums. Enums are themselves not
//! initialized via `init_aggr`, but they can appear as fields/elements of
//! structs, tuples, and arrays that are. The lowering must treat an enum field
//! as an opaque value (for zero analysis it is a non-`init_aggr` leaf).
library;

use ::types::*;

enum E {
    Unit: (),
    Num: u64,
    Pair: (u64, u256),
}

impl PartialEq for E {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (E::Unit, E::Unit) => true,
            (E::Num(a), E::Num(b)) => a == b,
            (E::Pair(a), E::Pair(b)) => a.0 == b.0 && a.1 == b.1,
            _ => false,
        }
    }
}
impl Eq for E {}

struct HasEnum {
    tag: u64,
    e: E,
    trailing: u64,
}

#[test]
fn test_struct_with_enum_unit() {
    struct_with_enum_unit();
}

#[inline(never)]
pub fn struct_with_enum_unit() {
    let s = HasEnum {
        tag: 0,
        e: E::Unit,
        trailing: 0,
    };

    assert_eq(s.tag, 0);
    assert_eq(s.e, E::Unit);
    assert_eq(s.trailing, 0);
}

#[test]
fn test_struct_with_enum_num() {
    struct_with_enum_num();
}

#[inline(never)]
pub fn struct_with_enum_num() {
    let s = HasEnum {
        tag: 7,
        e: E::Num(42),
        trailing: 9,
    };

    assert_eq(s.tag, 7);
    assert_eq(s.e, E::Num(42));
    assert_eq(s.trailing, 9);
}

#[test]
fn test_tuple_with_enum() {
    tuple_with_enum();
}

#[inline(never)]
pub fn tuple_with_enum() {
    let t = (0u64, E::Pair((1, 2u256)), 0u64);

    assert_eq(t.0, 0);
    assert_eq(t.1, E::Pair((1, 2u256)));
    assert_eq(t.2, 0);

    // The `E::Pair` payload is a tuple built via `init_aggr`, and the expected
    // `E::Pair((1, 2u256))` above builds it the same way. Additionally validate
    // the payload field-by-field against primitive literals.
    match t.1 {
        E::Pair(p) => {
            assert_eq(p.0, 1u64);
            assert_eq(p.1, 2u256);
        },
        _ => assert(false),
    }
}

#[test]
fn test_array_of_enums() {
    array_of_enums();
}

#[inline(never)]
pub fn array_of_enums() {
    let a = [E::Num(1), E::Num(2), E::Num(3)];

    assert(a[0] == E::Num(1));
    assert(a[1] == E::Num(2));
    assert(a[2] == E::Num(3));
}

#[test]
fn test_array_of_enums_repeat() {
    array_of_enums_repeat();
}

#[inline(never)]
pub fn array_of_enums_repeat() {
    let a = [E::Num(5); 10];

    let mut i = 0;
    while i < 10 {
        assert(a[i] == E::Num(5));
        i += 1;
    }
}

#[test]
fn test_option_in_struct() {
    option_in_struct();
}

// Test a mostly-zeroed struct that holds `Option`.
#[inline(never)]
pub fn option_in_struct() {
    let s = HasEnumOption {
        a: 0,
        opt: Some(42u64),
        b: 0,
    };

    assert_eq(s.a, 0);
    assert_eq(s.opt.unwrap(), 42);
    assert_eq(s.b, 0);

    let s = HasEnumOption {
        a: 0,
        opt: None,
        b: 0,
    };

    assert_eq(s.a, 0);
    assert(s.opt.is_none());
    assert_eq(s.b, 0);
}

struct HasEnumOption {
    a: u64,
    opt: Option<u64>,
    b: u64,
}
