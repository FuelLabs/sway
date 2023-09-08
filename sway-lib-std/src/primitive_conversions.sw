library;

use ::option::Option::{self, *};
use ::assert::*;
use core::primitive_conversions::*;

impl u16 {
    pub fn try_as_u8(self) -> Option<u8> {
        if self <= u8::max().as_u16() {
            Some(asm(input: self) {
                input: u8
            })
        } else {
            None
        }
    }
}

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

impl str {
    pub fn try_as_str_array<S>(self) -> Option<S> {
         __assert_is_str_array::<S>();
        let str_size = __size_of_str_array::<S>();
        let source = self.as_ptr();

        if self.len() == str_size {
            let s: S = asm(str_size: str_size, source: source, dest) {
                move dest sp;
                cfe str_size;
                mcp dest source str_size;
                dest: S
            };
            asm (str_size: str_size) {
                cfs str_size;
            }
            Some(s)
        } else {
            None
        }
    }
}

#[test]
fn str_slice_to_str_array() {
    use ::assert::*;
    use core::str::*;

    let a = "abcd";
    let b: str[4] = a.try_as_str_array().unwrap();
    let c = from_str_array(b);
    
    assert(a == c);
}