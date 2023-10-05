library;

use ::convert::TryFrom;
use ::option::Option::{self, *};

impl u64 {
    pub fn try_as_u8(self) -> Option<u8> {
        if self <= u8::max().as_u64() {
            Some(asm(input: self) {
                input: u8
            })
        } else {
            None
        }
    }

    pub fn try_as_u16(self) -> Option<u16> {
        if self <= u16::max().as_u64() {
            Some(asm(input: self) {
                input: u16
            })
        } else {
            None
        }
    }
    
    pub fn try_as_u32(self) -> Option<u32> {
        if self <= u32::max().as_u64() {
            Some(asm(input: self) {
                input: u32
            })
        } else {
            None
        }
    }
}

impl TryFrom<u256> for u64 {
    fn try_from(u: u256) -> Option<Self> {
        let parts = asm(r1: u) { r1: (u64, u64, u64, u64) };

        if parts.0 != 0 || parts.1 != 0 || parts.2 != 0 {
            None
        } else {
            Some(parts.3)
        }
    }
}

#[test]
fn test_u64_try_from_u256() {
    use ::assert::assert;
    
    let u256_1 = 0x0000000000000000000000000000000000000000000000000000000000000002u256;
    let u256_2 = 0x1000000000000000000000000000000000000000000000000000000000000000u256;

    let u64_1 = u64::try_from(u256_1);
    let u64_2 = u64::try_from(u256_2);

    assert(u64_1.is_some());
    assert(u64_1.unwrap() == 2);

    assert(u64_2.is_none());
}
