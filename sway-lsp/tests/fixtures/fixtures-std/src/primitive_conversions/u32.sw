library;

use ::convert::TryFrom;
use ::option::Option::{self, *};

impl u32 {
    pub fn try_as_u8(self) -> Option<u8> {
        if self <= u8::max().as_u32() {
            Some(asm(input: self) {
                input: u8
            })
        } else {
            None
        }
    }

    pub fn try_as_u16(self) -> Option<u16> {
        if self <= u16::max().as_u32() {
            Some(asm(input: self) {
                input: u16
            })
        } else {
            None
        }
    }
}


impl TryFrom<u64> for u32 {
    fn try_from(u: u64) -> Option<Self> {
        if u > u32::max().as_u64() {
            None
        } else {
            Some(asm(r1: u) {r1: u32})
        }
    }
}

impl TryFrom<u256> for u32 {
    fn try_from(u: u256) -> Option<Self> {
        let parts = asm(r1: u) { r1: (u64, u64, u64, u64) };

        if parts.0 != 0 || parts.1 != 0 || parts.2 != 0 || parts.3 > u32::max().as_u64() {
            None
        } else {
            Some(asm(r1: parts.3) {r1: u32})
        }
    }
}

#[test]
fn test_u32_try_from_u64() {
    use ::assert::assert;
    
    let u64_1: u64 = 2;
    let u64_2: u64 = u32::max().as_u64() + 1;

    let u32_1 = <u32 as TryFrom<u64>>::try_from(u64_1);
    let u32_2 = <u32 as TryFrom<u64>>::try_from(u64_2);

    assert(u32_1.is_some());
    assert(u32_1.unwrap() == 2u32);

    assert(u32_2.is_none());
}

#[test]
fn test_u32_try_from_u256() {
    use ::assert::assert;
    
    let u256_1: u256 = 0x0000000000000000000000000000000000000000000000000000000000000002u256;
    let u256_2: u256 = 0x1000000000000000000000000000000000000000000000000000000000000000u256;

    let u32_1 = <u32 as TryFrom<u256>>::try_from(u256_1);
    let u32_2 = <u32 as TryFrom<u256>>::try_from(u256_2);

    assert(u32_1.is_some());
    assert(u32_1.unwrap() == 2u32);

    assert(u32_2.is_none());
}
