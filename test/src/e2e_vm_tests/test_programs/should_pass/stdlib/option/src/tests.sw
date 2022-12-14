library tests;

dep data_structures;

use core::ops::*;
use data_structures::*;
use std::hash::sha256;

/* Currently need to occasionally use `sha256` to compare generic types because
 * the correct implementation of `eq` for a type is not always detected
 * correctly. See https://github.com/FuelLabs/sway/issues/3351 and
 * https://github.com/FuelLabs/sway/issues/3326 */
/////////////////////////////////////////////////////////////////////////////
// Generic Tests
/////////////////////////////////////////////////////////////////////////////
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

/////////////////////////////////////////////////////////////////////////////
// Tests for Various Types
/////////////////////////////////////////////////////////////////////////////
pub fn test_bool() {
    test_is_some(true);
    test_is_none(true);
    test_unwrap(true);
    test_unwrap_or(true, false);
    test_some_ok_or(true, Error::BoolError(true));
    test_none_ok_or(true, Error::BoolError(true));
}

pub fn test_u8() {
    test_is_some(42u8);
    test_is_none(42u8);
    test_unwrap(42u8);
    test_unwrap_or(42u8, 1u8);
    test_some_ok_or(42u8, Error::U8Error(69u8));
    test_none_ok_or(42u8, Error::U8Error(69u8));
}

pub fn test_u16() {
    test_is_some(42u16);
    test_is_none(42u16);
    test_unwrap(42u16);
    test_unwrap_or(42u16, 1u16);
    test_some_ok_or(42u16, Error::U16Error(69u16));
    test_none_ok_or(42u16, Error::U16Error(69u16));
}

pub fn test_u32() {
    test_is_some(42u32);
    test_is_none(42u32);
    test_unwrap(42u32);
    test_unwrap_or(42u32, 1u32);
    test_some_ok_or(42u32, Error::U32Error(69u32));
    test_none_ok_or(42u32, Error::U32Error(69u32));
}

pub fn test_u64() {
    test_is_some(42u64);
    test_is_none(42u64);
    test_unwrap(42u64);
    test_unwrap_or(42u64, 1u64);
    test_some_ok_or(42u64, Error::U64Error(69u64));
    test_none_ok_or(42u64, Error::U64Error(69u64));
}

pub fn test_struct() {
    let s = MyStruct { x: 42, y: 43 };
    test_is_some(s);
    test_is_none(s);
    test_unwrap(s);
    test_unwrap_or(s, MyStruct { x: 1, y: 1 });
    test_some_ok_or(s, Error::StructError(MyStruct { x: 69, y: 70 }));
    test_none_ok_or(s, Error::StructError(MyStruct { x: 69, y: 70 }));
}

pub fn test_enum() {
    let e = MyEnum::Y(42);
    test_is_some(e);
    test_is_none(e);
    test_unwrap(e);
    test_unwrap_or(e, MyEnum::X(1));
    test_some_ok_or(e, Error::EnumError(MyEnum::X(69)));
    test_none_ok_or(e, Error::EnumError(MyEnum::X(69)));
}

pub fn test_tuple() {
    let t = (42, 43);
    test_is_some(t);
    test_is_none(t);
    test_unwrap(t);
    test_unwrap_or(t, (1, 1));
    test_some_ok_or(t, Error::TupleError((69, 70)));
    test_none_ok_or(t, Error::TupleError((69, 70)));
}

pub fn test_array() {
    let a = [42, 43, 44];
    test_is_some(a);
    test_is_none(a);
    test_unwrap(a);
    test_unwrap_or(a, [1, 1, 1]);
    test_some_ok_or(a, Error::ArrayError([69, 70, 71]));
    test_none_ok_or(a, Error::ArrayError([69, 70, 71]));
}

pub fn test_string() {
    let s = "fuel";
    test_is_some(s);
    test_is_none(s);
    test_unwrap(s);
    test_unwrap_or(s, "0000");
    test_some_ok_or(s, Error::StringError("0000"));
    test_none_ok_or(s, Error::StringError("0000"));
}
