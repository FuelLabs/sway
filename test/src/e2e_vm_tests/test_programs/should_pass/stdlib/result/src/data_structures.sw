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

fn sha256_str4(value: str[4]) -> b256 {
    let mut hasher = Hasher::new();
    hasher.write_str(value);
    hasher.sha256()
}

impl Eq for str[4] {
    fn eq(self, other: Self) -> bool {
        sha256_str4(self) == sha256_str4(other)
    }
}
