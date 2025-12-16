library;

use std::bytes_conversions::u256::*;
use std::bytes_conversions::b256::*;

pub trait TestInstance {
    fn new() -> Self;
    fn different() -> Self;
}

impl TestInstance for bool {
    fn new() -> Self {
        true
    }
    fn different() -> Self {
        false
    }
}

impl TestInstance for u8 {
    fn new() -> Self {
        123
    }
    fn different() -> Self {
        223
    }
}

impl TestInstance for u16 {
    fn new() -> Self {
        1234
    }
    fn different() -> Self {
        4321
    }
}

impl TestInstance for u32 {
    fn new() -> Self {
        12345
    }
    fn different() -> Self {
        54321
    }
}

impl TestInstance for u64 {
    fn new() -> Self {
        123456
    }
    fn different() -> Self {
        654321
    }
}

impl TestInstance for u256 {
    fn new() -> Self {
        0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20u256
    }
    fn different() -> Self {
        0x0203040405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1f1a20u256
    }
}

impl TestInstance for str {
    fn new() -> Self {
        "1a2B3c"
    }
    fn different() -> Self {
        "3A2b1C"
    }
}

impl TestInstance for str[6] {
    fn new() -> Self {
        __to_str_array("1a2B3c")
    }
    fn different() -> Self {
        __to_str_array("3A2b1C")
    }
}

impl TestInstance for [u64; 2] {
    fn new() -> Self {
        [123456, 654321]
    }
    fn different() -> Self {
        [654321, 123456]
    }
}

pub struct Struct {
    pub x: u64,
}

impl PartialEq for Struct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x
    }
}
impl Eq for Struct {}

impl TestInstance for Struct {
    fn new() -> Self {
        Self { x: 98765 }
    }
    fn different() -> Self {
        Self { x: 56789 }
    }
}

pub struct EmptyStruct {}

impl PartialEq for EmptyStruct {
    fn eq(self, other: Self) -> bool {
        true
    }
}
impl Eq for EmptyStruct {}

impl TestInstance for EmptyStruct {
    fn new() -> Self {
        EmptyStruct {}
    }
    fn different() -> Self {
        EmptyStruct {}
    }
}

pub enum Enum {
    A: u64,
}

impl PartialEq for Enum {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (Enum::A(l), Enum::A(r)) => l == r,
        }
    }
}
impl Eq for Enum {}

impl TestInstance for Enum {
    fn new() -> Self {
        Self::A(123456)
    }
    fn different() -> Self {
        Self::A(654321)
    }
}

impl TestInstance for (u8, u32) {
    fn new() -> Self {
        (123, 12345)
    }
    fn different() -> Self {
        (223, 54321)
    }
}

impl TestInstance for b256 {
    fn new() -> Self {
        0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
    }
    fn different() -> Self {
        0x0202020405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1f1a20
    }
}

impl TestInstance for raw_slice {
    fn new() -> Self {
        let null_ptr = asm() {
            zero: raw_ptr
        };

        raw_slice::from_parts::<u64>(null_ptr, 42)
    }
    fn different() -> Self {
        let null_ptr = asm() {
            zero: raw_ptr
        };

        raw_slice::from_parts::<u64>(null_ptr, 42 * 2)
    }
}

impl TestInstance for () {
    fn new() -> Self {
        ()
    }
    fn different() -> Self {
        ()
    }
}

impl TestInstance for [u64; 0] {
    fn new() -> Self {
        []
    }
    fn different() -> Self {
        []
    }
}

pub struct RawPtrNewtype {
    ptr: raw_ptr,
}

impl TestInstance for RawPtrNewtype {
    fn new() -> Self {
        let null_ptr = asm() {
            zero: raw_ptr
        };

        Self {
            ptr: null_ptr.add::<u64>(42),
        }
    }
    fn different() -> Self {
        let null_ptr = asm() {
            zero: raw_ptr
        };

        Self {
            ptr: null_ptr.add::<u64>(42 * 2),
        }
    }
}

impl AbiEncode for RawPtrNewtype {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let ptr_as_u64 = asm(p: self.ptr) {
            p: u64
        };
        ptr_as_u64.abi_encode(buffer)
    }
}

impl PartialEq for RawPtrNewtype {
    fn eq(self, other: Self) -> bool {
        self.ptr == other.ptr
    }
}
impl Eq for RawPtrNewtype {}