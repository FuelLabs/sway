library num;

impl u64 {
    /// The smallest value that can be represented by this integer type.
    pub fn MIN() -> u64 {
        0
    }

    /// The largest value that can be represented by this integer type,
    /// 2<sup>64</sup> - 1.
    pub fn MAX() -> u64 {
        18446744073709551615
    }

    /// The size of this integer type in bits.
    pub fn BITS() -> u32 {
        64
    }
}

impl u32 {
    /// The smallest value that can be represented by this integer type.
    pub fn MIN() -> u32 {
        0
    }

    /// The largest value that can be represented by this integer type,
    /// 2<sup>32</sup> - 1.
    pub fn MAX() -> u32 {
        4294967295
    }

    /// The size of this integer type in bits.
    pub fn BITS() -> u32 {
        32
    }
}

impl u16 {
    /// The smallest value that can be represented by this integer type.
    pub fn MIN() -> u16 {
        0
    }

    /// The largest value that can be represented by this integer type,
    /// 2<sup>16</sup> - 1.
    pub fn MAX() -> u16 {
        65535
    }

    /// The size of this integer type in bits.
    pub fn BITS() -> u32 {
        16
    }
}

impl u8 {
    /// The smallest value that can be represented by this integer type.
    pub fn MIN() -> u8 {
        0
    }

    /// The largest value that can be represented by this integer type,
    /// 2<sup>8</sup> - 1.
    pub fn MAX() -> u8 {
        255
    }

    /// The size of this integer type in bits.
    pub fn BITS() -> u32 {
        8
    }
}
