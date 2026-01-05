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
