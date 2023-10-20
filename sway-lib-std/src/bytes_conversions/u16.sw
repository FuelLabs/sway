library;

use ::assert::assert;
use ::bytes::Bytes;
use ::alloc::alloc;
use ::option::Option;

impl u16 {
    /// Converts the `u16` to a sequence of little-endian bytes.
    /// 
    /// # Returns
    ///
    /// * [Bytes] - The 2 bytes that compose the `u16`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let x: u16 = 513;
    ///     let result = x.to_le_bytes();
    /// 
    ///     assert(result.get(0).unwrap() == 1_u8);
    ///     assert(result.get(1).unwrap() == 2_u8);
    /// }
    /// ```
    pub fn to_le_bytes(self) -> Bytes {
        let ptr = asm(input: self, off: 0xFF, i: 0x8, size: 2, ptr, r1) {
            aloc size;
            move ptr hp;
            
            and r1 input off;
            sb ptr r1 i0;

            srl r1 input i;
            and r1 r1 off;
            sb ptr r1 i1;            

            ptr: raw_ptr
        };

        let rs = asm(parts: (ptr, 2)) {
            parts: raw_slice
        };

        Bytes::from(rs)
    }

    /// Converts a sequence of little-endian bytes to a `u16`.
    ///
    /// # Arguments
    /// 
    /// * `bytes`: [Bytes] - The 2 bytes that compose the `u16`.
    ///
    /// # Returns
    ///
    /// * [u16] - The resulting `u16` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(1_u8);
    ///     bytes.push(2_u8);
    ///     let result = u16::from_le_bytes(bytes);
    ///
    ///     assert(result == 513_u16);
    /// }
    /// ```
    pub fn from_le_bytes(bytes: Bytes) -> Self {
        assert(bytes.len() == 2);
        let ptr = bytes.buf.ptr();
        let a = ptr.read_byte();
        let b = (ptr.add_uint_offset(1)).read_byte();
        let i = 0x8;
        asm(a: a, b: b, i: i, r1) {
            sll  r1 b i;
            or   r1 a r1;
            r1: u16
        }
    }

    /// Converts the `u16` to a sequence of big-endian bytes.
    /// 
    /// # Returns
    ///
    /// * [Bytes] - The 2 bytes that compose the `u16`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes_conversions::u16;
    ///
    /// fn foo() {
    ///     let x: u16 = 513;
    ///     let result = x.to_be_bytes();
    ///
    ///     assert(result.get(0).unwrap() == 2_u8);
    ///     assert(result.get(1).unwrap() == 1_u8);
    /// }
    /// ```
    pub fn to_be_bytes(self) -> Bytes {
        let ptr = asm(input: self, off: 0xFF, i: 0x8, size: 2, ptr, r1) {
            aloc size;
            move ptr hp;
            
            srl r1 input i;
            sb ptr r1 i0;

            and r1 input off;
            sb ptr r1 i1;

            ptr: raw_ptr
        };

        let rs = asm(parts: (ptr, 2)) {
            parts: raw_slice
        };
        
        Bytes::from(rs)
    }

    /// Converts a sequence of big-endian bytes to a `u16`.
    ///
    /// # Arguments
    /// 
    /// * `bytes`: [Bytes] - The 2 bytes that compose the `u16`.
    ///
    /// # Returns
    ///
    /// * [u16] - The resulting `u16` value.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::bytes::Bytes;
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(2_u8);
    ///     bytes.push(1_u8);
    ///     let result = u16::from_be_bytes(bytes);
    ///
    ///     assert(result == 513_u16);
    /// }
    /// ```
    pub fn from_be_bytes(bytes: Bytes) -> Self {
        assert(bytes.len() == 2);
        let ptr = bytes.buf.ptr();
        let a = ptr.read_byte();
        let b = (ptr.add_uint_offset(1)).read_byte();

        asm(a: a, b: b, i: 0x8, r1) {
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

    assert(result.get(0).unwrap() == 1_u8);
    assert(result.get(1).unwrap() == 2_u8);
}

#[test]
fn test_u16_from_le_bytes() {
    let mut bytes = Bytes::new();
    bytes.push(1_u8);
    bytes.push(2_u8);
    let result = u16::from_le_bytes(bytes);

    assert(result == 513_u16);
}

#[test]
fn test_u16_to_be_bytes() {
    let x: u16 = 513;
    let result = x.to_be_bytes();

    assert(result.get(0).unwrap() == 2_u8);
    assert(result.get(1).unwrap() == 1_u8);
}

#[test]
fn test_u16_from_be_bytes() {
    let mut bytes = Bytes::new();
    bytes.push(2_u8);
    bytes.push(1_u8);
    let result = u16::from_be_bytes(bytes);

    assert(result == 513_u16);
}
