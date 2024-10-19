//! A utility library for comparing values.
library;

/// A common trait for comparing values.
pub trait TotalOrd {
    /// Compares and returns the minimum of two values.
    fn min(self, other: Self) -> Self;
    /// Compares and returns the maximum of two values.
    fn max(self, other: Self) -> Self;
}

impl TotalOrd for u8 {
    fn min(self, other: Self) -> Self {
        if self < other { self } else { other }
    }

    fn max(self, other: Self) -> Self {
        if self > other { self } else { other }
    }
}

impl TotalOrd for u16 {
    fn min(self, other: Self) -> Self {
        if self < other { self } else { other }
    }

    fn max(self, other: Self) -> Self {
        if self > other { self } else { other }
    }
}

impl TotalOrd for u32 {
    fn min(self, other: Self) -> Self {
        if self < other { self } else { other }
    }

    fn max(self, other: Self) -> Self {
        if self > other { self } else { other }
    }
}

impl TotalOrd for u64 {
    fn min(self, other: Self) -> Self {
        if self < other { self } else { other }
    }

    fn max(self, other: Self) -> Self {
        if self > other { self } else { other }
    }
}

impl TotalOrd for u256 {
    fn min(self, other: Self) -> Self {
        if self < other { self } else { other }
    }

    fn max(self, other: Self) -> Self {
        if self > other { self } else { other }
    }
}
