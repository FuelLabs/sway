library;

use core::ops::Eq;

pub trait New {
    fn new() -> Self;
}

impl New for bool {
    fn new() -> Self {
        true
    }
}

impl New for u8 {
    fn new() -> Self {
        123
    }
}

impl New for u16 {
    fn new() -> Self {
        1234
    }
}

impl New for u32 {
    fn new() -> Self {
        12345
    }
}

impl New for u64 {
    fn new() -> Self {
        123456
    }
}

impl New for u256 {
    fn new() -> Self {
        0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20_u256
    }
}

impl New for str {
    fn new() -> Self {
        "1a2B3c"
    }
}

impl Eq for str[6] {
    fn eq(self, other: Self) -> bool {
        let mut i = 0;
        while i < 6 {
            let ptr_self = __addr_of(self).add::<u8>(i);
            let ptr_other = __addr_of(other).add::<u8>(i);

            if ptr_self.read::<u8>() != ptr_other.read::<u8>() {
                return false;
            }

            i = i + 1;
        };
        
        true
    }
}

impl New for str[6] {
    fn new() -> Self {
        __to_str_array("1a2B3c")
    }
}

impl Eq for [u64;2] {
    fn eq(self, other: Self) -> bool {
        self[0] == other[0] && self[1] == other[1] 
    }
}

impl New for [u64;2] {
    fn new() -> Self {
        [123456, 654321]
    }
}

pub struct Struct {
    x: u64,
}

impl Eq for Struct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x
    }
}

impl New for Struct {
    fn new() -> Self {
        Self { x: 98765 }
    }
}

pub enum Enum {
    A: u64,
}

impl Eq for Enum {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (Enum::A(l), Enum::A(r)) => l == r,
        }
    }
}

impl New for Enum {
    fn new() -> Self {
        Self::A(123456)
    }
}

impl Eq for (u8, u32) {
    fn eq(self, other: Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl New for (u8, u32) {
    fn new() -> Self {
        (123, 12345)
    }
}

impl New for b256 {
    fn new() -> Self {
        0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
    }
}

impl New for raw_ptr {
    fn new() -> Self {
        let null_ptr = asm() { zero: raw_ptr };

        null_ptr.add::<u64>(42)
    }
}

impl New for raw_slice {
    fn new() -> Self {
        let null_ptr = asm() { zero: raw_ptr };
        
        std::raw_slice::from_parts::<u64>(null_ptr, 42)
    }
}

impl Eq for raw_slice {
    fn eq(self, other: Self) -> bool {
        self.ptr() == other.ptr() && self.number_of_bytes() == other.number_of_bytes()
    }
}

impl New for &u8 {
    fn new() -> Self {
        let ptr = asm() { zero: raw_ptr }.add::<u64>(42);
        
        asm(r: ptr) { r: &u8 }
    }
}

impl Eq for &u8 {
    fn eq(self, other: Self) -> bool {
        // For the purpose of this test, we consider two references
        // to be equal if they point to the same memory location,
        // not if values they point to are equal.
        let self_ptr = asm(r: self) { r: raw_ptr };
        let other_ptr = asm(r: other) { r: raw_ptr };

        self_ptr == other_ptr
    }
}