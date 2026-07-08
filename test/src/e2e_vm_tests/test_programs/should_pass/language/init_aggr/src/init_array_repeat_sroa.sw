//! Initialization of repeat arrays where only a single element is later
//! accessed. This exercises the interaction between the `init_aggr` lowering
//! and SROA. Covers both the "large" (loop-lowered) and "small" repeat arrays.
library;

use ::types::*;

#[test]
fn test_u64_all_zeros_repeat() {
    u64_all_zeros_repeat();
}

#[inline(never)]
pub fn u64_all_zeros_repeat() {
    let a = [0u64; 10];

    assert_eq(a[0], 0u64);
}

#[test]
fn test_u64_all_zeros_small_array_repeat() {
    u64_all_zeros_small_array_repeat();
}

#[inline(never)]
pub fn u64_all_zeros_small_array_repeat() {
    let a = [0u64; 4];

    assert_eq(a[0], 0u64);
}

#[test]
fn test_u64_all_42s_repeat() {
    u64_all_42s_repeat();
}

#[inline(never)]
pub fn u64_all_42s_repeat() {
    let a = [42u64; 10];

    assert_eq(a[0], 42u64);
}

#[test]
fn test_u64_all_42s_small_array_repeat() {
    u64_all_42s_small_array_repeat();
}

#[inline(never)]
pub fn u64_all_42s_small_array_repeat() {
    let a = [42u64; 4];

    assert_eq(a[0], 42u64);
}
