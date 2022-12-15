library data_structures;

use core::ops::*;
use std::hash::sha256;

/////////////////////////////////////////////////////////////////////////////
// Data Structures Used in in the Tests
/////////////////////////////////////////////////////////////////////////////
pub struct MyStruct {
    x: u64,
    y: u64,
}

pub enum MyEnum {
    X: u64,
    Y: u64,
}

impl Eq for MyStruct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for MyEnum {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (MyEnum::X(val1), MyEnum::X(val2)) => val1 == val2,
            (MyEnum::Y(val1), MyEnum::Y(val2)) => val1 == val2,
            _ => false,
        }
    }
}

impl Eq for (u64, u64) {
    fn eq(self, other: Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl Eq for [u64; 3] {
    fn eq(self, other: Self) -> bool {
        self[0] == other[0] && self[1] == other[1] && self[2] == other[2]
    }
}

impl Eq for str[4] {
    fn eq(self, other: Self) -> bool {
        sha256(self) == sha256(other)
    }
}

/////////////////////////////////////////////////////////////////////////////
// Error 
/////////////////////////////////////////////////////////////////////////////
pub enum Error {
    BoolError: bool,
    U8Error: u8,
    U16Error: u16,
    U32Error: u32,
    U64Error: u64,
    StructError: MyStruct,
    EnumError: MyEnum,
    TupleError: (u64, u64),
    ArrayError: [u64; 3],
    StringError: str[4],
}

impl Eq for Error {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (Error::BoolError(val1), Error::BoolError(val2)) => val1 == val2,
            (Error::U8Error(val1), Error::U8Error(val2)) => val1 == val2,
            (Error::U16Error(val1), Error::U16Error(val2)) => val1 == val2,
            (Error::U32Error(val1), Error::U32Error(val2)) => val1 == val2,
            (Error::U64Error(val1), Error::U64Error(val2)) => val1 == val2,
            (Error::StructError(val1), Error::StructError(val2)) => val1 == val2,
            (Error::EnumError(val1), Error::EnumError(val2)) => val1 == val2,
            (Error::TupleError(val1), Error::TupleError(val2)) => val1 == val2,
            (Error::StringError(val1), Error::StringError(val2)) => sha256(val1) == sha256(val2),
            _ => false,
        }
    }
}
