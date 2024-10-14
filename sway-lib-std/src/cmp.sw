//! A utility library for comparing values.
library;

/// A common trait for comparing values.
pub trait Cmp {
    /// Compares and returns the minimum of two values.
    fn min(self, other: Self) -> Self;
    /// Compares and returns the maximum of two values.
    fn max(self, other: Self) -> Self;
}

impl Cmp for u8 {
    pub fn min(self, other: Self) -> Self {
        if self < other {
            self
        } else {
            other
        }
    }

    pub fn max(self, other: Self) -> Self {
        if self > other {
            self
        } else {
            other
        }
    }
}

impl Cmp for u16 {
    pub fn min(self, other: Self) -> Self {
        if self < other {
            self
        } else {
            other
        }
    }

    pub fn max(self, other: Self) -> Self {
        if self > other {
            self
        } else {
            other
        }
    }
}

impl Cmp for u32 {
    pub fn min(self, other: Self) -> Self {
        if self < other {
            self
        } else {
            other
        }
    }

    pub fn max(self, other: Self) -> Self {
        if self > other {
            self
        } else {
            other
        }
    }
}

impl Cmp for u64 {
    pub fn min(self, other: Self) -> Self {
        if self < other {
            self
        } else {
            other
        }
    }

    pub fn max(self, other: Self) -> Self {
        if self > other {
            self
        } else {
            other
        }
    }
}

impl Cmp for U128 {
    pub fn min(self, other: Self) -> Self {
        if self < other {
            self
        } else {
            other
        }
    }

    pub fn max(self, other: Self) -> Self {
        if self > other {
            self
        } else {
            other
        }
    }
}

impl Cmp for U256 {
    pub fn min(self, other: Self) -> Self {
        if self < other {
            self
        } else {
            other
        }
    }

    pub fn max(self, other: Self) -> Self {
        if self > other {
            self
        } else {
            other
        }
    }
}