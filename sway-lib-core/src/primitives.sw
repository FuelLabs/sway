library;

const OFF: u64 = 0xFF;
const I: u64 = 8;
const J: u64 = 16;
const K: u64 = 24;
const L: u64 = 32;
const M: u64 = 40;
const N: u64 = 48;
const O: u64 = 56;

impl u64 {
    /// The smallest value that can be represented by this integer type.
    pub fn min() -> u64 {
        0
    }

    /// The largest value that can be represented by this integer type,
    /// 2<sup>64</sup> - 1.
    pub fn max() -> u64 {
        18446744073709551615
    }

    /// The size of this integer type in bits.
    pub fn bits() -> u32 {
        64
    }

    pub fn to_le_bytes(self) -> [u8; 8] {
        let output = [0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8];

        asm(input: self, off: OFF, i: I, j: J, k: K, l: L, m: M, n: N, n: N, o: O, output: output, r1) {
            and  r1 input off;
            sw  output r1 i0;

            srl  r1 input i;
            and  r1 r1 off;
            sw  output r1 i1;

            srl  r1 input j;
            and  r1 r1 off;
            sw  output r1 i2;

            srl  r1 input k;
            and  r1 r1 off;
            sw  output r1 i3;

            srl  r1 input l;
            and  r1 r1 off;
            sw  output r1 i4;

            srl  r1 input m;
            and  r1 r1 off;
            sw  output r1 i5;

            srl  r1 input n;
            and  r1 r1 off;
            sw  output r1 i6;

            srl  r1 input o;
            and  r1 r1 off;
            sw  output r1 i7;

            output: [u8; 8]
        }
    }

    pub fn from_le_bytes(bytes: [u8; 8]) -> u64 {
        let a = bytes[0];
        let b = bytes[1];
        let c = bytes[2];
        let d = bytes[3];
        let e = bytes[4];
        let f = bytes[5];
        let g = bytes[6];
        let h = bytes[7];

        asm(a: a, b: b, c: c, d: d, e: e, f: f, g: g, h: h, i: I, j: J, k: K, l: L, m: M, n: N, n: N, o: O, r1, r2, r3) {
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

    pub fn to_be_bytes(self) -> [u8; 8] {
        let output = [0; 8];

        asm(input: self, off: OFF, i: I, j: J, k: K, l: L, m: M, n: N, n: N, o: O, output: output, r1) {
            and  r1 input off;
            sw  output r1 i7;

            srl  r1 input i;
            and  r1 r1 off;
            sw  output r1 i6;

            srl  r1 input j;
            and  r1 r1 off;
            sw  output r1 i5;

            srl  r1 input k;
            and  r1 r1 off;
            sw  output r1 i4;

            srl  r1 input l;
            and  r1 r1 off;
            sw  output r1 i3;

            srl  r1 input m;
            and  r1 r1 off;
            sw  output r1 i2;

            srl  r1 input n;
            and  r1 r1 off;
            sw  output r1 i1;

            srl  r1 input o;
            and  r1 r1 off;
            sw  output r1 i0;

            output: [u8; 8]
        }
    }

    pub fn from_be_bytes(bytes: [u8; 8]) -> u64 {
        let a = bytes[0];
        let b = bytes[1];
        let c = bytes[2];
        let d = bytes[3];
        let e = bytes[4];
        let f = bytes[5];
        let g = bytes[6];
        let h = bytes[7];

        asm(a: a, b: b, c: c, d: d, e: e, f: f, g: g, h: h, i: I, j: J, k: K, l: L, m: M, n: N, n: N, o: O, r1, r2, r3) {
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
    /// The smallest value that can be represented by this integer type.
    pub fn min() -> u32 {
        0
    }

    /// The largest value that can be represented by this integer type,
    /// 2<sup>32</sup> - 1.
    pub fn max() -> u32 {
        4294967295
    }

    /// The size of this integer type in bits.
    pub fn bits() -> u32 {
        32
    }

    pub fn to_le_bytes(self) -> [u8; 4] {
        let output = [0_u8, 0_u8, 0_u8, 0_u8];

        asm(input: self, off: OFF, i: I, j: J, k: K, output: output, r1) {
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

    pub fn from_le_bytes(bytes: [u8; 4]) -> u32 {
        asm(a: bytes[0], b: bytes[1], c: bytes[2], d: bytes[3], i: I, j: J, k: K, r1, r2, r3) {
            sll  r1 c j;
            sll  r2 d k;
            or   r3 r1 r2;
            sll  r1 b i;
            or   r2 a r1;
            or   r1 r2 r3;
            r1: u32
        }
    }

    pub fn to_be_bytes(self) -> [u8; 4] {
        let output = [0_u8, 0_u8, 0_u8, 0_u8];

        asm(input: self, off: OFF, i: I, j: J, k: K, output: output, r1) {
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

    pub fn from_be_bytes(bytes: [u8; 4]) -> u32 {
        asm(a: bytes[0], b: bytes[1], c: bytes[2], d: bytes[3], i: I, j: J, k: K, r1, r2, r3) {
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
    /// The smallest value that can be represented by this integer type.
    pub fn min() -> u16 {
        0
    }

    /// The largest value that can be represented by this integer type,
    /// 2<sup>16</sup> - 1.
    pub fn max() -> u16 {
        65535
    }

    /// The size of this integer type in bits.
    pub fn bits() -> u32 {
        16
    }

    pub fn to_le_bytes(self) -> [u8; 2] {
        let output = [0_u8, 0_u8];

        asm(input: self, off: OFF, i: I, output: output, r1) {
            and  r1 input off;
            sw   output r1 i0;

            srl  r1 input i;
            and  r1 r1 off;
            sw   output r1 i1;

            output: [u8; 2]
        }
    }

    pub fn from_le_bytes(bytes: [u8; 2]) -> u16 {
        asm(a: bytes[0], b: bytes[1], i: I, r1) {
            sll  r1 b i;
            or   r1 a r1;
            r1: u16
        }
    }

    pub fn to_be_bytes(self) -> [u8; 2] {
        let output = [0_u8, 0_u8];

        asm(input: self, off: OFF, i: I, output: output, r1) {
            srl r1 input i;
            sw output r1 i0;

            and r1 input off;
            sw output r1 i1;

            output: [u8; 2]
        }
    }

    pub fn from_be_bytes(bytes: [u8; 2]) -> u16 {
        asm(a: bytes[0], b: bytes[1], i: I, r1) {
            sll  r1 a i;
            or   r1 r1 b;
            r1: u16
        }
    }
}

impl u8 {
    /// The smallest value that can be represented by this integer type.
    pub fn min() -> u8 {
        0
    }

    /// The largest value that can be represented by this integer type,
    /// 2<sup>8</sup> - 1.
    pub fn max() -> u8 {
        255
    }

    /// The size of this integer type in bits.
    pub fn bits() -> u32 {
        8
    }
}

impl b256 {
    /// The smallest value that can be represented by this type.
    pub fn min() -> b256 {
        0x0000000000000000000000000000000000000000000000000000000000000000
    }

    /// The largest value that can be represented by this type,
    /// 2<sup>256</sup> - 1.
    pub fn max() -> b256 {
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
    }

    /// The size of this type in bits.
    pub fn bits() -> u64 {
        256
    }

    pub fn to_le_bytes(self) -> [u8; 32] {
        let (a, b, c, d): (u64, u64, u64, u64) = asm(r1: self) {r1: (u64, u64, u64, u64)};
        let a_bytes = a.to_le_bytes();
        let b_bytes = b.to_le_bytes();
        let c_bytes = c.to_le_bytes();
        let d_bytes = d.to_le_bytes();

        let output = [a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7],
                      b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
                      c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7],
                      d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7]];

        output
    }

    pub fn from_le_bytes(bytes: [u8; 32]) -> b256 {
        let a = u64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]);
        let b = u64::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]]);
        let c = u64::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23]]);
        let d = u64::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27], bytes[28], bytes[29], bytes[30], bytes[31]]);

        let result = (a, b, c, d);

        asm(r1: result) {
            r1: b256
        }
    }

    pub fn to_be_bytes(self) -> [u8; 32] {
        let (a, b, c, d): (u64, u64, u64, u64) = asm(r1: self) {r1: (u64, u64, u64, u64)};
        let a_bytes = a.to_be_bytes();
        let b_bytes = b.to_be_bytes();
        let c_bytes = c.to_be_bytes();
        let d_bytes = d.to_be_bytes();

        let output = [a[0], a[1], a[2], a[3], a[4], a[5], a[6], a[7],
                      b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
                      c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7],
                      d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7]];

        output
    }

    pub fn from_be_bytes(bytes: [u8; 32]) -> b256 {
        let a = u64::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]);
        let b = u64::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]]);
        let c = u64::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23]]);
        let d = u64::from_be_bytes([bytes[24], bytes[25], bytes[26], bytes[27], bytes[28], bytes[29], bytes[30], bytes[31]]);

        let result = (a, b, c, d);

        asm(r1: result) {
            r1: b256
        }
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

    assert(result[0] == 1);
    assert(result[1] == 2);
    assert(result[2] == 3);
    assert(result[3] == 4);
    assert(result[4] == 5);
    assert(result[5] == 6);
    assert(result[6] == 7);
    assert(result[7] == 8);
}

#[test]
fn test_u64_from_le_bytes() {
    let bytes = [1, 2, 3, 4, 5, 6, 7, 8];
    let result = u64::from_le_bytes(bytes);

    assert(result == 578437695752307201);
}

#[test]
fn test_u64_to_be_bytes() {
    let x: u64 = 578437695752307201;
    let result = x.to_be_bytes();

    assert(result[0] == 8);
    assert(result[1] == 7);
    assert(result[2] == 6);
    assert(result[3] == 5);
    assert(result[4] == 4);
    assert(result[5] == 3);
    assert(result[6] == 2);
    assert(result[7] == 1);
}

#[test]
fn test_u64_from_be_bytes() {
    let bytes = [8, 7, 6, 5, 4, 3, 2, 1];
    let result = u64::from_be_bytes(bytes);

    assert(result == 578437695752307201);
}

#[test]
fn test_u32_to_le_bytes() {
    let x: u32 = 67305985;
    let result = x.to_le_bytes();

    assert(result[0] == 1);
    assert(result[1] == 2);
    assert(result[2] == 3);
    assert(result[3] == 4);
}

#[test]
fn test_u32_from_le_bytes() {
    let bytes = [1, 2, 3, 4];
    let result = u32::from_le_bytes(bytes);

    assert(result == 67305985);
}

#[test]
fn test_u32_to_be_bytes() {
    let x: u32 = 67305985;
    let result = x.to_be_bytes();

    assert(result[0] == 4);
    assert(result[1] == 3);
    assert(result[2] == 2);
    assert(result[3] == 1);
}

#[test]
fn test_u32_from_be_bytes() {
    let bytes = [4, 3, 2, 1];
    let result = u32::from_be_bytes(bytes);

    assert(result == 67305985);
}

#[test]
fn test_u16_to_le_bytes() {
    let x: u16 = 513;
    let result = x.to_le_bytes();

    assert(result[0] == 1);
    assert(result[1] == 2);
}

#[test]
fn test_u16_from_le_bytes() {
    let bytes = [1, 2];
    let result = u16::from_le_bytes(bytes);

    assert(result == 513);
}

#[test]
fn test_u16_to_be_bytes() {
    let x: u16 = 513;
    let result = x.to_be_bytes();

    assert(result[0] == 2);
    assert(result[1] == 1);
}

#[test]
fn test_u16_from_be_bytes() {
    let bytes = [2, 1];
    let result = u16::from_be_bytes(bytes);

    assert(result == 513);
}

#[test]
fn test_b256_from_le_bytes() {
    let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
                 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
                 21, 22, 23, 24, 25, 26, 27, 28, 29, 30,
                 31, 32];

    let x = b256::from_le_bytes(bytes);

    assert(x == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20);
}

#[test]
fn test_b256_to_le_bytes() {
    let x: b256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20;

    let bytes = x.to_le_bytes();

    assert(bytes[0] == 1);
    assert(bytes[1] == 2);
    assert(bytes[2] == 3);
    assert(bytes[3] == 4);
    assert(bytes[4] == 5);
    assert(bytes[5] == 6);
    assert(bytes[6] == 7);
    assert(bytes[7] == 8);
    assert(bytes[8] == 9);
    assert(bytes[9] == 10);
    assert(bytes[10] == 11);
    assert(bytes[11] == 12);
    assert(bytes[12] == 13);
    assert(bytes[13] == 14);
    assert(bytes[14] == 15);
    assert(bytes[15] == 16);
    assert(bytes[16] == 17);
    assert(bytes[17] == 18);
    assert(bytes[18] == 19);
    assert(bytes[19] == 20);
    assert(bytes[20] == 21);
    assert(bytes[21] == 22);
    assert(bytes[22] == 23);
    assert(bytes[23] == 24);
    assert(bytes[24] == 25);
    assert(bytes[25] == 26);
    assert(bytes[26] == 27);
    assert(bytes[27] == 28);
    assert(bytes[28] == 29);
    assert(bytes[29] == 30);
    assert(bytes[30] == 31);
    assert(bytes[31] == 32);
}

#[test]
fn test_b256_from_be_bytes() {
    let bytes = [32, 31, 30, 29, 28, 27, 26, 25, 24, 23,
                 22, 21, 20, 19, 18, 17, 16, 15, 14, 13,
                 12, 11, 10, 9, 8, 7, 6, 5, 4, 3,
                 2, 1];

    let x = b256::from_be_bytes(bytes);

    assert(x == 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20);
}

#[test]
fn test_b256_to_be_bytes() {
    let x: b256 = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20;

    let bytes = x.to_be_bytes();

    assert(bytes[0] == 1);
    assert(bytes[1] == 2);
    assert(bytes[2] == 3);
    assert(bytes[3] == 4);
    assert(bytes[4] == 5);
    assert(bytes[5] == 6);
    assert(bytes[6] == 7);
    assert(bytes[7] == 8);
    assert(bytes[8] == 9);
    assert(bytes[9] == 10);
    assert(bytes[10] == 11);
    assert(bytes[11] == 12);
    assert(bytes[12] == 13);
    assert(bytes[13] == 14);
    assert(bytes[14] == 15);
    assert(bytes[15] == 16);
    assert(bytes[16] == 17);
    assert(bytes[17] == 18);
    assert(bytes[18] == 19);
    assert(bytes[19] == 20);
    assert(bytes[20] == 21);
    assert(bytes[21] == 22);
    assert(bytes[22] == 23);
    assert(bytes[23] == 24);
    assert(bytes[24] == 25);
    assert(bytes[25] == 26);
    assert(bytes[26] == 27);
    assert(bytes[27] == 28);
    assert(bytes[28] == 29);
    assert(bytes[29] == 30);
    assert(bytes[30] == 31);
    assert(bytes[31] == 32);
}