library tests;

dep data_structures;

use data_structures::*;

/////////////////////////////////////////////////////////////////////////////
// Generic Tests
/////////////////////////////////////////////////////////////////////////////
fn test_is_ok<T>(val: T) {
    assert(Result::Ok(val).is_ok());
    assert(!Result::Err(val).is_ok());
}

fn test_is_err<T>(val: T) {
    assert(!Result::Ok(val).is_err());
    assert(Result::Err(val).is_err());
}

fn test_unwrap<T>(val: T)
where
    T: Eq
{
    assert(Result::Ok(val).unwrap() == val);
}

fn test_unwrap_or<T>(val: T, default: T)
where
    T: Eq
{
    assert(Result::Ok(val).unwrap_or(default) == val);
    assert(Result::Err(val).unwrap_or(default) == default);
}

/////////////////////////////////////////////////////////////////////////////
// Tests for Various Types
/////////////////////////////////////////////////////////////////////////////
pub fn test_bool() {
    test_is_ok(true);
    test_is_err(true);
    test_unwrap(true);
    test_unwrap_or(true, false);
}

pub fn test_u8() {
    test_is_ok(42_u8);
    test_is_err(42_u8);
    test_unwrap(42_u8);
    test_unwrap_or(42_u8, 69_u8);
}

pub fn test_u16() {
    test_is_ok(42_u16);
    test_is_err(42_u16);
    test_unwrap(42_u16);
    test_unwrap_or(42_u16, 69_u16);
}

pub fn test_u32() {
    test_is_ok(42_u32);
    test_is_err(42_u32);
    test_unwrap(42_u32);
    test_unwrap_or(42_u32, 69_u32);
}

pub fn test_u64() {
    test_is_ok(42_u64);
    test_is_err(42_u64);
    test_unwrap(42_u64);
    test_unwrap_or(42_u64, 69_u64);
}

pub fn test_struct() {
    let s = MyStruct { x: 42, y: 43 };
    test_is_ok(s);
    test_is_err(s);
    test_unwrap(s);
    test_unwrap_or(s, MyStruct { x: 69, y: 69 });
}

pub fn test_enum() {
    let e = MyEnum::Y(42);
    test_is_ok(e);
    test_is_err(e);
    test_unwrap(e);
    test_unwrap_or(e, MyEnum::X(69));
}

pub fn test_tuple() {
    let t = (42, 43);
    test_is_ok(t);
    test_is_err(t);
    test_unwrap(t);
    test_unwrap_or(t, (69, 70));
}

pub fn test_array() {
    let a = [42, 43, 44];
    test_is_ok(a);
    test_is_err(a);
    test_unwrap(a);
    test_unwrap_or(a, [69, 70, 71]);
}

pub fn test_string() {
    let s = "fuel";
    test_is_ok(s);
    test_is_err(s);
    test_unwrap(s);
    test_unwrap_or(s, "sway");
}