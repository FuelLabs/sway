library;

use ::vec::*;
use ::bytes::{Bytes, *};
use ::revert::require;
use ::crypto::{point2d::*, scalar::*, errors::ZKError};
use ::alloc::alloc;

/// The curve types supported by the Fuel VM.
pub enum CurveType {
    /// The Alt BN128 curve.
    AltBN128: (),
}

/// Performs an elliptic curve multiplication with a given curve, point, and scalar.
///
/// # Additional Information
///
/// The Fuel VM currently only supports the Alt BN128 curve.
///
/// # Arguments
///
/// * `curve_type`: [CurveType] - The type of curve which the multiplication should be performed.
/// * `point`: [Point2D] - The point used to perform the multiplication.
/// * `scalar`: [Scalar] - The scalar used perform the multiplication.
///
/// # Returns
///
/// * [Point2D] - The resulting computed point.
///
/// # Examples
///
/// ```sway
/// use std::{point2d::Point2D, scalar::Scalar, zk::ec_mul};
///
/// fn foo(curve_type: CurveType, point: Point2D, scalar: Scalar) {
///     let result = ec_mul(curve_type, point, scalar);
///     assert(!result.is_zero());
/// }
/// ```
pub fn ec_mul(curve_type: CurveType, point: Point2D, scalar: Scalar) -> Point2D {
    let curve = match curve_type {
        CurveType::AltBN128 => {
            require(valid_alt_bn128_point(point), ZKError::InvalidEllipticCurvePoint);
            require(valid_alt_bn128_scalar(scalar), ZKError::InvalidEllipticCurveScalar);

            0
        }
    };

    // 1P = ([32 bytes], [32 bytes]) 
    let mut result = [b256::zero(), b256::zero()];
    // 1P1S = (X, Y), Z = ([32 bytes], [32 bytes]), [32 bytes] = 3 * 32 bytes
    let mut ptr = alloc::<b256>(3);
    point.x().ptr().copy_to::<b256>(ptr.add::<b256>(0), 1);
    point.y().ptr().copy_to::<b256>(ptr.add::<b256>(1), 1);
    scalar.bytes().ptr().copy_to::<b256>(ptr.add::<b256>(2), 1);

    asm(buffer: result, curve: curve, operation: 1, scalar: ptr) {
        ecop buffer curve operation scalar;
    };

    Point2D::from(result)
}

/// Performs an elliptic curve additions with a given curve and 2 points.
///
/// # Additional Information
///
/// The Fuel VM currently only supports the Alt BN128 curve.
///
/// # Arguments
///
/// * `curve_type`: [CurveType] - The type of curve which the addition should be performed.
/// * `point_1`: [Point2D] - The first point used to perform the addition.
/// * `point_2`: [Point2D] - The second point used to perform the addition.
///
/// # Returns
///
/// * [Point2D] - The resulting computed point.
///
/// # Examples
///
/// ```sway
/// use std::{point2d::Point2D, scalar::Scalar, zk::ec_mul};
///
/// fn foo(curve_type: CurveType, point_1: Point2D, point_2: Point2D) {
///     let result = ec_add(curve_type, point_1, point_2);
///     assert(!result.is_zero());
/// }
/// ```
pub fn ec_add(curve_type: CurveType, point_1: Point2D, point_2: Point2D) -> Point2D {
    let curve = match curve_type {
        CurveType::AltBN128 => {
            require(valid_alt_bn128_point(point_1), ZKError::InvalidEllipticCurvePoint);
            require(valid_alt_bn128_point(point_2), ZKError::InvalidEllipticCurvePoint);

            0
        }
    };

    // 1P = ([32 bytes], [32 bytes]) 
    let mut result = [b256::zero(), b256::zero()];
    // 1P1P = (X, Y), (X, Y) = ([32 bytes], [32 bytes]), ([32 bytes], [32 bytes]) = 4 * 32 bytes
    let mut points_ptr = alloc::<b256>(4);
    point_1.x().ptr().copy_to::<b256>(points_ptr.add::<b256>(0), 1);
    point_1.y().ptr().copy_to::<b256>(points_ptr.add::<b256>(1), 1);
    point_2.x().ptr().copy_to::<b256>(points_ptr.add::<b256>(2), 1);
    point_2.y().ptr().copy_to::<b256>(points_ptr.add::<b256>(3), 1);

    asm(buffer: result, curve: curve, operation: 0, points: points_ptr) {
        ecop buffer curve operation points;
    };

    Point2D::from(result)
}

/// Performs an elliptic curve paring check with a given curve and 3 points.
///
/// # Additional Information
///
/// The Fuel VM currently only supports the Alt BN128 curve.
///
/// # Arguments
///
/// * `curve_type`: [CurveType] - The type of curve which the addition should be performed.
/// * `points`: [Vec<(Point2D, [Point2D; 2])>] - The points used to perform the pairing check.
///
/// # Returns
///
/// * [bool] - True if the pairing is valid, false otherwise.
///
/// # Examples
///
/// ```sway
/// use std::{point2d::Point2D, scalar::Scalar, zk::ec_mul};
///
/// fn foo(curve_type: CurveType, points: Vec<(Point2D, [Point2D; 2])>) {
///     let result = ec_pairing_check(curve_type, points);
///     assert(result);
/// }
/// ```
pub fn ec_pairing_check(curve_type: CurveType, points: Vec<(Point2D, [Point2D; 2])>) -> bool {
    // Total bytes is (P1, (G1, G2)) = ([32 bytes, 32 bytes], ([32 bytes, 32 bytes], [32 bytes, 32 bytes])) = 6 * 32 bytes * length
    let mut points_ptr = alloc::<b256>(points.len() * 6);

    let curve = match curve_type {
        CurveType::AltBN128 => {
            let mut iter = 0;
            while iter < points.len() {
                let p1 = points.get(iter).unwrap().0;
                let p2 = points.get(iter).unwrap().1[0];
                let p3 = points.get(iter).unwrap().1[1];

                require(valid_alt_bn128_point(p1), ZKError::InvalidEllipticCurvePoint);
                require(valid_alt_bn128_point(p2), ZKError::InvalidEllipticCurvePoint);
                require(valid_alt_bn128_point(p3), ZKError::InvalidEllipticCurvePoint);

                // Copy all 6 32 byte length points to the single slice
                p1.x().ptr().copy_to::<b256>(points_ptr.add::<b256>(iter * 6), 1);
                p1.y().ptr().copy_to::<b256>(points_ptr.add::<b256>((iter * 6) + 1), 1);
                p2.x().ptr().copy_to::<b256>(points_ptr.add::<b256>((iter * 6) + 2), 1);
                p2.y().ptr().copy_to::<b256>(points_ptr.add::<b256>((iter * 6) + 3), 1);
                p3.x().ptr().copy_to::<b256>(points_ptr.add::<b256>((iter * 6) + 4), 1);
                p3.y().ptr().copy_to::<b256>(points_ptr.add::<b256>((iter * 6) + 5), 1);

                iter += 1;
            }

            0
        }
    };

    // Result is bool
    asm(buffer, curve: curve, length: points.len(), points: points_ptr) {
        epar buffer curve length points;
        buffer: bool
    }
}

// Returns true if the point is in valid alt bn128 format.
fn valid_alt_bn128_point(point: Point2D) -> bool {
    // 1P = ([32 bytes], [32 bytes])
    point.x().len() == 32 && point.y().len() == 32

    // y**2 = x**3 + 3
    // p = 21888242871839275222246405745257275088696311157297823662689037894645226208583 = 0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47

    // let p = u256::from(0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47);
    // let res = <(u256, u256) as TryFrom<Point2D>>::try_from(point);
    // let (x, y) = match res {
    //     Some((x, y)) => (x, y),
    //     None => return false,
    // };

    // // Ensure x and y are within the field range
    // if x > p || y > p {
    //     return false;
    // }

    // // Compute y^2 mod p    
    // let y_squared = (y * y);
    // // // Compute x^3 + 3
    // let x_cubed = (x * x * x);

    // y_squared == (x_cubed + 3) % p 
}

// Returns true if the scalar is in valid alt bn128 format.
fn valid_alt_bn128_scalar(scalar: Scalar) -> bool {
    // 1S = [32 bytes]
    scalar.bytes().len() == 32
}
