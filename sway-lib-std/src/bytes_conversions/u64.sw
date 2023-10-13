library;

use ::assert::assert;
use ::bytes::Bytes;
use ::alloc::alloc;
use ::option::Option;

impl u64 {
    /// Converts the `u64` to a sequence of little-endian bytes.
    /// 
    /// # Returns
    ///
    /// * [Bytes] - The bytes that compose the `u64`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes_conversions::u64::*;
    ///
    /// fn foo() {
    ///     let x: u64 = 578437695752307201;
    ///     let result = x.to_le_bytes();
    /// 
    ///     assert(result.get(0).unwrap() == 1_u8);
    ///     assert(result.get(1).unwrap() == 2_u8);
    ///     assert(result.get(2).unwrap() == 3_u8);
    ///     assert(result.get(3).unwrap() == 4_u8);
    ///     assert(result.get(4).unwrap() == 5_u8);
    ///     assert(result.get(5).unwrap() == 6_u8);
    ///     assert(result.get(6).unwrap() == 7_u8);
    ///     assert(result.get(7).unwrap() == 8_u8);
    /// }
    /// ```
    pub fn to_le_bytes(self) -> Bytes {
        let ptr = asm(input: self, off: 0xFF, i: 0x8, j: 0x10, k: 0x18, l: 0x20, m: 0x28, n: 0x30, o: 0x38, size: 8, ptr, r1) {
            aloc size;
            move ptr hp;
            
            and r1 input off;
            sb ptr r1 i0;

            srl r1 input i;
            and r1 r1 off;
            sb ptr r1 i1;

            srl r1 input j;
            and r1 r1 off;
            sb ptr r1 i2;

            srl r1 input k;
            and r1 r1 off;
            sb ptr r1 i3;

            srl r1 input l;
            and r1 r1 off;
            sb ptr r1 i4;

            srl r1 input m;
            and r1 r1 off;
            sb ptr r1 i5;

            srl r1 input n;
            and r1 r1 off;
            sb ptr r1 i6;

            srl r1 input o;
            and r1 r1 off;
            sb ptr r1 i7;

            ptr: raw_ptr
        };

        let rs = asm(parts: (ptr, 8)) {
            parts: raw_slice
        };

        Bytes::from(rs)
    }

    /// Converts a sequence of little-endian bytes to a `u64`.
    ///
    /// # Arguments
    /// 
    /// * `bytes`: [Bytes] - A `Bytes` object that represent a `u64`.
    ///
    /// # Returns
    ///
    /// * [u64] - The resulting `u64` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{bytes::Bytes, bytes_conversions::u64::*};
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(1_u8);
    ///     bytes.push(2_u8);
    ///     bytes.push(3_u8);
    ///     bytes.push(4_u8);
    ///     bytes.push(5_u8);
    ///     bytes.push(6_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(8_u8);
    ///     let result = u64::from_le_bytes(bytes);
    ///
    ///     assert(result == 578437695752307201);
    /// }
    /// ```
    pub fn from_le_bytes(bytes: Bytes) -> Self {
        assert(bytes.len() == 8);
        let ptr = bytes.buf.ptr();
        let a = ptr.read_byte();
        let b = (ptr.add_uint_offset(1)).read_byte();
        let c = (ptr.add_uint_offset(2)).read_byte();
        let d = (ptr.add_uint_offset(3)).read_byte();
        let e = (ptr.add_uint_offset(4)).read_byte();
        let f = (ptr.add_uint_offset(5)).read_byte();
        let g = (ptr.add_uint_offset(6)).read_byte();
        let h = (ptr.add_uint_offset(7)).read_byte();

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
    /// * [Bytes] - The bytes that compose the `u64`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes_conversions::u64::*;
    ///
    /// fn foo() {
    ///     let x: u64 = 578437695752307201;
    ///     let result = x.to_be_bytes();
    ///
    ///     assert(result.get(0).unwrap() == 8_u8);
    ///     assert(result.get(1).unwrap() == 7_u8);
    ///     assert(result.get(2).unwrap() == 6_u8);
    ///     assert(result.get(3).unwrap() == 5_u8);
    ///     assert(result.get(4).unwrap() == 4_u8);
    ///     assert(result.get(5).unwrap() == 3_u8);
    ///     assert(result.get(6).unwrap() == 2_u8);
    ///     assert(result.get(7).unwrap() == 1_u8);
    /// }
    /// ```
    pub fn to_be_bytes(self) -> Bytes {
        let ptr = asm(input: self, off: 0xFF, i: 0x8, j: 0x10, k: 0x18, l: 0x20, m: 0x28, n: 0x30, o: 0x38, size: 8, ptr, r1) {
            aloc size;
            move ptr hp;

            and  r1 input off;
            sb  ptr r1 i7;

            srl  r1 input i;
            and  r1 r1 off;
            sb  ptr r1 i6;

            srl  r1 input j;
            and  r1 r1 off;
            sb  ptr r1 i5;

            srl  r1 input k;
            and  r1 r1 off;
            sb  ptr r1 i4;

            srl  r1 input l;
            and  r1 r1 off;
            sb  ptr r1 i3;

            srl  r1 input m;
            and  r1 r1 off;
            sb  ptr r1 i2;

            srl  r1 input n;
            and  r1 r1 off;
            sb  ptr r1 i1;

            srl  r1 input o;
            and  r1 r1 off;
            sb  ptr r1 i0;

            ptr: raw_ptr
        };
        
        let rs = asm(parts: (ptr, 8)) {
            parts: raw_slice
        };

        Bytes::from(rs)
    }

    /// Converts a sequence of big-endian bytes to a `u64`.
    ///
    /// # Arguments
    /// 
    /// * `bytes`: [Bytes] - A `Bytes` object that represent a `u64`.
    ///
    /// # Returns
    ///
    /// * [u64] - The resulting `u64` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{bytes::Bytes, bytes_conversions::u64::*};
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(8_u8);
    ///     bytes.push(7_u8);
    ///     bytes.push(6_u8);
    ///     bytes.push(5_u8);
    ///     bytes.push(4_u8);
    ///     bytes.push(3_u8);
    ///     bytes.push(2_u8);
    ///     bytes.push(1_u8);
    ///     let result = u64::from_be_bytes(bytes);
    ///
    ///     assert(result == 578437695752307201);
    /// }
    /// ```
    pub fn from_be_bytes(bytes: Bytes) -> Self {
        assert(bytes.len() == 8);
        let ptr = bytes.buf.ptr();
        let h = ptr.read_byte();
        let g = (ptr.add_uint_offset(1)).read_byte();
        let f = (ptr.add_uint_offset(2)).read_byte();
        let e = (ptr.add_uint_offset(3)).read_byte();
        let d = (ptr.add_uint_offset(4)).read_byte();
        let c = (ptr.add_uint_offset(5)).read_byte();
        let b = (ptr.add_uint_offset(6)).read_byte();
        let a = (ptr.add_uint_offset(7)).read_byte();

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
}

#[test]
fn test_u64_to_be_bytes() {
    let x: u64 = 578437695752307201;
    let result = x.to_be_bytes();

    assert(result.get(0).unwrap() == 8_u8);
    assert(result.get(1).unwrap() == 7_u8);
    assert(result.get(2).unwrap() == 6_u8);
    assert(result.get(3).unwrap() == 5_u8);
    assert(result.get(4).unwrap() == 4_u8);
    assert(result.get(5).unwrap() == 3_u8);
    assert(result.get(6).unwrap() == 2_u8);
    assert(result.get(7).unwrap() == 1_u8);
}

#[test]
fn test_u64_from_be_bytes() {
    let mut bytes = Bytes::new();
    bytes.push(8_u8);
    bytes.push(7_u8);
    bytes.push(6_u8);
    bytes.push(5_u8);
    bytes.push(4_u8);
    bytes.push(3_u8);
    bytes.push(2_u8);
    bytes.push(1_u8);
    let result = u64::from_be_bytes(bytes);

    assert(result == 578437695752307201);
}

#[test]
fn test_u64_to_le_bytes() {
    let x: u64 = 578437695752307201;
    let result = x.to_le_bytes();

    assert(result.get(0).unwrap() == 1_u8);
    assert(result.get(1).unwrap() == 2_u8);
    assert(result.get(2).unwrap() == 3_u8);
    assert(result.get(3).unwrap() == 4_u8);
    assert(result.get(4).unwrap() == 5_u8);
    assert(result.get(5).unwrap() == 6_u8);
    assert(result.get(6).unwrap() == 7_u8);
    assert(result.get(7).unwrap() == 8_u8);
}

#[test]
fn test_u64_from_le_bytes() {
    let mut bytes = Bytes::new();
    bytes.push(1_u8);
    bytes.push(2_u8);
    bytes.push(3_u8);
    bytes.push(4_u8);
    bytes.push(5_u8);
    bytes.push(6_u8);
    bytes.push(7_u8);
    bytes.push(8_u8);
    let result = u64::from_le_bytes(bytes);

    assert(result == 578437695752307201);
}
