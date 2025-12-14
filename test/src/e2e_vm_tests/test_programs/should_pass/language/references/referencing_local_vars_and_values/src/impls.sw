library;

pub trait New {
    fn new() -> Self;
}

pub trait ZeroSize {
}

impl ZeroSize for () {}

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
        // We cannot just return a constant here, because it would
        // end up being a reference type constant which means
        // we will get the same reference for each `new()`
        // instance which breaks the semantics of the test.
        // That's why this acrobatics here, to disable optimizations.
        let mut result = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20u256;
        poke(result)
    }
}

impl New for str {
    fn new() -> Self {
        "1a2B3c"
    }
}

impl PartialEq for str[6] {
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
impl Eq for str[6] {}

impl New for str[6] {
    fn new() -> Self {
        __to_str_array("1a2B3c")
    }
}

impl PartialEq for [u64; 2] {
    fn eq(self, other: Self) -> bool {
        self[0] == other[0] && self[1] == other[1]
    }
}
impl Eq for [u64; 2] {}

impl New for [u64; 2] {
    fn new() -> Self {
        [123456, 654321]
    }
}

pub struct Struct {
    x: u64,
}

impl PartialEq for Struct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x
    }
}
impl Eq for Struct {}

impl New for Struct {
    fn new() -> Self {
        Self { x: 98765 }
    }
}

pub struct EmptyStruct {}

impl PartialEq for EmptyStruct {
    fn eq(self, other: Self) -> bool {
        true
    }
}
impl Eq for EmptyStruct {}

impl New for EmptyStruct {
    fn new() -> Self {
        EmptyStruct {}
    }
}

impl ZeroSize for EmptyStruct {}

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

impl New for Enum {
    fn new() -> Self {
        Self::A(123456)
    }
}

impl New for (u8, u32) {
    fn new() -> Self {
        (123, 12345)
    }
}

impl New for b256 {
    fn new() -> Self {
        // We cannot just return a constant here, because it would
        // end up being a reference type constant which means
        // we will get the same reference for each `new()`
        // instance which breaks the semantics of the test.
        // That's why this acrobatics here, to disable optimizations.
        let mut result = 0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20;
        poke(result)
    }
}

impl New for raw_ptr {
    fn new() -> Self {
        let null_ptr = asm() {
            zero: raw_ptr
        };

        null_ptr.add::<u64>(42)
    }
}

impl New for raw_slice {
    fn new() -> Self {
        let null_ptr = asm() {
            zero: raw_ptr
        };

        raw_slice::from_parts::<u64>(null_ptr, 42)
    }
}

impl New for () {
    fn new() -> Self {
        ()
    }
}

impl New for [u64; 0] {
    fn new() -> Self {
        []
    }
}

impl PartialEq for [u64; 0] {
    fn eq(self, other: Self) -> bool {
        true
    }
}
impl Eq for [u64; 0] {}

impl ZeroSize for [u64; 0] {}

#[inline(never)]
fn poke<T>(ref mut x: T) -> T {
    x
}
