library;

use ::assert::assert;

impl u16 {
    /// Converts the `u16` to a sequence of little-endian bytes.
    /// 
    /// # Returns
    ///
    /// * [[u8; 2]] - An array of 2 `u8` bytes that compose the `u16`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let x: u16 = 513;
    ///     let result = x.to_le_bytes();
    /// 
    ///     assert(result[0] == 1_u8);
    ///     assert(result[1] == 2_u8);
    /// }
    /// ```
    pub fn to_le_bytes(self) -> [u8; 2] {
        let output = [0_u8, 0_u8];

        asm(input: self, off: 0xFF, i: 0x8, output: output, r1) {
            and  r1 input off;
            sb   output r1 i0;

            srl  r1 input i;
            and  r1 r1 off;
            sb   output r1 i1;

            output: [u8; 2]
        }
    }

    /// Converts a sequence of little-endian bytes to a `u16`.
    ///
    /// # Arguments
    /// 
    /// * `bytes`: [[u8; 2]] - A sequence of 2 `u8` bytes that represent a `u16`.
    ///
    /// # Returns
    /// 
    /// * [u16] - The resulting `u16` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let bytes = [1_u8, 2_u8];
    ///     let result = u16::from_le_bytes(bytes);
    ///
    ///     assert(result == 513_u16);
    /// }
    /// ```
    pub fn from_le_bytes(bytes: [u8; 2]) -> Self {
        asm(a: bytes[0], b: bytes[1], i: 0x8, r1) {
            sll  r1 b i;
            or   r1 a r1;
            r1: u16
        }
    }

    /// Converts the `u16` to a sequence of big-endian bytes.
    /// 
    /// # Returns
    ///
    /// * [[u8; 2]] - An array of 2 `u8` bytes that compose the `u16`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let x: u16 = 513;
    ///     let result = x.to_be_bytes();
    ///
    ///     assert(result[0] == 2_u8);
    ///     assert(result[1] == 1_u8);
    /// }
    /// ```
    pub fn to_be_bytes(self) -> [u8; 2] {
        let output = [0_u8, 0_u8];

        asm(input: self, off: 0xFF, i: 0x8, output: output, r1) {
            srl r1 input i;
            sb output r1 i0;

            and r1 input off;
            sb output r1 i1;

            output: [u8; 2]
        }
    }

    /// Converts a sequence of big-endian bytes to a `u16`.
    ///
    /// # Arguments
    /// 
    /// * `bytes`: [[u8; 2]] - A sequence of 2 `u8` bytes that represent a `u16`.
    ///
    /// # Returns
    /// 
    /// * [u16] - The resulting `u16` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let bytes = [2_u8, 1_u8];
    ///     let result = u16::from_be_bytes(bytes);
    /// 
    ///     assert(result == 513_u16);
    /// }
    /// ```
    pub fn from_be_bytes(bytes: [u8; 2]) -> Self {
        asm(a: bytes[0], b: bytes[1], i: 0x8, r1) {
            sll  r1 a i;
            or   r1 r1 b;
            r1: u16
        }
    }
}

#[test]
fn test_u16_to_le_bytes() {
    let x: u16 = 513;
    let result = x.to_le_bytes();

    assert(result[0] == 1_u8);
    assert(result[1] == 2_u8);
}

#[test]
fn test_u16_from_le_bytes() {
    let bytes = [1_u8, 2_u8];
    let result = u16::from_le_bytes(bytes);

    assert(result == 513_u16);
}

#[test]
fn test_u16_to_be_bytes() {
    let x: u16 = 513;
    let result = x.to_be_bytes();

    assert(result[0] == 2_u8);
    assert(result[1] == 1_u8);
}

#[test]
fn test_u16_from_be_bytes() {
    let bytes = [2_u8, 1_u8];
    let result = u16::from_be_bytes(bytes);

    assert(result == 513_u16);
}
