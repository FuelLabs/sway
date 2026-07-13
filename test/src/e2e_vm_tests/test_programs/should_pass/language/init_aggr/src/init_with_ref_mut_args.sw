//! Initialization of aggregates whose initializers are `ref mut` function
//! arguments, and initialization from `self`. These tests `init_aggr`s
//! whose initializer values are pointers to memory that must be correctly
//! loaded/copied.
library;

use ::types::*;

#[test]
fn test_ref_mut_args() {
    let mut a: u64 = 42;
    let mut b: bool = true;
    let mut c: u256 = 42u256;
    let mut d: b256 = b256::zero();
    let mut e: () = ();
    let mut s: Struct = Struct {
        x: 100,
        simple: create_some_simple(10),
        b: false,
    };
    ref_mut_args(a, b, c, d, e, s);
}

#[inline(never)]
pub fn ref_mut_args(
    ref mut a: u64,
    ref mut b: bool,
    ref mut c: u256,
    ref mut d: b256,
    ref mut e: (),
    ref mut s: Struct,
) {
    let t = (a, b, c, d, e, s);

    assert_eq(t.0, a);
    assert_eq(t.1, b);
    assert_eq(t.2, c);
    assert_eq(t.3, d);
    assert_eq(t.4, e);
    assert_eq(t.5, s);

    // The `t.5 == s` assert above compares two `Struct`s that are both created
    // via `init_aggr`. Additionally validate the struct element field-by-field
    // against the primitive literals used by the single caller of this function.
    assert_eq(t.5.x, 100);
    assert_eq(t.5.simple.a, 10);
    assert_eq(t.5.simple.b, 2);
    assert_eq(t.5.simple.c, true);
    assert_eq(t.5.simple.d, 3u256);
    assert_eq(t.5.b, false);
}

struct SelfCopieable {
    value: u64,
}

impl SelfCopieable {
    // `expected` is the primitive literal `value` the caller initialized `self`
    // with. We assert against it in addition to `self.value`, because `self` is
    // itself created via `init_aggr` and asserting only against `self.value`
    // would compare two identically-initialized values.
    #[inline(never)]
    pub fn copy_me(self, expected: u64) {
        let copied = (self, self);
        assert_eq(copied.0.value, self.value);
        assert_eq(copied.1.value, self.value);
        assert_eq(copied.0.value, expected);
        assert_eq(copied.1.value, expected);
    }

    #[inline(never)]
    pub fn copy_me_ref_mut(ref mut self, expected: u64) {
        let copied = (self, self);
        assert_eq(copied.0.value, self.value);
        assert_eq(copied.1.value, self.value);
        assert_eq(copied.0.value, expected);
        assert_eq(copied.1.value, expected);
    }
}

#[test]
fn test_self_copieable_copy_me() {
    let sc = SelfCopieable { value: 123 };
    sc.copy_me(123);
}

#[test]
fn test_self_copieable_copy_me_ref_mut() {
    let mut sc = SelfCopieable { value: 456 };
    sc.copy_me_ref_mut(456);
}
