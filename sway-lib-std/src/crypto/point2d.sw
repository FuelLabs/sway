library;

use ::convert::{From, TryFrom};
use ::bytes::{Bytes, *};
use ::option::Option::{self, *};
use ::hash::{Hash, Hasher};
use ::ops::*;
use ::primitive_conversions::u256::*;
use ::codec::*;
use ::debug::*;

// NOTE: Bytes are used to support numbers greater than 32 bytes for future curves.
/// A 2D point on a field.
///
/// # Additional Information
///
/// The Point2D type only supports positive integer points.
pub struct Point2D {
    /// The x point on the field.
    x: Bytes,
    /// The y point on the field.
    y: Bytes,
}

// All points must be of length 32
impl PartialEq for Point2D {
    fn eq(self, other: Self) -> bool {
        self.x.len() == 32 && self.y.len() == 32 && self.x == other.x && self.y == other.y
    }
}
// Note that `Point2D` implements `PartialEq` but not `Eq`,
// because an uninitialized `Point2D`, created by `Point2D::new`
// is not equal to any other point, including itself.

impl Hash for Point2D {
    fn hash(self, ref mut state: Hasher) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl Point2D {
    /// Returns a new, uninitialized Point2D.
    ///
    /// # Returns
    ///
    /// * [Point2D] - The new Point2D.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::point2d::Point2D;
    ///
    /// fn foo() {
    ///     let new_point = Point2D::new();
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            x: Bytes::new(),
            y: Bytes::new(),
        }
    }

    /// Returns a zeroed Point2D.
    ///
    /// # Returns
    ///
    /// * [Point2D] - The new zeroed Point2D.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::point2d::Point2D;
    ///
    /// fn foo() {
    ///     let zero_point = Point2D::zero();
    ///     assert(b256::try_from(new_point.x()).unwrap() == b256::zero());
    ///     assert(b256::try_from(new_point.y()).unwrap() == b256::zero());
    /// }
    /// ```
    pub fn zero() -> Self {
        Self {
            x: Bytes::from(b256::zero()),
            y: Bytes::from(b256::zero()),
        }
    }

    /// Returns true if the point is (0, 0), otherwise false.
    ///
    /// # Returns
    ///
    // * [bool] - The boolean representing whether the point is zero.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::point2d::Point2D;
    ///
    /// fn foo() {
    ///     let zero_point = Point2D::zero();
    ///     assert(zero_point.is_zero());
    /// }
    /// ```
    pub fn is_zero(self) -> bool {
        // Note that we could simply return `self == Self::zero()` here,
        // but this would cause creating a new `Point2D` zero instance
        // every time we call this function. `Self::zero()` is expensive
        // both in terms of gas and allocated memory.
        // In cases like calling this function in a loop, the performance
        // impact would be significant.
        self.x.len() == 32 && self.y.len() == 32 && self.x.are_all_zero() && self.y.are_all_zero()
    }

    /// Returns the minimum point.
    ///
    /// # Returns
    ///
    /// * [Point2D] - The new minimum Point2D.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::point2d::Point2D;
    ///
    /// fn foo() {
    ///     let zero_point = Point2D::zero();
    ///     assert(b256::try_from(new_point.x()).unwrap() == b256::zero());
    ///     assert(b256::try_from(new_point.y()).unwrap() == b256::zero());
    /// }
    /// ```
    pub fn min() -> Self {
        Self {
            x: Bytes::from(b256::zero()),
            y: Bytes::from(b256::zero()),
        }
    }

    /// Returns the underlying x point as bytes.
    ///
    /// # Returns
    ///
    /// * [Bytes] - The x point represented as bytes.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::point2d::Point2D;
    ///
    /// fn foo(point: Point2D) {
    ///     let x = point.x();
    ///     assert(x.len() != 0);
    /// }
    /// ```
    pub fn x(self) -> Bytes {
        self.x
    }

    /// Returns the underlying y point as bytes.
    ///
    /// # Returns
    ///
    /// * [Bytes] - The y point represented as bytes.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::point2d::Point2D;
    ///
    /// fn foo(point: Point2D) {
    ///     let y = point.y();
    ///     assert(y.len() != 0);
    /// }
    /// ```
    pub fn y(self) -> Bytes {
        self.y
    }
}

impl From<[u256; 2]> for Point2D {
    fn from(bytes: [u256; 2]) -> Self {
        Self {
            x: Bytes::from(bytes[0].as_b256()),
            y: Bytes::from(bytes[1].as_b256()),
        }
    }
}

impl From<[b256; 2]> for Point2D {
    fn from(bytes: [b256; 2]) -> Self {
        Self {
            x: Bytes::from(bytes[0]),
            y: Bytes::from(bytes[1]),
        }
    }
}

impl From<(b256, b256)> for Point2D {
    fn from(bytes: (b256, b256)) -> Self {
        Self {
            x: Bytes::from(bytes.0),
            y: Bytes::from(bytes.1),
        }
    }
}

impl From<(u256, u256)> for Point2D {
    fn from(bytes: (u256, u256)) -> Self {
        Self {
            x: Bytes::from(bytes.0.as_b256()),
            y: Bytes::from(bytes.1.as_b256()),
        }
    }
}

impl From<[u8; 64]> for Point2D {
    fn from(bytes: [u8; 64]) -> Self {
        // TODO: Once const generics are available directly call `From<[u8; N]>` on `Bytes`
        //       instead of having a loop.
        //       Use allocation free `asm` cast to convert the single array to two arrays.
        let mut x = Bytes::with_capacity(32);
        let mut y = Bytes::with_capacity(32);

        let mut iter = 0;
        while iter < 32 {
            x.push(bytes[iter]);
            y.push(bytes[iter + 32]);
            iter += 1;
        }

        Self { x: x, y: y }
    }
}

impl TryFrom<Point2D> for (u256, u256) {
    /// # Example
    ///
    /// ```sway
    /// fn foo(point: Point2D) {
    ///     let (x, y) = <(u256, u256) as TryFrom<Point2D>>::try_from(point).unwrap();
    /// }
    /// ```
    fn try_from(point: Point2D) -> Option<Self> {
        if point.x.len() != 32 || point.y.len() != 32 {
            return None;
        }

        let mut value_x = 0x0000000000000000000000000000000000000000000000000000000000000000_u256;
        let mut value_y = 0x0000000000000000000000000000000000000000000000000000000000000000_u256;
        let ptr_x = __addr_of(value_x);
        let ptr_y = __addr_of(value_y);

        point.x.ptr().copy_to::<u256>(ptr_x, 1);
        point.y.ptr().copy_to::<u256>(ptr_y, 1);

        Some((value_x, value_y))
    }
}

impl TryFrom<Point2D> for [u256; 2] {
    /// # Example
    ///
    /// ```sway
    /// fn foo(point: Point2D) {
    ///     let array = <[u256; 2] as TryFrom<Point2D>>::try_from(point).unwrap();
    /// }
    /// ```
    fn try_from(point: Point2D) -> Option<Self> {
        if point.x.len() != 32 || point.y.len() != 32 {
            return None;
        }

        let mut value_x = 0x0000000000000000000000000000000000000000000000000000000000000000_u256;
        let mut value_y = 0x0000000000000000000000000000000000000000000000000000000000000000_u256;
        let ptr_x = __addr_of(value_x);
        let ptr_y = __addr_of(value_y);

        point.x.ptr().copy_to::<u256>(ptr_x, 1);
        point.y.ptr().copy_to::<u256>(ptr_y, 1);

        Some([value_x, value_y])
    }
}

impl TryFrom<Point2D> for (b256, b256) {
    /// # Example
    ///
    /// ```sway
    /// fn foo(point: Point2D) {
    ///     let (x, y) = <(b256, b256) as TryFrom<Point2D>>::try_from(point).unwrap();
    /// }
    /// ```
    fn try_from(point: Point2D) -> Option<Self> {
        if point.x.len() != 32 || point.y.len() != 32 {
            return None;
        }

        let mut value_x = 0x0000000000000000000000000000000000000000000000000000000000000000;
        let mut value_y = 0x0000000000000000000000000000000000000000000000000000000000000000;
        let ptr_x = __addr_of(value_x);
        let ptr_y = __addr_of(value_y);

        point.x.ptr().copy_to::<b256>(ptr_x, 1);
        point.y.ptr().copy_to::<b256>(ptr_y, 1);

        Some((value_x, value_y))
    }
}

impl TryFrom<Point2D> for [b256; 2] {
    /// # Example
    ///
    /// ```sway
    /// fn foo(point: Point2D) {
    ///     let array = <[b256; 2] as TryFrom<Point2D>>::try_from(point).unwrap();
    /// }
    /// ```
    fn try_from(point: Point2D) -> Option<Self> {
        if point.x.len() != 32 || point.y.len() != 32 {
            return None;
        }

        let mut value_x = 0x0000000000000000000000000000000000000000000000000000000000000000;
        let mut value_y = 0x0000000000000000000000000000000000000000000000000000000000000000;
        let ptr_x = __addr_of(value_x);
        let ptr_y = __addr_of(value_y);

        point.x.ptr().copy_to::<b256>(ptr_x, 1);
        point.y.ptr().copy_to::<b256>(ptr_y, 1);

        Some([value_x, value_y])
    }
}
