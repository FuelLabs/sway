library;

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
            sw   output r1 i0;

            srl  r1 input i;
            and  r1 r1 off;
            sw   output r1 i1;

            srl  r1 input j;
            and  r1 r1 off;
            sw   output r1 i2;

            srl  r1 input k;
            and  r1 r1 off;
            sw   output r1 i3;

            srl  r1 input l;
            and  r1 r1 off;
            sw   output r1 i4;

            srl  r1 input m;
            and  r1 r1 off;
            sw   output r1 i5;

            srl  r1 input n;
            and  r1 r1 off;
            sw   output r1 i6;

            srl  r1 input o;
            and  r1 r1 off;
            sw   output r1 i7;

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
            sw   output r1 i7;

            srl  r1 input i;
            and  r1 r1 off;
            sw   output r1 i6;

            srl  r1 input j;
            and  r1 r1 off;
            sw   output r1 i5;

            srl  r1 input k;
            and  r1 r1 off;
            sw   output r1 i4;

            srl  r1 input l;
            and  r1 r1 off;
            sw   output r1 i3;

            srl  r1 input m;
            and  r1 r1 off;
            sw   output r1 i2;

            srl  r1 input n;
            and  r1 r1 off;
            sw   output r1 i1;

            srl  r1 input o;
            and  r1 r1 off;
            sw   output r1 i0;

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
        asm(input: self) { input: u64 }
    }

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
            sw   output r1 i0;

            srl  r1 input i;
            and  r1 r1 off;
            sw   output r1 i1;

            srl  r1 input j;
            and  r1 r1 off;
            sw   output r1 i2;

            srl  r1 input k;
            and  r1 r1 off;
            sw   output r1 i3;

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
            sw   output r1 i0;

            srl  r1 input j;
            and  r1 r1 off;
            sw   output r1 i1;

            srl  r1 input i;
            and  r1 r1 off;
            sw   output r1 i2;

            and  r1 input off;
            sw   output r1 i3;

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
        asm(input: self) { input: u32 }
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
        asm(input: self) { input: u64 }
    }

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
            sw   output r1 i0;

            srl  r1 input i;
            and  r1 r1 off;
            sw   output r1 i1;

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
            srl  r1 input i;
            sw   output r1 i0;

            and  r1 input off;
            sw   output r1 i1;

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
    /// * [u32] - The resulting `u16` value.
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
        asm(input: self) { input: u16 }
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
        asm(input: self) { input: u32 }
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
        asm(input: self) { input: u64 }
    }
}

impl b256 {
    /// Converts the `b256` to a sequence of little-endian bytes.
    ///
    /// # Returns
    ///
    /// * [[u8; 32]] - An array of 32 `u8` bytes that compose the `b256`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let bytes = [32_u8, 31_u8, 30_u8, 29_u8, 28_u8, 27_u8, 26_u8, 25_u8, 24_u8, 23_u8,
    ///             22_u8, 21_u8, 20_u8, 19_u8, 18_u8, 17_u8, 16_u8, 15_u8, 14_u8, 13_u8,
    ///             12_u8, 11_u8, 10_u8, 9_u8, 8_u8, 7_u8, 6_u8, 5_u8, 4_u8, 3_u8,
    ///             2_u8, 1_u8];
    ///
    ///     let x = b256::from_le_bytes(bytes);
    ///
    ///     assert(x == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20);
    /// }
    /// ```
    pub fn to_le_bytes(self) -> [u8; 32] {
        let (a, b, c, d): (u64, u64, u64, u64) = asm(r1: self) { r1: (u64, u64, u64, u64) };
        let a = a.to_le_bytes();
        let b = b.to_le_bytes();
        let c = c.to_le_bytes();
        let d = d.to_le_bytes();

        let (a, b, c, d) = (d, c, b, a);

        let output = [
            a[0],
            a[1],
            a[2],
            a[3],
            a[4],
            a[5],
            a[6],
            a[7],
            b[0],
            b[1],
            b[2],
            b[3],
            b[4],
            b[5],
            b[6],
            b[7],
            c[0],
            c[1],
            c[2],
            c[3],
            c[4],
            c[5],
            c[6],
            c[7],
            d[0],
            d[1],
            d[2],
            d[3],
            d[4],
            d[5],
            d[6],
            d[7],
        ];

        output
    }

    /// Converts a sequence of little-endian bytes to a `b256`.
    ///
    /// # Arguments
    ///
    /// * `bytes`: [[u8; 32]] - A sequence of 32 `u8` bytes that represent a `b256`.
    ///
    /// # Returns
    ///
    /// * [b256] - The resulting `b256` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let bytes = [32_u8, 31_u8, 30_u8, 29_u8, 28_u8, 27_u8, 26_u8, 25_u8, 24_u8, 23_u8,
    ///             22_u8, 21_u8, 20_u8, 19_u8, 18_u8, 17_u8, 16_u8, 15_u8, 14_u8, 13_u8,
    ///             12_u8, 11_u8, 10_u8, 9_u8, 8_u8, 7_u8, 6_u8, 5_u8, 4_u8, 3_u8,
    ///             2_u8, 1_u8];
    ///
    ///     let x = b256::from_le_bytes(bytes);
    ///
    ///     assert(x == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20);
    /// ```
    pub fn from_le_bytes(bytes: [u8; 32]) -> Self {
        let a = u64::from_le_bytes([
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3],
            bytes[4],
            bytes[5],
            bytes[6],
            bytes[7],
        ]);
        let b = u64::from_le_bytes([
            bytes[8],
            bytes[9],
            bytes[10],
            bytes[11],
            bytes[12],
            bytes[13],
            bytes[14],
            bytes[15],
        ]);
        let c = u64::from_le_bytes([
            bytes[16],
            bytes[17],
            bytes[18],
            bytes[19],
            bytes[20],
            bytes[21],
            bytes[22],
            bytes[23],
        ]);
        let d = u64::from_le_bytes([
            bytes[24],
            bytes[25],
            bytes[26],
            bytes[27],
            bytes[28],
            bytes[29],
            bytes[30],
            bytes[31],
        ]);

        let result = (d, c, b, a);

        asm(r1: result) { r1: b256 }
    }

    /// Converts the `b256` to a sequence of big-endian bytes.
    ///
    /// # Returns
    ///
    /// * [[u8; 32]] - An array of 32 `u8` bytes that compose the `b256`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let x: b256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20;
    ///     let bytes = x.to_be_bytes();
    ///
    ///     let mut i: u8 = 0;
    ///     while i < 32_u8 {
    ///         assert(bytes[i.as_u64()] == i + 1_u8);
    ///         i += 1_u8;
    ///     }
    /// }
    /// ```
    pub fn to_be_bytes(self) -> [u8; 32] {
        let (a, b, c, d): (u64, u64, u64, u64) = asm(r1: self) { r1: (u64, u64, u64, u64) };
        let a = a.to_be_bytes();
        let b = b.to_be_bytes();
        let c = c.to_be_bytes();
        let d = d.to_be_bytes();

        let output = [
            a[0],
            a[1],
            a[2],
            a[3],
            a[4],
            a[5],
            a[6],
            a[7],
            b[0],
            b[1],
            b[2],
            b[3],
            b[4],
            b[5],
            b[6],
            b[7],
            c[0],
            c[1],
            c[2],
            c[3],
            c[4],
            c[5],
            c[6],
            c[7],
            d[0],
            d[1],
            d[2],
            d[3],
            d[4],
            d[5],
            d[6],
            d[7],
        ];

        output
    }

    /// Converts a sequence of big-endian bytes to a `b256`.
    ///
    /// # Arguments
    ///
    /// * `bytes`: [[u8; 32]] - A sequence of 32 `u8` bytes that represent a `b256`.
    ///
    /// # Returns
    ///
    /// * [b256] - The resulting `b256` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let bytes = [1_u8, 2_u8, 3_u8, 4_u8, 5_u8, 6_u8, 7_u8, 8_u8, 9_u8, 10_u8,
    ///             11_u8, 12_u8, 13_u8, 14_u8, 15_u8, 16_u8, 17_u8, 18_u8, 19_u8, 20_u8,
    ///             21_u8, 22_u8, 23_u8, 24_u8, 25_u8, 26_u8, 27_u8, 28_u8, 29_u8, 30_u8,
    ///             31_u8, 32_u8];
    ///     let x = b256::from_be_bytes(bytes);
    ///
    ///     assert(x == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20);
    /// }
    /// ```
    pub fn from_be_bytes(bytes: [u8; 32]) -> Self {
        let a = u64::from_be_bytes([
            bytes[0],
            bytes[1],
            bytes[2],
            bytes[3],
            bytes[4],
            bytes[5],
            bytes[6],
            bytes[7],
        ]);
        let b = u64::from_be_bytes([
            bytes[8],
            bytes[9],
            bytes[10],
            bytes[11],
            bytes[12],
            bytes[13],
            bytes[14],
            bytes[15],
        ]);
        let c = u64::from_be_bytes([
            bytes[16],
            bytes[17],
            bytes[18],
            bytes[19],
            bytes[20],
            bytes[21],
            bytes[22],
            bytes[23],
        ]);
        let d = u64::from_be_bytes([
            bytes[24],
            bytes[25],
            bytes[26],
            bytes[27],
            bytes[28],
            bytes[29],
            bytes[30],
            bytes[31],
        ]);

        let result = (a, b, c, d);

        asm(r1: result) { r1: b256 }
    }
}

fn assert(condition: bool) {
    if !condition {
        __revert(0)
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

#[test]
fn test_b256_from_le_bytes() {
    let bytes = [
        32_u8,
        31_u8,
        30_u8,
        29_u8,
        28_u8,
        27_u8,
        26_u8,
        25_u8,
        24_u8,
        23_u8,
        22_u8,
        21_u8,
        20_u8,
        19_u8,
        18_u8,
        17_u8,
        16_u8,
        15_u8,
        14_u8,
        13_u8,
        12_u8,
        11_u8,
        10_u8,
        9_u8,
        8_u8,
        7_u8,
        6_u8,
        5_u8,
        4_u8,
        3_u8,
        2_u8,
        1_u8,
    ];

    let x = b256::from_le_bytes(bytes);

    assert(x == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20);
}

#[test]
fn test_b256_to_le_bytes() {
    let x: b256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20;

    let bytes = x.to_le_bytes();

    let mut i: u8 = 0;
    while i < 32_u8 {
        assert(bytes[i.as_u64()] == 32_u8 - i);
        i += 1_u8;
    }
}

#[test]
fn test_b256_from_be_bytes() {
    let bytes = [
        1_u8,
        2_u8,
        3_u8,
        4_u8,
        5_u8,
        6_u8,
        7_u8,
        8_u8,
        9_u8,
        10_u8,
        11_u8,
        12_u8,
        13_u8,
        14_u8,
        15_u8,
        16_u8,
        17_u8,
        18_u8,
        19_u8,
        20_u8,
        21_u8,
        22_u8,
        23_u8,
        24_u8,
        25_u8,
        26_u8,
        27_u8,
        28_u8,
        29_u8,
        30_u8,
        31_u8,
        32_u8,
    ];

    let x = b256::from_be_bytes(bytes);

    assert(x == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20);
}

#[test]
fn test_b256_to_be_bytes() {
    let x: b256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20;

    let bytes = x.to_be_bytes();

    let mut i: u8 = 0;
    while i < 32_u8 {
        assert(bytes[i.as_u64()] == i + 1_u8);
        i += 1_u8;
    }
}
