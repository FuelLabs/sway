library;

impl u64 {
    /// The smallest value that can be represented by this integer type.
    ///
    /// # Returns
    ///
    /// * [u64] - The smallest `u64` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = u64::min();
    ///     assert(val == 0);
    // }
    /// ```
    pub fn min() -> Self {
        0
    }

    /// The largest value that can be represented by this integer type,
    /// 2<sup>64</sup> - 1.
    ///
    /// # Returns
    ///
    /// * [u64] - The largest `u64` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = u64::max();
    ///     assert(val == 18446744073709551615);
    /// }
    /// ```
    pub fn max() -> Self {
        18446744073709551615
    }

    /// The size of this integer type in bits.
    ///
    /// # Returns
    ///
    /// * [u32] - The number of bits for a `u64`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let bits = u64::bits();
    ///     assert(bits == 64);
    /// }
    /// ```
    pub fn bits() -> u64 {
        64
    }
}

impl u32 {
    /// The smallest value that can be represented by this integer type.
    ///
    /// # Returns
    ///
    /// * [u32] - The smallest `u32` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val == u32::min();
    ///     assert(val == 0u32);
    /// }
    /// ```
    pub fn min() -> Self {
        0
    }

    /// The largest value that can be represented by this integer type,
    /// 2<sup>32</sup> - 1.
    ///
    /// # Returns
    ///
    /// * [u32] - The largest `u32` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val == u32:max();
    ///     assert(val == 4294967295u32);
    /// }
    /// ```
    pub fn max() -> Self {
        4294967295
    }

    /// The size of this integer type in bits.
    ///
    /// # Returns
    ///
    /// * [u32] - The number of bits for a `u32`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let bits = u32::bits();
    ///     assert(bits == 32);
    /// }
    /// ```
    pub fn bits() -> u64 {
        32
    }
}

impl u16 {
    /// The smallest value that can be represented by this integer type.
    ///
    /// # Returns
    ///
    /// * [u16] - The smallest `u16` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = u16::min();
    ///     assert(val == 0u16);
    /// }
    /// ```
    pub fn min() -> Self {
        0
    }

    /// The largest value that can be represented by this integer type,
    /// 2<sup>16</sup> - 1.
    ///
    /// # Returns
    ///
    /// * [u16] - The largest `u16` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = u16::max();
    ///     assert(val == 65535u16);
    /// }
    /// ```
    pub fn max() -> Self {
        65535
    }

    /// The size of this integer type in bits.
    ///
    /// # Returns
    ///
    /// * [u32] - The number of bits for a `u16`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let bits = u16::bits();
    ///     assert(bits == 16);
    /// }
    /// ```
    pub fn bits() -> u64 {
        16
    }
}

impl u8 {
    /// The smallest value that can be represented by this integer type.
    ///
    /// # Returns
    ///
    /// * [u8] - The smallest `u8` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = u8::min();
    ///     assert(val == 0u8);
    /// }
    /// ```
    pub fn min() -> Self {
        0
    }

    /// The largest value that can be represented by this integer type,
    /// 2<sup>8</sup> - 1.
    ///
    /// # Returns
    ///
    /// * [u8] - The largest `u8` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = u8::max();
    ///     assert(val == 255u8);
    /// }
    /// ```
    pub fn max() -> Self {
        255
    }

    /// The size of this integer type in bits.
    ///
    /// # Returns
    ///
    /// * [u64] - The number of bits for a `u8`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let bits = u8::bits();
    ///     assert(bits == 8);
    /// }
    /// ```
    pub fn bits() -> u64 {
        8
    }
}

impl b256 {
    /// The smallest value that can be represented by this type.
    ///
    /// # Returns
    ///
    /// * [b256] - The smallest `b256` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::constants::ZERO_B256;
    ///
    /// fn foo() {
    ///     let val = b256::min();
    ///     assert(val == ZERO_B256);
    /// }
    /// ```
    pub fn min() -> Self {
        0x0000000000000000000000000000000000000000000000000000000000000000
    }

    /// The largest value that can be represented by this type,
    /// 2<sup>256</sup> - 1.
    ///
    /// # Returns
    ///
    /// * [b256] - The largest `b256` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val == b256::max();
    ///     assert(val == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    /// }
    /// ```
    pub fn max() -> Self {
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
    }

    /// The size of this type in bits.
    ///
    /// # Returns
    ///
    /// * [u64] - The number of bits for a `b256`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let bits == b256::bits();
    ///     assert(bits == 256);
    /// }
    /// ```
    pub fn bits() -> u64 {
        256
    }
}
