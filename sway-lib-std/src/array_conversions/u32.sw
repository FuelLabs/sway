library;

use ::assert::assert;

impl u32 {
    /// Converts the `u32` to a sequence of little-endian bytes.
    /// 
    /// # Returns
    ///
    /// * [[u8; 4]] - An array of 4 `u8` bytes that compose the `u32`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let x: u32 = 67305985;
    ///     let result = x.to_le_bytes();
    /// 
    ///     assert(result[0] == 1_u8);
    ///     assert(result[1] == 2_u8);
    ///     assert(result[2] == 3_u8);
    ///     assert(result[3] == 4_u8);
    /// }
    /// ```
    pub fn to_le_bytes(self) -> [u8; 4] {
        let output = [0_u8, 0_u8, 0_u8, 0_u8];

        asm(input: self, off: 0xFF, i: 0x8, j: 0x10, k: 0x18, output: output, r1) {
            and  r1 input off;
            sb   output r1 i0;

            srl  r1 input i;
            and  r1 r1 off;
            sb   output r1 i1;

            srl  r1 input j;
            and  r1 r1 off;
            sb   output r1 i2;

            srl  r1 input k;
            and  r1 r1 off;
            sb   output r1 i3;

            output: [u8; 4]
        }
    }

    /// Converts a sequence of little-endian bytes to a `u32`.
    ///
    /// # Arguments
    /// 
    /// * `bytes`: [[u8; 4]] - A sequence of 4 `u8` bytes that represent a `u32`.
    ///
    /// # Returns
    /// 
    /// * [u32] - The resulting `u32` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let bytes = [1_u8, 2_u8, 3_u8, 4_u8];
    ///     let result = u32::from_le_bytes(bytes);
    ///
    ///     assert(result == 67305985_u32);
    /// }
    /// ```
    pub fn from_le_bytes(bytes: [u8; 4]) -> Self {
        asm(a: bytes[0], b: bytes[1], c: bytes[2], d: bytes[3], i: 0x8, j: 0x10, k: 0x18, r1, r2, r3) {
            sll  r1 c j;
            sll  r2 d k;
            or   r3 r1 r2;
            sll  r1 b i;
            or   r2 a r1;
            or   r1 r2 r3;
            r1: u32
        }
    }

    /// Converts the `u32` to a sequence of big-endian bytes.
    /// 
    /// # Returns
    ///
    /// * [[u8; 4]] - An array of 4 `u8` bytes that compose the `u32`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let x: u32 = 67305985;
    ///     let result = x.to_be_bytes();
    ///
    ///     assert(result[0] == 4_u8);
    ///     assert(result[1] == 3_u8);
    ///     assert(result[2] == 2_u8);
    ///     assert(result[3] == 1_u8);
    /// }
    /// ```
    pub fn to_be_bytes(self) -> [u8; 4] {
        let output = [0_u8, 0_u8, 0_u8, 0_u8];

        asm(input: self, off: 0xFF, i: 0x8, j: 0x10, k: 0x18, output: output, r1) {
            srl  r1 input k;
            and  r1 r1 off;
            sb   output r1 i0;

            srl  r1 input j;
            and  r1 r1 off;
            sb   output r1 i1;

            srl  r1 input i;
            and  r1 r1 off;
            sb   output r1 i2;

            and  r1 input off;
            sb   output r1 i3;

            output: [u8; 4]
        }
    }

    /// Converts a sequence of big-endian bytes to a `u32`.
    ///
    /// # Arguments
    /// 
    /// * `bytes`: [[u8; 4]] - A sequence of 4 `u8` bytes that represent a `u32`.
    ///
    /// # Returns
    /// 
    /// * [u32] - The resulting `u32` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let bytes = [4_u8, 3_u8, 2_u8, 1_u8];
    ///     let result = u32::from_be_bytes(bytes);
    ///
    ///     assert(result == 67305985_u32);
    /// }
    /// ```
    pub fn from_be_bytes(bytes: [u8; 4]) -> Self {
        asm(a: bytes[0], b: bytes[1], c: bytes[2], d: bytes[3], i: 0x8, j: 0x10, k: 0x18, r1, r2, r3) {
            sll  r1 a k;
            sll  r2 b j;
            or   r3 r1 r2;
            sll  r1 c i;
            or   r2 r3 r1;
            or   r1 r2 d;
            r1: u32
        }
    }
}

#[test]
fn test_u32_to_le_bytes() {
    let x: u32 = 67305985;
    let result = x.to_le_bytes();

    assert(result[0] == 1_u8);
    assert(result[1] == 2_u8);
    assert(result[2] == 3_u8);
    assert(result[3] == 4_u8);
}

#[test]
fn test_u32_from_le_bytes() {
    let bytes = [1_u8, 2_u8, 3_u8, 4_u8];
    let result = u32::from_le_bytes(bytes);

    assert(result == 67305985_u32);
}

#[test]
fn test_u32_to_be_bytes() {
    let x: u32 = 67305985;
    let result = x.to_be_bytes();

    assert(result[0] == 4_u8);
    assert(result[1] == 3_u8);
    assert(result[2] == 2_u8);
    assert(result[3] == 1_u8);
}

#[test]
fn test_u32_from_be_bytes() {
    let bytes = [4_u8, 3_u8, 2_u8, 1_u8];
    let result = u32::from_be_bytes(bytes);

    assert(result == 67305985_u32);
}
