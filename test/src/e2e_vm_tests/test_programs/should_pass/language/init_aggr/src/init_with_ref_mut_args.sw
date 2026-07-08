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
}

struct SelfCopieable {
    value: u64,
}

impl SelfCopieable {
    #[inline(never)]
    pub fn copy_me(self) {
        let copied = (self, self);
        assert_eq(copied.0.value, self.value);
        assert_eq(copied.1.value, self.value);
    }

    #[inline(never)]
    pub fn copy_me_ref_mut(ref mut self) {
        let copied = (self, self);
        assert_eq(copied.0.value, self.value);
        assert_eq(copied.1.value, self.value);
    }
}

#[test]
fn test_self_copieable_copy_me() {
    let sc = SelfCopieable { value: 123 };
    sc.copy_me();
}

#[test]
fn test_self_copieable_copy_me_ref_mut() {
    let mut sc = SelfCopieable { value: 456 };
    sc.copy_me_ref_mut();
}
