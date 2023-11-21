library;

impl u64 {
    /// Extends a `u64` to a `u256`.
    ///
    /// # Returns
    ///
    /// * [u256] - The converted `u64` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = 2;
    ///     let result = val.as_u256();
    ///     assert(result == 0x0000000000000000000000000000000000000000000000000000000000000002u256);
    /// }
    /// ```
    pub fn as_u256(self) -> u256 {
        let input = (0u64, 0u64, 0u64, self);
        asm(input: input) {
            input: u256
        }
    }
}

impl u32 {
    /// Extends a `u32` to a `u64`.
    ///
    /// # Returns
    ///
    /// * [u64] - The converted `u32` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = 10u32;
    ///     let result = val.as_u64();
    ///     assert(result == 10);
    /// }
    /// ```
    pub fn as_u64(self) -> u64 {
        asm(input: self) {
            input: u64
        }
    }
}

// TODO: This must be in a separate impl block until https://github.com/FuelLabs/sway/issues/1548 is resolved
impl u32 {
    /// Extends a `u32` to a `u256`.
    ///
    /// # Returns
    ///
    /// * [u256] - The converted `u32` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = 2u32;
    ///     let result = val.as_u256();
    ///     assert(result == 0x0000000000000000000000000000000000000000000000000000000000000002u256);
    /// }
    /// ```
    pub fn as_u256(self) -> u256 {
        let input = (0u64, 0u64, 0u64, self.as_u64());
        asm(input: input) {
            input: u256
        }
    }
}

impl u16 {
    /// Extends a `u16` to a `u32`.
    ///
    /// # Returns
    ///
    /// * [u32] - The converted `u16` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = 10u16;
    ///     let result = val.as_u32();
    ///     assert(result == 10u32);
    /// }
    /// ```
    pub fn as_u32(self) -> u32 {
        asm(input: self) {
            input: u32
        }
    }

    /// Extends a `u16` to a `u64`.
    ///
    /// # Returns
    ///
    /// * [u64] - The converted `u16` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = 10u16;
    ///     let result = val.as_u64();
    ///     assert(result == 10);
    /// }
    /// ```
    pub fn as_u64(self) -> u64 {
        asm(input: self) {
            input: u64
        }
    }
}

// TODO: This must be in a separate impl block until https://github.com/FuelLabs/sway/issues/1548 is resolved
impl u16 {
    /// Extends a `u16` to a `u256`.
    ///
    /// # Returns
    ///
    /// * [u256] - The converted `u16` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = 2u16;
    ///     let result = val.as_u256();
    ///     assert(result == 0x0000000000000000000000000000000000000000000000000000000000000002u256);
    /// }
    /// ```
    pub fn as_u256(self) -> u256 {
        let input = (0u64, 0u64, 0u64, self.as_u64());
        asm(input: input) {
            input: u256
        }
    }
}

impl u8 {
    /// Extends a `u8` to a `u16`.
    ///
    /// # Returns
    ///
    /// * [u16] - The converted `u8` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = 2u8;
    ///     let result = val.as_u16();
    ///     assert(result == 2u16);
    /// }
    /// ```
    pub fn as_u16(self) -> u16 {
        asm(input: self) {
            input: u16
        }
    }

    /// Extends a `u8` to a `u32`.
    ///
    /// # Returns
    ///
    /// * [u32] - The converted `u8` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = 2u8;
    ///     let result = val.as_u32();
    ///     assert(result == 2u32);
    /// }
    /// ```
    pub fn as_u32(self) -> u32 {
        asm(input: self) {
            input: u32
        }
    }

    /// Extends a `u8` to a `u64`.
    ///
    /// # Returns
    ///
    /// * [u64] - The converted `u8` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = 2u8;
    ///     let result = val.as_u64();
    ///     assert(result == 2);
    /// }
    /// ```
    pub fn as_u64(self) -> u64 {
        asm(input: self) {
            input: u64
        }
    }
}

// TODO: This must be in a separate impl block until https://github.com/FuelLabs/sway/issues/1548 is resolved
impl u8 {
    /// Extends a `u8` to a `u256`.
    ///
    /// # Returns
    ///
    /// * [u256] - The converted `u8` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val = 2u8;
    ///     let result = val.as_u256();
    ///     assert(result == 0x0000000000000000000000000000000000000000000000000000000000000002u256);
    /// }
    /// ```
    pub fn as_u256(self) -> u256 {
        let input = (0u64, 0u64, 0u64, self.as_u64());
        asm(input: input) {
            input: u256
        }
    }
}

impl b256 {
    /// Converts a `b256` to a `u256`.
    ///
    /// # Returns
    ///
    /// * [u256] - The converted `b256` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val: b256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20;
    ///     let result = val.as_u256();
    ///     assert(result == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20u256);
    /// }
    /// ```
    pub fn as_u256(self) -> u256 {
        asm(input: self) {
            input: u256
        }
    }
}

impl u256 {
    /// Converts a `u256` to a `b256`.
    ///
    /// # Returns
    ///
    /// * [b256] - The converted `u256` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let val: u256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20u256;
    ///     let result = val.as_b256();
    ///     assert(result == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20);
    /// }
    /// ```
    pub fn as_b256(self) -> b256 {
        asm(input: self) {
            input: b256
        }
    }
}

fn assert(condition: bool) {
    if !condition {
        __revert(0)
    }
}

#[test]
fn test_u64_as_u256() {
    let val = 2;
    let result = val.as_u256();
    assert(result == 0x0000000000000000000000000000000000000000000000000000000000000002u256);
}

#[test]
fn test_u32_as_u64() {
    let val = 2u32;
    let result = val.as_u64();
    assert(result == 2);
}

#[test]
fn test_u32_as_u256() {
    let val = 2u32;
    let result = val.as_u256();
    assert(result == 0x0000000000000000000000000000000000000000000000000000000000000002u256);
}

#[test]
fn test_u16_as_u64() {
    let val = 2u16;
    let result = val.as_u64();
    assert(result == 2);
}

#[test]
fn test_u16_as_u32() {
    let val = 2u16;
    let result = val.as_u32();
    assert(result == 2u32);
}

#[test]
fn test_u16_as_u256() {
    let val = 2u16;
    let result = val.as_u256();
    assert(result == 0x0000000000000000000000000000000000000000000000000000000000000002u256);
}

#[test]
fn test_u8_as_u64() {
    let val = 2u8;
    let result = val.as_u64();
    assert(result == 2);
}

#[test]
fn test_u8_as_u32() {
    let val = 2u8;
    let result = val.as_u32();
    assert(result == 2u32);
}

#[test]
fn test_u8_as_u16() {
    let val = 2u8;
    let result = val.as_u16();
    assert(result == 2u16);
}

#[test]
fn test_u8_as_u256() {
    let val = 2u8;
    let result = val.as_u256();
    assert(result == 0x0000000000000000000000000000000000000000000000000000000000000002u256);
}

#[test]
fn test_b256_as_u256() {
    let val = 0x0000000000000000000000000000000000000000000000000000000000000002;
    let result = val.as_u256();
    assert(result == 0x0000000000000000000000000000000000000000000000000000000000000002u256);
}
