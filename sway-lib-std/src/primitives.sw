library;

impl u256 {
    /// The smallest value that can be represented by this integer type.
    ///
    /// # Returns
    ///
    /// * [u256] - The smallest `u256` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = u256::min();
    ///     assert(val == 0x0000000000000000000000000000000000000000000000000000000000000000u256);
    // }
    /// ```
    pub fn min() -> Self {
        0x0000000000000000000000000000000000000000000000000000000000000000u256
    }

    /// The largest value that can be represented by this integer type,
    /// 2<sup>256</sup> - 1.
    ///
    /// # Returns
    ///
    /// * [u256] - The largest `u256` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = u256::max();
    ///     assert(val == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu256);
    /// }
    /// ```
    pub fn max() -> Self {
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu256
    }

    /// The size of this integer type in bits.
    ///
    /// # Returns
    ///
    /// * [u32] - The number of bits for a `u256`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let bits = u256::bits();
    ///     assert(bits == 256);
    /// }
    /// ```
    pub fn bits() -> u64 {
        256
    }

    /// Returns the zero value for the `u256` type.
    ///
    /// # Returns
    ///
    /// * [u256] -> The zero value for the `u256` type.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_u256 = u256::zero();
    ///     assert(zero_u256 == 0x00u256);
    /// }
    /// ```
    pub fn zero() -> Self {
        0x00u256
    }
}

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

    /// Returns the zero value for the `u64` type.
    ///
    /// # Returns
    ///
    /// * [u64] -> The zero value for the `u64` type.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_u64 = u64::zero();
    ///     assert(zero_u64 == 0u64);
    /// }
    /// ```
    pub fn zero() -> Self {
        0u64
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

    /// Returns the zero value for the `u32` type.
    ///
    /// # Returns
    ///
    /// * [u32] -> The zero value for the `u32` type.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_u32 = u32::zero();
    ///     assert(zero_u32 == 0u32);
    /// }
    /// ```
    pub fn zero() -> Self {
        0u32
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

    /// Returns the zero value for the `u16` type.
    ///
    /// # Returns
    ///
    /// * [u16] -> The zero value for the `u16` type.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_u16 = u16::zero();
    ///     assert(zero_u16 == 0u16);
    /// }
    /// ```
    pub fn zero() -> Self {
        0u16
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

    // Returns the zero value for the `u8` type.
    ///
    /// # Returns
    ///
    /// * [u8] -> The zero value for the `u8` type.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_u8 = u8::zero();
    ///     assert(zero_u8 == 0u8);
    /// }
    /// ```
    pub fn zero() -> Self {
        0u8
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
    /// fn foo() {
    ///     let val = b256::min();
    ///     assert(val == b256::zero());
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

    /// Returns the zero value for the `b256` type.
    ///
    /// # Returns
    ///
    /// * [b256] -> The zero value for the `b256` type.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_b256 = b256::zero();
    ///     assert(zero_b256 == 0x0000000000000000000000000000000000000000000000000000000000000000);
    /// }
    /// ```
    pub fn zero() -> Self {
        0x0000000000000000000000000000000000000000000000000000000000000000
    }
}

#[cfg(experimental_const_generics = true)]
impl<T, const N: u64> [T; N] {
    pub fn len(self) -> u64 {
        N
    }
}

#[cfg(experimental_const_generics = true)]
impl<const N: u64> str[N] {
    pub fn len(self) -> u64 {
        N
    }
}
