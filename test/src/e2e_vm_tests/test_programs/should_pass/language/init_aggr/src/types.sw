//! Shared types and helper functions used across the `init_aggr` test modules.
library;

pub struct EmptyStruct {}

impl PartialEq for EmptyStruct {
    fn eq(self, other: Self) -> bool {
        true
    }
}
impl Eq for EmptyStruct {}

pub struct EmptyStructContainer {
    pub e: EmptyStruct,
}

impl PartialEq for EmptyStructContainer {
    fn eq(self, other: Self) -> bool {
        self.e == other.e
    }
}
impl Eq for EmptyStructContainer {}


#[inline(never)]
pub fn return_empty_struct() -> EmptyStruct {
    asm() {}; // To forbid const-eval.
    EmptyStruct {}
}

/// A struct with no nested aggregates. Covers a mix of small and large scalar
/// fields, as well as a zero-sized `()` field.
pub struct NoNesting {
    pub a: u64,
    pub b: bool,
    pub c: u256,
    pub d: b256,
    pub u: (),
}

impl NoNesting {
    pub fn default() -> Self {
        Self {
            a: 0,
            b: false,
            c: 0u256,
            d: b256::zero(),
            u: (),
        }
    }
}

impl PartialEq for NoNesting {
    fn eq(self, other: Self) -> bool {
        self.a == other.a && self.b == other.b && self.c == other.c && self.d == other.d && self.u == other.u
    }
}
impl Eq for NoNesting {}

/// A struct that nests two `NoNesting` structs. Covers nested `init_aggr`s.
pub struct Nested {
    pub n1: NoNesting,
    pub n2: NoNesting,
}

pub struct Simple {
    pub a: u8,
    pub b: u64,
    pub c: bool,
    pub d: u256,
}

#[inline(never)]
pub fn create_some_simple(a: u8) -> Simple {
    Simple {
        a,
        b: 2,
        c: true,
        d: 3u256,
    }
}

pub struct Struct {
    pub x: u64,
    pub simple: Simple,
    pub b: bool,
}

impl PartialEq for Struct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x && self.simple.a == other.simple.a && self.simple.b == other.simple.b && self.simple.c == other.simple.c && self.simple.d == other.simple.d && self.b == other.b
    }
}
impl Eq for Struct {}

#[inline(never)]
pub fn assert_no_nesting_all_zeros(s: NoNesting) {
    assert_eq(s.a, 0);
    assert_eq(s.b, false);
    assert_eq(s.c, 0u256);
    assert_eq(s.d, b256::zero());
    assert_eq(s.u, ());
}

#[inline(never)]
pub fn assert_no_nesting(s: NoNesting, a: u64, b: bool, c: u256, d: b256) {
    assert_eq(s.a, a);
    assert_eq(s.b, b);
    assert_eq(s.c, c);
    assert_eq(s.d, d);
    assert_eq(s.u, ());
}

#[inline(never)]
pub fn assert_simple_all_zeros(s: Simple) {
    assert_eq(s.a, 0);
    assert_eq(s.b, 0);
    assert_eq(s.c, false);
    assert_eq(s.d, 0u256);
}

#[inline(never)]
pub fn assert_simple(s: Simple, a: u8, b: u64, c: bool, d: u256) {
    assert_eq(s.a, a);
    assert_eq(s.b, b);
    assert_eq(s.c, c);
    assert_eq(s.d, d);
}

#[inline(never)]
pub fn assert_no_nesting_tuple_all_zeros(t: (u64, bool, u256, b256, ())) {
    assert_eq(t.0, 0);
    assert_eq(t.1, false);
    assert_eq(t.2, 0u256);
    assert_eq(t.3, b256::zero());
    assert_eq(t.4, ());
}

#[inline(never)]
pub fn assert_no_nesting_tuple(
    t: (u64, bool, u256, b256, ()),
    a: u64,
    b: bool,
    c: u256,
    d: b256,
) {
    assert_eq(t.0, a);
    assert_eq(t.1, b);
    assert_eq(t.2, c);
    assert_eq(t.3, d);
    assert_eq(t.4, ());
}
