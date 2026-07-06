library;

use std::crypto::point2d::*;

#[test]
fn point2d_new() {
    let new_point = Point2D::new();

    assert_eq(new_point.x().len(), 0);
    assert_eq(new_point.y().len(), 0);
}

#[test]
fn point2d_zero() {
    let zero_point = Point2D::zero();

    assert_eq(zero_point.x().len(), 32);
    assert_eq(zero_point.y().len(), 32);

    assert_eq(b256::try_from(zero_point.x()).unwrap(), b256::zero());
    assert_eq(b256::try_from(zero_point.y()).unwrap(), b256::zero());
}

#[test]
fn point2d_is_zero() {
    let zero_point = Point2D::zero();
    assert(zero_point.is_zero());

    let other_point = Point2D::from((b256::zero(), b256::zero()));
    assert(other_point.is_zero());

    let not_zero_point = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    assert(!not_zero_point.is_zero());
}

#[test]
fn point2d_min() {
    let min_point = Point2D::min();

    assert_eq(min_point.x().len(), 32);
    assert_eq(min_point.y().len(), 32);

    assert_eq(b256::try_from(min_point.x()).unwrap(), b256::zero());
    assert_eq(b256::try_from(min_point.y()).unwrap(), b256::zero());
}

#[test]
fn point2d_x() {
    let zero_point = Point2D::zero();

    let zero_x = zero_point.x();
    assert_eq(zero_x.len(), 32);
    assert_eq(zero_x.capacity(), 32);

    let point_1 = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let point_1_x = point_1.x();
    assert_eq(point_1_x.len(), 32);
    assert_eq(point_1_x.capacity(), 32);

    let point_2 = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000001,
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let point_2_x = point_2.x();
    assert_eq(point_2_x.len(), 32);
    assert_eq(point_2_x.capacity(), 32);
}

#[test]
fn point2d_y() {
    let zero_point = Point2D::zero();
    let zero_y = zero_point.y();
    assert_eq(zero_y.len(), 32);
    assert_eq(zero_y.capacity(), 32);

    let point_1 = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let point_1_y = point_1.y();
    assert_eq(point_1_y.len(), 32);
    assert_eq(point_1_y.capacity(), 32);

    let point_2 = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000001,
        0x0000000000000000000000000000000000000000000000000000000000000001,
    ));
    let point_2_y = point_2.y();
    assert_eq(point_2_y.len(), 32);
    assert_eq(point_2_y.capacity(), 32);
}

#[test]
fn point2d_from_u256_array() {
    let min = Point2D::from([
        0x0000000000000000000000000000000000000000000000000000000000000000_u256,
        0x0000000000000000000000000000000000000000000000000000000000000000_u256,
    ]);
    assert_eq(min.x().len(), 32);
    assert_eq(min.y().len(), 32);
    assert_eq(min.x().capacity(), 32);
    assert_eq(min.y().capacity(), 32);
    assert_eq(
        b256::try_from(min.x())
            .unwrap(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert_eq(
        b256::try_from(min.y())
            .unwrap(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let max = Point2D::from([
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
    ]);
    assert_eq(max.x().len(), 32);
    assert_eq(max.y().len(), 32);
    assert_eq(max.x().capacity(), 32);
    assert_eq(max.y().capacity(), 32);
    assert_eq(
        b256::try_from(max.x())
            .unwrap(),
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
    assert_eq(
        b256::try_from(max.y())
            .unwrap(),
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let other = Point2D::from([
        0x0000000000000000000000000000000000000000000000000000000000000000_u256,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
    ]);
    assert_eq(other.x().len(), 32);
    assert_eq(other.y().len(), 32);
    assert_eq(other.x().capacity(), 32);
    assert_eq(other.y().capacity(), 32);
    assert_eq(
        b256::try_from(other.x())
            .unwrap(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert_eq(
        b256::try_from(other.y())
            .unwrap(),
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
}

#[test]
fn point2d_from_b256_array() {
    let min = Point2D::from([
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    ]);
    assert_eq(min.x().len(), 32);
    assert_eq(min.y().len(), 32);
    assert_eq(min.x().capacity(), 32);
    assert_eq(min.y().capacity(), 32);
    assert_eq(
        b256::try_from(min.x())
            .unwrap(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert_eq(
        b256::try_from(min.y())
            .unwrap(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let max = Point2D::from([
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    ]);
    assert_eq(max.x().len(), 32);
    assert_eq(max.y().len(), 32);
    assert_eq(max.x().capacity(), 32);
    assert_eq(max.y().capacity(), 32);
    assert_eq(
        b256::try_from(max.x())
            .unwrap(),
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
    assert_eq(
        b256::try_from(max.y())
            .unwrap(),
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let other = Point2D::from([
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    ]);
    assert_eq(other.x().len(), 32);
    assert_eq(other.y().len(), 32);
    assert_eq(other.x().capacity(), 32);
    assert_eq(other.y().capacity(), 32);
    assert_eq(
        b256::try_from(other.x())
            .unwrap(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert_eq(
        b256::try_from(other.y())
            .unwrap(),
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
}

#[test]
fn point2d_from_u256_tuple() {
    let min = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000000_u256,
        0x0000000000000000000000000000000000000000000000000000000000000000_u256,
    ));
    assert_eq(min.x().len(), 32);
    assert_eq(min.y().len(), 32);
    assert_eq(min.x().capacity(), 32);
    assert_eq(min.y().capacity(), 32);
    assert_eq(
        b256::try_from(min.x())
            .unwrap(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert_eq(
        b256::try_from(min.y())
            .unwrap(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let max = Point2D::from((
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
    ));
    assert_eq(max.x().len(), 32);
    assert_eq(max.y().len(), 32);
    assert_eq(max.x().capacity(), 32);
    assert_eq(max.y().capacity(), 32);
    assert_eq(
        b256::try_from(max.x())
            .unwrap(),
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
    assert_eq(
        b256::try_from(max.y())
            .unwrap(),
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let other = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000000_u256,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
    ));
    assert_eq(other.x().len(), 32);
    assert_eq(other.y().len(), 32);
    assert_eq(other.x().capacity(), 32);
    assert_eq(other.y().capacity(), 32);
    assert_eq(
        b256::try_from(other.x())
            .unwrap(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert_eq(
        b256::try_from(other.y())
            .unwrap(),
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
}

#[test]
fn point2d_from_b256_tuple() {
    let min = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    ));
    assert_eq(min.x().len(), 32);
    assert_eq(min.y().len(), 32);
    assert_eq(min.x().capacity(), 32);
    assert_eq(min.y().capacity(), 32);
    assert_eq(
        b256::try_from(min.x())
            .unwrap(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert_eq(
        b256::try_from(min.y())
            .unwrap(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let max = Point2D::from((
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    ));
    assert_eq(max.x().len(), 32);
    assert_eq(max.y().len(), 32);
    assert_eq(max.x().capacity(), 32);
    assert_eq(max.y().capacity(), 32);
    assert_eq(
        b256::try_from(max.x())
            .unwrap(),
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
    assert_eq(
        b256::try_from(max.y())
            .unwrap(),
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let other = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    ));
    assert_eq(other.x().len(), 32);
    assert_eq(other.y().len(), 32);
    assert_eq(other.x().capacity(), 32);
    assert_eq(other.y().capacity(), 32);
    assert_eq(
        b256::try_from(other.x())
            .unwrap(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert_eq(
        b256::try_from(other.y())
            .unwrap(),
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
}

#[test]
fn point2d_from_u8_array() {
    let min = Point2D::from([
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8,
    ]);
    assert_eq(min.x().len(), 32);
    assert_eq(min.y().len(), 32);
    assert_eq(min.x().capacity(), 32);
    assert_eq(min.y().capacity(), 32);
    assert_eq(
        b256::try_from(min.x())
            .unwrap(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert_eq(
        b256::try_from(min.y())
            .unwrap(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let max = Point2D::from([
        255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
        255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
        255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
        255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
        255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
        255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
    ]);
    assert_eq(max.x().len(), 32);
    assert_eq(max.y().len(), 32);
    assert_eq(max.x().capacity(), 32);
    assert_eq(max.y().capacity(), 32);
    assert_eq(
        b256::try_from(max.x())
            .unwrap(),
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
    assert_eq(
        b256::try_from(max.y())
            .unwrap(),
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let other = Point2D::from([
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        0u8, 0u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
        255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
        255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8, 255u8,
    ]);
    assert_eq(other.x().len(), 32);
    assert_eq(other.y().len(), 32);
    assert_eq(other.x().capacity(), 32);
    assert_eq(other.y().capacity(), 32);
    assert_eq(
        b256::try_from(other.x())
            .unwrap(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert_eq(
        b256::try_from(other.y())
            .unwrap(),
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
}

#[test]
fn point2d_u256_tuple_try_from() {
    let min = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000000_u256,
        0x0000000000000000000000000000000000000000000000000000000000000000_u256,
    ));
    let (x, y) = <(u256, u256) as TryFrom<Point2D>>::try_from(min).unwrap();
    assert_eq(
        x,
        0x0000000000000000000000000000000000000000000000000000000000000000_u256,
    );
    assert_eq(
        y,
        0x0000000000000000000000000000000000000000000000000000000000000000_u256,
    );

    let max = Point2D::from((
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
    ));
    let (x, y) = <(u256, u256) as TryFrom<Point2D>>::try_from(max).unwrap();
    assert_eq(
        x,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
    );
    assert_eq(
        y,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
    );

    let other = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000000_u256,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
    ));
    let (x, y) = <(u256, u256) as TryFrom<Point2D>>::try_from(other).unwrap();
    assert_eq(
        x,
        0x0000000000000000000000000000000000000000000000000000000000000000_u256,
    );
    assert_eq(
        y,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
    );
}

#[test]
fn point2d_u256_array_try_from() {
    let min = Point2D::from([
        0x0000000000000000000000000000000000000000000000000000000000000000_u256,
        0x0000000000000000000000000000000000000000000000000000000000000000_u256,
    ]);
    let array = <[u256; 2] as TryFrom<Point2D>>::try_from(min).unwrap();
    assert_eq(
        array[0],
        0x0000000000000000000000000000000000000000000000000000000000000000_u256,
    );
    assert_eq(
        array[1],
        0x0000000000000000000000000000000000000000000000000000000000000000_u256,
    );

    let max = Point2D::from([
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
    ]);
    let array = <[u256; 2] as TryFrom<Point2D>>::try_from(max).unwrap();
    assert_eq(
        array[0],
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
    );
    assert_eq(
        array[1],
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
    );

    let other = Point2D::from([
        0x0000000000000000000000000000000000000000000000000000000000000000_u256,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
    ]);
    let array = <[u256; 2] as TryFrom<Point2D>>::try_from(other).unwrap();
    assert_eq(
        array[0],
        0x0000000000000000000000000000000000000000000000000000000000000000_u256,
    );
    assert_eq(
        array[1],
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u256,
    );
}

#[test]
fn point2d_b256_tuple_try_from() {
    let min = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    ));
    let (x, y) = <(b256, b256) as TryFrom<Point2D>>::try_from(min).unwrap();
    assert_eq(
        x,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert_eq(
        y,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let max = Point2D::from((
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    ));
    let (x, y) = <(b256, b256) as TryFrom<Point2D>>::try_from(max).unwrap();
    assert_eq(
        x,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
    assert_eq(
        y,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let other = Point2D::from((
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    ));
    let (x, y) = <(b256, b256) as TryFrom<Point2D>>::try_from(other).unwrap();
    assert_eq(
        x,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert_eq(
        y,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
}

#[test]
fn point2d_b256_array_try_from() {
    let min = Point2D::from([
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    ]);
    let array = <[b256; 2] as TryFrom<Point2D>>::try_from(min).unwrap();
    assert_eq(
        array[0],
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert_eq(
        array[1],
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let max = Point2D::from([
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    ]);
    let array = <[b256; 2] as TryFrom<Point2D>>::try_from(max).unwrap();
    assert_eq(
        array[0],
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
    assert_eq(
        array[1],
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let other = Point2D::from([
        0x0000000000000000000000000000000000000000000000000000000000000000,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    ]);
    let array = <[b256; 2] as TryFrom<Point2D>>::try_from(other).unwrap();
    assert_eq(
        array[0],
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert_eq(
        array[1],
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
}

#[test]
fn point2d_codec() {
    let point = Point2D::new();
    log(point);
}
