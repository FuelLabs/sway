library;

use ::assert::assert;

impl u64 {
    /// Converts the `u64` to a sequence of little-endian bytes.
    /// 
    /// # Returns
    ///
    /// * [[u8; 8]] - An array of 8 `u8` bytes that compose the `u64`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let x: u64 = 578437695752307201;
    ///     let result = x.to_le_bytes();
    ///
    ///     assert(result[0] == 1_u8);
    ///     assert(result[1] == 2_u8);
    ///     assert(result[2] == 3_u8);
    ///     assert(result[3] == 4_u8);
    ///     assert(result[4] == 5_u8);
    ///     assert(result[5] == 6_u8);
    ///     assert(result[6] == 7_u8);
    ///     assert(result[7] == 8_u8);
    /// }
    /// ```
    pub fn to_le_bytes(self) -> [u8; 8] {
        let output = [0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8];

        asm(input: self, off: 0xFF, i: 0x8, j: 0x10, k: 0x18, l: 0x20, m: 0x28, n: 0x30, o: 0x38, output: output, r1) {
            and  r1 input off;
            sb  output r1 i0;

            srl  r1 input i;
            and  r1 r1 off;
            sb  output r1 i1;

            srl  r1 input j;
            and  r1 r1 off;
            sb  output r1 i2;

            srl  r1 input k;
            and  r1 r1 off;
            sb  output r1 i3;

            srl  r1 input l;
            and  r1 r1 off;
            sb  output r1 i4;

            srl  r1 input m;
            and  r1 r1 off;
            sb  output r1 i5;

            srl  r1 input n;
            and  r1 r1 off;
            sb  output r1 i6;

            srl  r1 input o;
            and  r1 r1 off;
            sb  output r1 i7;

            output: [u8; 8]
        }
    }

    /// Converts a sequence of little-endian bytes to a `u64`.
    ///
    /// # Arguments
    /// 
    /// * `bytes`: [[u8; 8]] - A sequence of 8 `u8` bytes that represent a `u64`.
    ///
    /// # Returns
    /// 
    /// * [u64] - The resulting `u64` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let bytes = [1_u8, 2_u8, 3_u8, 4_u8, 5_u8, 6_u8, 7_u8, 8_u8];
    ///     let result = u64::from_le_bytes(bytes);
    ///
    ///     assert(result == 578437695752307201);
    /// }
    /// ```
    pub fn from_le_bytes(bytes: [u8; 8]) -> Self {
        let a = bytes[0];
        let b = bytes[1];
        let c = bytes[2];
        let d = bytes[3];
        let e = bytes[4];
        let f = bytes[5];
        let g = bytes[6];
        let h = bytes[7];

        asm(a: a, b: b, c: c, d: d, e: e, f: f, g: g, h: h, i: 0x8, j: 0x10, k: 0x18, l: 0x20, m: 0x28, n: 0x30, o: 0x38, r1, r2, r3) {
            sll  r1 h o;
            sll  r2 g n;
            or   r3 r1 r2;
            sll  r1 f m;
            or   r2 r3 r1;
            sll  r3 e l;
            or   r1 r2 r3;
            sll  r2 d k;
            or   r3 r1 r2;
            sll  r1 c j;
            or   r2 r3 r1;
            sll  r3 b i;
            or   r1 r2 r3;
            or   r2 r1 a;

            r2: u64    
        }
    }

    /// Converts the `u64` to a sequence of big-endian bytes.
    /// 
    /// # Returns
    ///
    /// * [[u8; 8]] - An array of 8 `u8` bytes that compose the `u64`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let x: u64 = 578437695752307201;
    ///     let result = x.to_be_bytes();
    ///
    ///     assert(result[0] == 8_u8);
    ///     assert(result[1] == 7_u8);
    ///     assert(result[2] == 6_u8);
    ///     assert(result[3] == 5_u8);
    ///     assert(result[4] == 4_u8);
    ///     assert(result[5] == 3_u8);
    ///     assert(result[6] == 2_u8);
    ///     assert(result[7] == 1_u8);
    /// }
    /// ```
    pub fn to_be_bytes(self) -> [u8; 8] {
        let output = [0; 8];

        asm(input: self, off: 0xFF, i: 0x8, j: 0x10, k: 0x18, l: 0x20, m: 0x28, n: 0x30, o: 0x38, output: output, r1) {
            and  r1 input off;
            sb  output r1 i7;

            srl  r1 input i;
            and  r1 r1 off;
            sb  output r1 i6;

            srl  r1 input j;
            and  r1 r1 off;
            sb  output r1 i5;

            srl  r1 input k;
            and  r1 r1 off;
            sb  output r1 i4;

            srl  r1 input l;
            and  r1 r1 off;
            sb  output r1 i3;

            srl  r1 input m;
            and  r1 r1 off;
            sb  output r1 i2;

            srl  r1 input n;
            and  r1 r1 off;
            sb  output r1 i1;

            srl  r1 input o;
            and  r1 r1 off;
            sb  output r1 i0;

            output: [u8; 8]
        }
    }

    /// Converts a sequence of big-endian bytes to a `u64`.
    ///
    /// # Arguments
    /// 
    /// * `bytes`: [[u8; 8]] - A sequence of 8 `u8` bytes that represent a `u64`.
    ///
    /// # Returns
    /// 
    /// * [u64] - The resulting `u64` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let bytes = [8_u8, 7_u8, 6_u8, 5_u8, 4_u8, 3_u8, 2_u8, 1_u8];
    ///     let result = u64::from_be_bytes(bytes);
    ///
    ///     assert(result == 578437695752307201);
    /// }
    /// ```
    pub fn from_be_bytes(bytes: [u8; 8]) -> Self {
        let a = bytes[0];
        let b = bytes[1];
        let c = bytes[2];
        let d = bytes[3];
        let e = bytes[4];
        let f = bytes[5];
        let g = bytes[6];
        let h = bytes[7];

        asm(a: a, b: b, c: c, d: d, e: e, f: f, g: g, h: h, i: 0x8, j: 0x10, k: 0x18, l: 0x20, m: 0x28, n: 0x30, o: 0x38, r1, r2, r3) {
            sll  r1 a o;
            sll  r2 b n;
            or   r3 r1 r2;
            sll  r1 c m;
            or   r2 r3 r1;
            sll  r3 d l;
            or   r1 r2 r3;
            sll  r2 e k;
            or   r3 r1 r2;
            sll  r1 f j;
            or   r2 r3 r1;
            sll  r3 g i;
            or   r1 r2 r3;
            or   r2 r1 h;

            r2: u64
        }
    }
}

#[test]
fn test_u64_to_le_bytes() {
    let x: u64 = 578437695752307201;
    let result = x.to_le_bytes();

    assert(result[0] == 1_u8);
    assert(result[1] == 2_u8);
    assert(result[2] == 3_u8);
    assert(result[3] == 4_u8);
    assert(result[4] == 5_u8);
    assert(result[5] == 6_u8);
    assert(result[6] == 7_u8);
    assert(result[7] == 8_u8);
}

#[test]
fn test_u64_from_le_bytes() {
    let bytes = [1_u8, 2_u8, 3_u8, 4_u8, 5_u8, 6_u8, 7_u8, 8_u8];
    let result = u64::from_le_bytes(bytes);

    assert(result == 578437695752307201);
}

#[test]
fn test_u64_to_be_bytes() {
    let x: u64 = 578437695752307201;
    let result = x.to_be_bytes();

    assert(result[0] == 8_u8);
    assert(result[1] == 7_u8);
    assert(result[2] == 6_u8);
    assert(result[3] == 5_u8);
    assert(result[4] == 4_u8);
    assert(result[5] == 3_u8);
    assert(result[6] == 2_u8);
    assert(result[7] == 1_u8);
}

#[test]
fn test_u64_from_be_bytes() {
    let bytes = [8_u8, 7_u8, 6_u8, 5_u8, 4_u8, 3_u8, 2_u8, 1_u8];
    let result = u64::from_be_bytes(bytes);

    assert(result == 578437695752307201);
}
