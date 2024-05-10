//! The zero trait, for defining the zero value of a type.
library;

/// The `Zero` trait.
pub trait Zero {
    /// Returns the zero value of `Self`.
    ///
    /// # Returns
    ///
    /// * [Self] -> The zero value for the `Self` type.
    fn zero() -> Self;

    /// Returns whether a type is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if `Self` is zero, otherwise false.
    fn is_zero(self) -> bool;
}

impl Zero for u8 {
    /// Returns the zero value for the `u8` type.
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
    fn zero() -> Self {
        0u8
    }

    /// Returns whether a `u8` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `u8` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_u8 = u8::zero();
    ///     assert(zero_u8.is_zero());
    /// }
    /// ```
    fn is_zero(self) -> bool {
        self == 0u8
    }
}

impl Zero for u16 {
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
    fn zero() -> Self {
        0u16
    }

    /// Returns whether a `u16` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `u16` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_u16 = u16::zero();
    ///     assert(zero_u16.is_zero());
    /// }
    /// ```
    fn is_zero(self) -> bool {
        self == 0u16
    }
}

impl Zero for u32 {
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
    fn zero() -> Self {
        0u32
    }

    /// Returns whether a `u32` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `u32` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_u32 = u32::zero();
    ///     assert(zero_u32.is_zero());
    /// }
    /// ```
    fn is_zero(self) -> bool {
        self == 0u32
    }
}

impl Zero for u64 {
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
    fn zero() -> Self {
        0u64
    }

    /// Returns whether a `u64` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `u64` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_u64 = u64::zero();
    ///     assert(zero_u64.is_zero());
    /// }
    /// ```
    fn is_zero(self) -> bool {
        self == 0u64
    }
}

impl Zero for u256 {
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
    fn zero() -> Self {
        0x00u256
    }

    /// Returns whether a `u256` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `u256` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_u256 = u256::zero();
    ///     assert(zero_u256.is_zero());
    /// }
    /// ```
    fn is_zero(self) -> bool {
        self == 0x00u256
    }
}

impl Zero for b256 {
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
    fn zero() -> Self {
        0x0000000000000000000000000000000000000000000000000000000000000000
    }

    /// Returns whether a `b256` is set to zero.
    ///
    /// # Returns
    ///
    /// * [bool] -> True if the `b256` is zero, otherwise false.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let zero_b256 = b256::zero();
    ///     assert(zero_b256.is_zero());
    /// }
    /// ```
    fn is_zero(self) -> bool {
        self == 0x0000000000000000000000000000000000000000000000000000000000000000
    }
}
