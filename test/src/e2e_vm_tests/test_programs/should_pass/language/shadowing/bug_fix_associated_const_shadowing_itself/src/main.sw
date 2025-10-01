// This test proves that https://github.com/FuelLabs/sway/issues/7396 is fixed.
library;

use test_asserts::*;

pub trait A {
    const C: bool;
}

pub trait B {
    const C: bool = true;
}

impl A for bool {
    const C: bool = false;
}

impl B for bool {
    const C: bool = false;
}

struct S1<T> { }

impl<T> A for S1<T> where T: A {
    const C: bool = false;
}

impl<T> B for S1<T> where T: B {
}

// TODO: Enable this test once https://github.com/FuelLabs/sway/issues/7396 is actually fixed.
// struct S2A<T> { }

// impl<T> S2A<T> where T: A {
//     const C: bool = true;
// }

struct S2B<T> { }

// Even this was failing before the fix ;-)
impl<T> S2B<T> where T: B {
    fn f() -> bool {
        true
    }
}

struct S3<T> { }

impl<T> A for S3<T> where T: A + B {
    const C: bool = false;
}

impl<T> B for S3<T> where T: A + B {
}

struct S4<T> { }

impl<T> S4<T> where T: A + B {
}

struct S5<T> { }

impl<T> S5<T> {
    fn f() -> bool where T: A + B {
        true
    }
}

struct S6 {}

impl S6 {
    fn f<T>() -> bool where T: A + B {
        true
    }
}

#[test]
fn test() {
    assert_false(1, <bool as A>::C);
    assert_false(2, <bool as B>::C);
    assert_false(3, <S1::<bool> as A>::C);
    assert_true(4, <S1::<bool> as B>::C);
    // TODO: Enable this assert once https://github.com/FuelLabs/sway/issues/7396 is actually fixed.
    // assert_true(5, S2A::<bool>::C);
    assert_true(6, S2B::<bool>::f());
    assert_false(7, <S3::<bool> as A>::C);
    assert_true(8, <S3::<bool> as B>::C);
    assert_true(9, S5::<bool>::f());
    assert_true(10, S6::f::<bool>());
}