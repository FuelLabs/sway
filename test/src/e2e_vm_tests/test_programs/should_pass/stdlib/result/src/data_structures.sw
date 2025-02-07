library;

use core::ops::*;
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

#[cfg(experimental_partial_eq = false)]
impl Eq for MyStruct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
#[cfg(experimental_partial_eq = true)]
impl PartialEq for MyStruct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}
#[cfg(experimental_partial_eq = true)]
impl Eq for MyStruct {}

#[cfg(experimental_partial_eq = false)]
impl Eq for MyEnum {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (MyEnum::X(val1), MyEnum::X(val2)) => val1 == val2,
            (MyEnum::Y(val1), MyEnum::Y(val2)) => val1 == val2,
            _ => false,
        }
    }
}
#[cfg(experimental_partial_eq = true)]
impl PartialEq for MyEnum {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (MyEnum::X(val1), MyEnum::X(val2)) => val1 == val2,
            (MyEnum::Y(val1), MyEnum::Y(val2)) => val1 == val2,
            _ => false,
        }
    }
}
#[cfg(experimental_partial_eq = true)]
impl Eq for MyEnum {}

#[cfg(experimental_partial_eq = false)]
impl Eq for (u64, u64) {
    fn eq(self, other: Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}
#[cfg(experimental_partial_eq = true)]
impl PartialEq for (u64, u64) {
    fn eq(self, other: Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}
#[cfg(experimental_partial_eq = true)]
impl Eq for (u64, u64) {}

#[cfg(experimental_partial_eq = false)]
impl Eq for [u64; 3] {
    fn eq(self, other: Self) -> bool {
        self[0] == other[0] && self[1] == other[1] && self[2] == other[2]
    }
}
#[cfg(experimental_partial_eq = true)]
impl PartialEq for [u64; 3] {
    fn eq(self, other: Self) -> bool {
        self[0] == other[0] && self[1] == other[1] && self[2] == other[2]
    }
}
#[cfg(experimental_partial_eq = true)]
impl Eq for [u64; 3] {}

#[cfg(experimental_partial_eq = false)]
impl Eq for str[4] {
    fn eq(self, other: Self) -> bool {
        sha256_str_array(self) == sha256_str_array(other)
    }
}
#[cfg(experimental_partial_eq = true)]
impl PartialEq for str[4] {
    fn eq(self, other: Self) -> bool {
        sha256_str_array(self) == sha256_str_array(other)
    }
}
#[cfg(experimental_partial_eq = true)]
impl Eq for str[4] {}
