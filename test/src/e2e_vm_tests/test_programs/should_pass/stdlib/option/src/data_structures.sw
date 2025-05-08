library;

use std::ops::*;
use std::hash::*;

/////////////////////////////////////////////////////////////////////////////
// Data Structures Used in the Tests
/////////////////////////////////////////////////////////////////////////////
pub struct MyStruct {
    pub x: u64,
    pub y: u64,
}

pub enum MyEnum {
    X: u64,
    Y: u64,
}

impl PartialEq for MyStruct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
impl Eq for MyStruct {}

impl Hash for MyStruct {
    fn hash(self, ref mut state: Hasher) {
        self.x.hash(state);
        self.y.hash(state);
    }
}

impl PartialEq for MyEnum {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (MyEnum::X(val1), MyEnum::X(val2)) => val1 == val2,
            (MyEnum::Y(val1), MyEnum::Y(val2)) => val1 == val2,
            _ => false,
        }
    }
}
impl Eq for MyEnum {}

impl Hash for MyEnum {
    fn hash(self, ref mut state: Hasher) {
        match self {
            MyEnum::X(val) => {
                0_u8.hash(state);
                val.hash(state);
            }
            MyEnum::Y(val) => {
                1_u8.hash(state);
                val.hash(state);
            }
        }
    }
}

impl PartialEq for [u64; 3] {
    fn eq(self, other: Self) -> bool {
        self[0] == other[0] && self[1] == other[1] && self[2] == other[2]
    }
}
impl Eq for [u64; 3] {}

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
    StringError: str,
}

impl PartialEq for Error {
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
impl Eq for Error {}

impl Hash for Error {
    fn hash(self, ref mut state: Hasher) {
        match self {
            Error::BoolError(val) => {
                0_u8.hash(state);
                val.hash(state);
            },
            Error::U8Error(val) => {
                1_u8.hash(state);
                val.hash(state);
            },
            Error::U16Error(val) => {
                2_u8.hash(state);
                val.hash(state);
            },
            Error::U32Error(val) => {
                3_u8.hash(state);
                val.hash(state);
            },
            Error::U64Error(val) => {
                4_u8.hash(state);
                val.hash(state);
            },
            Error::StructError(val) => {
                5_u8.hash(state);
                val.hash(state);
            },
            Error::EnumError(val) => {
                6_u8.hash(state);
                val.hash(state);
            },
            Error::TupleError(val) => {
                7_u8.hash(state);
                val.hash(state);
            },
            Error::ArrayError(val) => {
                8_u8.hash(state);
                val.hash(state);
            },
            Error::StringError(val) => {
                9_u8.hash(state);
                state.write_str(val);
            },
        }
    }
}
