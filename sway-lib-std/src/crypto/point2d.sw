library;

use ::convert::{From, TryFrom};
use ::bytes::{Bytes, *};
use ::option::Option::{self, *};
use ::primitive_conversions::u256::*;

pub struct Point2D {
    x: Bytes,
    y: Bytes,
}

impl Eq for Point2D {
    fn eq(self, other: Self) -> bool {
        // All points must be of length 32
        if self.x.len() != 32 || self.y.len() != 32 || other.x.len() != 32 || other.y.len() != 32 {
            return false;
        }

        let mut iter = 0;
        while iter < 32 {
            if self.x.get(iter).unwrap() != other.x.get(iter).unwrap() {
                return false;
            } else if self.y.get(iter).unwrap() != other.y.get(iter).unwrap() {
                return false;
            }

            iter += 1;
        }
        true
    }
}

impl Point2D {
    pub fn new() -> Self {
        Self {
            x: Bytes::new(),
            y: Bytes::new(),
        }
    }

    pub fn zero() -> Self {
        Self {
            x: Bytes::from(b256::zero()),
            y: Bytes::from(b256::zero()),
        }
    }

    pub fn is_zero(self) -> bool {
        self == Self::zero()
    }

    pub fn min() -> Self {
        Self {
            x: Bytes::from(b256::zero()),
            y: Bytes::from(b256::zero()),
        }
    }

    pub fn max() -> Self {
        Self {
            x: Bytes::from(b256::max()),
            y: Bytes::from(b256::max()),
        }
    }

    pub fn x(self) -> Bytes {
        self.x
    }

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
        let mut x = Bytes::with_capacity(32);
        let mut y = Bytes::with_capacity(32);

        let mut iter = 0;
        while iter < 32 {
            x.push(bytes[iter]);
            y.push(bytes[iter + 32]);
            iter += 1;
        }

        Self {
            x: x,
            y: y,
        }
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
