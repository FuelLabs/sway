library;

use core::ops::*;
use std::hash::*;

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

impl Hash for MyStruct {
    fn hash(self, ref mut state: Hasher) {
        self.x.hash(state);
        self.y.hash(state);
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

impl Hash for MyEnum {
    fn hash(self, ref mut state: Hasher) {
        match self {
            MyEnum::X(val) => val.hash(state),
            MyEnum::Y(val) => val.hash(state),
        }
    }
}

impl Eq for (u64, u64) {
    fn eq(self, other: Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl Hash for (u64, u64) {
    fn hash(self, ref mut state: Hasher) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

impl Eq for [u64; 3] {
    fn eq(self, other: Self) -> bool {
        self[0] == other[0] && self[1] == other[1] && self[2] == other[2]
    }
}

impl Hash for [u64; 3] {
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
    }
}

impl Eq for str[4] {
    fn eq(self, other: Self) -> bool {
        sha256_str(self) == sha256_str(other)
    }
}

impl Hash for str[4] {
    fn hash(self, ref mut state: Hasher) {
        state.write_str(self);
    }
}

fn sha256_str<T>(s: T) -> b256 {
    let mut hasher = Hasher::new();
    hasher.write_str(s);
    hasher.sha256()
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
            (Error::StringError(val1), Error::StringError(val2)) => sha256_str(val1) == sha256_str(val2),
            _ => false,
        }
    }
}

impl Hash for Error {
    fn hash(self, ref mut state: Hasher) {
        match self {
            Error::BoolError(val) => val.hash(state),
            Error::U8Error(val) => val.hash(state),
            Error::U16Error(val) => val.hash(state),
            Error::U32Error(val) => val.hash(state),
            Error::U64Error(val) => val.hash(state),
            Error::StructError(val) => val.hash(state),
            Error::EnumError(val) => val.hash(state),
            Error::TupleError(val) => val.hash(state),
            Error::ArrayError(val) => val.hash(state),
            Error::StringError(val) => state.write_str(val),
        }
    }
}
