//! Regression tests for issues found while developing the `init_aggr`
//! optimization. These reproduce const-evaluation and lowering bugs around
//! initialization of tuples containing large scalars (`u256`) built from
//! runtime values, including through `std::crypto::point2d`.
library;

use ::types::*;
use std::crypto::point2d::*;

// Reproduces a const-eval issue where a tuple of `u256`s was initialized from
// values copied out of a `Point2D` (i.e. from runtime pointers), wrapped in an
// `Option`.
#[test]
fn test_const_eval_issue_extracted() {
    // Use `zero()` (not `new()`): `zero()` produces valid 32-byte zero buffers,
    // whereas `new()` produces empty `Bytes` and copying from them reads out of
    // bounds. The initialization pattern being exercised (a tuple of `u256`s
    // built from runtime pointer copies, wrapped in an `Option`) is the same.
    let point = Point2D::zero();

    let mut value_x = 0x0000000000000000000000000000000000000000000000000000000000000000_u256;
    let mut value_y = 0x0000000000000000000000000000000000000000000000000000000000000000_u256;
    let ptr_x = __addr_of(value_x);
    let ptr_y = __addr_of(value_y);

    point.x().ptr().copy_to::<u256>(ptr_x, 1);
    point.y().ptr().copy_to::<u256>(ptr_y, 1);

    let x = Some((value_x, value_y));

    assert_eq(x.unwrap().0, value_x);
    assert_eq(x.unwrap().1, value_y);
    assert_eq(x.unwrap().0, 0u256);
    assert_eq(x.unwrap().1, 0u256);
}

// Reproduces an issue where a `(u256, u256)` tuple produced by `TryFrom` for a
// `Point2D` was incorrectly initialized.
#[test]
fn test_point2d_u256_tuple_try_from() {
    let max = Point2D::from((
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
    ));

    let (x, y) = <(u256, u256) as TryFrom<Point2D>>::try_from(max).unwrap();

    assert_eq(x, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256);
    assert_eq(y, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256);
}
