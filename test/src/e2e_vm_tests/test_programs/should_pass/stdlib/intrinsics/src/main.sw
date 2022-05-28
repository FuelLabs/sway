script;

use std::intrinsics::*;
use std::assert::assert;
use core::num::*;

struct TestStruct {
    field_1: bool,
    field_2: u64,
}
fn is_ref_type<T>(param: T) -> bool {
    is_reference_type::<T>()
}

fn get_size_of<T>(param: T) -> u64 {
    size_of::<T>()
}

fn main() -> bool {
    let zero = ~b256::min();
    let a: u64 = 1;
    let b: u32 = 1;
    let c: u16 = 1;
    let d: u8 = 1;
    let e: b256 = zero;
    let f: str[11] = "Fuel rocks!";

    let test_array = [42u16;
    3];

    let test_struct = TestStruct {
        field_1: false,
        field_2: 11,
    };

    assert(!is_ref_type(42u64));
    assert(!is_ref_type(42u32));
    assert(!is_ref_type(42u16));
    assert(!is_ref_type(11u8));
    assert(is_ref_type(test_array));
    assert(is_ref_type(test_struct));
    assert(is_ref_type((true, 11, zero, 255u8)));
    assert(is_ref_type(e));
    assert(is_ref_type(f));

    assert(get_size_of(a) == 8);
    assert(get_size_of(b) == 8);
    assert(get_size_of(c) == 8);
    assert(get_size_of(d) == 8);
    assert(get_size_of(e) == 32);
    assert(get_size_of(f) == 16);

    true
}
