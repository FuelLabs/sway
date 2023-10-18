library;

use ::assert::assert;
use ::bytes::Bytes;
use ::alloc::alloc;
use ::option::Option;

impl u32 {
    /// Converts the `u32` to a sequence of little-endian bytes.
    /// 
    /// # Returns
    ///
    /// * [Bytes] - The 4 bytes that compose the `u32`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes_conversions::u32::*;
    ///
    /// fn foo() {
    ///     let x: u32 = 67305985;
    ///     let result = x.to_le_bytes();
    /// 
    ///     assert(result.get(0).unwrap() == 1_u8);
    ///     assert(result.get(1).unwrap() == 2_u8);
    ///     assert(result.get(2).unwrap() == 3_u8);
    ///     assert(result.get(3).unwrap() == 4_u8);
    /// }
    /// ```
    pub fn to_le_bytes(self) -> Bytes {
        let ptr = asm(input: self, off: 0xFF, i: 0x8, j: 0x10, k: 0x18, size: 4, ptr, r1) {
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

            ptr: raw_ptr
        };

        let rs = asm(parts: (ptr, 4)) {
            parts: raw_slice
        };

        Bytes::from(rs)
    }

    /// Converts a sequence of little-endian bytes to a `u32`.
    ///
    /// # Arguments
    /// 
    /// * `bytes`: [Bytes] - The 4 bytes that compose the `u32`.
    ///
    /// # Returns
    ///
    /// * [u32] - The resulting `u32` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{bytes::Bytes, bytes_conversions::u32::*};
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(1_u8);
    ///     bytes.push(2_u8);
    ///     bytes.push(3_u8);
    ///     bytes.push(4_u8);
    ///     let result = u32::from_le_bytes(bytes);
    ///
    ///     assert(result == 67305985_u32);
    /// }
    /// ```
    pub fn from_le_bytes(bytes: Bytes) -> Self {
        assert(bytes.len() == 4);
        let ptr = bytes.buf.ptr();
        let a = ptr.read_byte();
        let b = (ptr.add_uint_offset(1)).read_byte();
        let c = (ptr.add_uint_offset(2)).read_byte();
        let d = (ptr.add_uint_offset(3)).read_byte();

        asm(a: a, b: b, c: c, d: d, i: 0x8, j: 0x10, k: 0x18, r1, r2, r3) {
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
    /// * [Bytes] - The 4 bytes that compose the `u32`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes_conversions::u32::*;
    ///
    /// fn foo() {
    ///     let x: u32 = 67305985;
    ///     let result = x.to_be_bytes();
    ///
    ///     assert(result.get(0).unwrap() == 4_u8);
    ///     assert(result.get(1).unwrap() == 3_u8);
    ///     assert(result.get(2).unwrap() == 2_u8);
    ///     assert(result.get(3).unwrap() == 1_u8);
    /// }
    /// ```
    pub fn to_be_bytes(self) -> Bytes {
        let ptr = asm(input: self, off: 0xFF, i: 0x8, j: 0x10, k: 0x18, size: 4, ptr, r1) {
            aloc size;
            move ptr hp;
            
            srl  r1 input k;
            and  r1 r1 off;
            sb   ptr r1 i0;

            srl  r1 input j;
            and  r1 r1 off;
            sb   ptr r1 i1;

            srl  r1 input i;
            and  r1 r1 off;
            sb   ptr r1 i2;

            and  r1 input off;
            sb   ptr r1 i3;

            ptr: raw_ptr
        };

        let rs = asm(parts: (ptr, 4)) {
            parts: raw_slice
        };
        
        Bytes::from(rs)
    }

    /// Converts a sequence of big-endian bytes to a `u32`.
    ///
    /// # Arguments
    /// 
    /// * `bytes`: [Bytes] - The 4 bytes that compose the `u32`.
    ///
    /// # Returns
    ///
    /// * [u32] - The resulting `u32` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{bytes::Bytes, bytes_conversions::u32::*};
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(4_u8);
    ///     bytes.push(3_u8);
    ///     bytes.push(2_u8);
    ///     bytes.push(1_u8);
    ///     let result = u32::from_be_bytes(bytes);
    ///
    ///     assert(result == 67305985_u32);
    /// }
    /// ```
    pub fn from_be_bytes(bytes: Bytes) -> Self {
        assert(bytes.len() == 4);
        let ptr = bytes.buf.ptr();
        let a = ptr.read_byte();
        let b = (ptr.add_uint_offset(1)).read_byte();
        let c = (ptr.add_uint_offset(2)).read_byte();
        let d = (ptr.add_uint_offset(3)).read_byte();

        asm(a: a, b: b, c: c, d: d, i: 0x8, j: 0x10, k: 0x18, r1, r2, r3) {
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

    assert(result.get(0).unwrap() == 1_u8);
    assert(result.get(1).unwrap() == 2_u8);
    assert(result.get(2).unwrap() == 3_u8);
    assert(result.get(3).unwrap() == 4_u8);
}

#[test]
fn test_u32_from_le_bytes() {
    let mut bytes = Bytes::new();
    bytes.push(1_u8);
    bytes.push(2_u8);
    bytes.push(3_u8);
    bytes.push(4_u8);
    let result = u32::from_le_bytes(bytes);

    assert(result == 67305985_u32);
}

#[test]
fn test_u32_to_be_bytes() {
    let x: u32 = 67305985;
    let result = x.to_be_bytes();

    assert(result.get(0).unwrap() == 4_u8);
    assert(result.get(1).unwrap() == 3_u8);
    assert(result.get(2).unwrap() == 2_u8);
    assert(result.get(3).unwrap() == 1_u8);
}

#[test]
fn test_u32_from_be_bytes() {
    let mut bytes = Bytes::new();
    bytes.push(4_u8);
    bytes.push(3_u8);
    bytes.push(2_u8);
    bytes.push(1_u8);
    let result = u32::from_be_bytes(bytes);

    assert(result == 67305985_u32);
}
