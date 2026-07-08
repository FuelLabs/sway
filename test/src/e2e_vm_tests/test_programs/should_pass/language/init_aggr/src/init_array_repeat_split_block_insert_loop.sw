//! Large repeat arrays (length > 5) are lowered into an initialization loop.
//! These tests test that path, including several such loops within a
//! single function (multiple block splits in one function body).
library;

use ::types::*;

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
fn test_u64_all_42s_repeat_several_times() {
    u64_all_42s_repeat_several_times();
}

#[inline(never)]
pub fn u64_all_42s_repeat_several_times() {
    let a = [42u64; 10];
    assert_all_elems_equal(a, 42u64);

    let a = [333u64; 10];
    assert_all_elems_equal(a, 333u64);

    let a = [444u64; 10];
    assert_all_elems_equal(a, 444u64);
}

#[test]
fn test_boundary_small_vs_loop() {
    boundary_small_vs_loop();
}

/// The lowering initializes small repeat arrays (length <= 5) with individual
/// stores, and larger ones (length > 5) with a loop. This test covers both
/// sides of that boundary.
#[inline(never)]
pub fn boundary_small_vs_loop() {
    let small = [7u64; 5];

    let mut i = 0;
    while i < 5 {
        assert_eq(small[i], 7u64);
        i += 1;
    }

    let large = [7u64; 6];

    let mut i = 0;
    while i < 6 {
        assert_eq(large[i], 7u64);
        i += 1;
    }
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
