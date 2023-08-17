library;

impl u64 {
    /// The smallest value that can be represented by this integer type.
    pub fn min() -> Self {
        0
    }

    /// The largest value that can be represented by this integer type,
    /// 2<sup>64</sup> - 1.
    pub fn max() -> Self {
        18446744073709551615
    }

    /// The size of this integer type in bits.
    pub fn bits() -> u64 {
        64
    }
}

impl u32 {
    /// The smallest value that can be represented by this integer type.
    pub fn min() -> Self {
        0
    }

    /// The largest value that can be represented by this integer type,
    /// 2<sup>32</sup> - 1.
    pub fn max() -> Self {
        4294967295
    }

    /// The size of this integer type in bits.
    pub fn bits() -> u64 {
        32
    }
}

impl u16 {
    /// The smallest value that can be represented by this integer type.
    pub fn min() -> Self {
        0
    }

    /// The largest value that can be represented by this integer type,
    /// 2<sup>16</sup> - 1.
    pub fn max() -> Self {
        65535
    }

    /// The size of this integer type in bits.
    pub fn bits() -> u64 {
        16
    }
}

impl u8 {
    /// The smallest value that can be represented by this integer type.
    pub fn min() -> Self {
        0
    }

    /// The largest value that can be represented by this integer type,
    /// 2<sup>8</sup> - 1.
    pub fn max() -> Self {
        255
    }

    /// The size of this integer type in bits.
    pub fn bits() -> u64 {
        8
    }
}

impl b256 {
    /// The smallest value that can be represented by this type.
    pub fn min() -> Self {
        0x0000000000000000000000000000000000000000000000000000000000000000
    }

    /// The largest value that can be represented by this type,
    /// 2<sup>256</sup> - 1.
    pub fn max() -> Self {
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
    }

    /// The size of this type in bits.
    pub fn bits() -> u64 {
        256
    }
}
