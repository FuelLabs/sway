script;

use core::ops::*;
use std::hash::sha256;

struct MyStruct {
    x: u64,
    y: u64,
}

enum MyEnum {
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

enum Error {
    BoolError: bool,
    U8Error: u8,
    U16Error: u16,
    U32Error: u32,
    U64Error: u64,
    StructError: MyStruct,
    EnumError: MyEnum,
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
            _ => false,
        }
    }
}

fn main() -> bool {
    /* Test `bool` */
    test_is_some(true);
    test_is_none(true);
    test_unwrap(true);
    test_unwrap_or(true, false);
    test_some_ok_or(true, Error::BoolError(true));
    test_none_ok_or(true, Error::BoolError(true));
    
    /* Test `u8` */
    test_is_some(42u8);
    test_is_none(42u8);
    test_unwrap(42u8);
    test_unwrap_or(42u8, 1u8);
    test_some_ok_or(42u8, Error::U8Error(69u8));
    test_none_ok_or(42u8, Error::U8Error(69u8));
    
    /* Test `u16` */
    test_is_some(42u16);
    test_is_none(42u16);
    test_unwrap(42u16);
    test_unwrap_or(42u16, 1u16);
    test_some_ok_or(42u16, Error::U16Error(69u16));
    test_none_ok_or(42u16, Error::U16Error(69u16));
    
    /* Test `u32` */
    test_is_some(42u32);
    test_is_none(42u32);
    test_unwrap(42u32);
    test_unwrap_or(42u32, 1u32);
    test_some_ok_or(42u32, Error::U32Error(69u32));
    test_none_ok_or(42u32, Error::U32Error(69u32));

    /* Test `u32` */
    test_is_some(42u64);
    test_is_none(42u64);
    test_unwrap(42u64);
    test_unwrap_or(42u64, 1u64);
    test_some_ok_or(42u64, Error::U64Error(69u64));
    test_none_ok_or(42u64, Error::U64Error(69u64));
   
    let s = MyStruct {x : 42, y: 43 };
    test_is_some(s);
    test_is_none(s);
    test_unwrap(s);
    test_unwrap_or(s, MyStruct {x: 1, y: 1});
    test_some_ok_or(s, Error::StructError(MyStruct { x: 69, y: 70}));
    test_none_ok_or(s, Error::StructError(MyStruct { x: 69, y: 70}));

    let e = MyEnum::Y(42);
    test_is_some(e);
    test_is_none(e);
    test_unwrap(e);
    test_unwrap_or(e, MyEnum::X(1));
    test_some_ok_or(e, Error::EnumError(MyEnum::X(69)));
    test_none_ok_or(e, Error::EnumError(MyEnum::X(69)));

    true
}

fn test_is_some<T>(val: T) {
    assert(Option::Some(val).is_some());
    assert(!Option::None.is_some());
}

fn test_is_none<T>(val: T) {
    assert(!Option::Some(val).is_none());
    assert(Option::None.is_none());
}

fn test_unwrap<T>(val: T)
where
    T: Eq
{
    assert(Option::Some(val).unwrap() == val);
}

fn test_unwrap_or<T>(val: T, default: T)
where
    T: Eq
{
    assert(sha256(Option::Some(val).unwrap_or(default)) == sha256(val));
    assert(sha256(Option::None::<T>().unwrap_or(default)) == sha256(default));
}

/* Currently not able to combine the two functions below due to
 * https://github.com/FuelLabs/sway/issues/3325 */
fn test_some_ok_or<T, E>(val: T, default: E)
where
    T: Eq
{
    match Option::Some(val).ok_or(default) {
        Result::Ok(inner) => assert(val == inner),
        Result::Err(_) => revert(0),
    }
}
fn test_none_ok_or<T, E>(_val: T, default: E)
where
    E: Eq
{
    match Option::None::<T>().ok_or(default) {
        Result::Ok(_) => revert(0),
        Result::Err(e) => assert(sha256(default) == sha256(e)),
    }
}
