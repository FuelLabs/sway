script;

use std::intrinsics::*;
use std::assert::assert;
use std::constants::ZERO;

struct TestStruct {
    field_1: bool,
    field_2: u64,
}

fn main() -> bool {
    let a: u64 = 1;
    let b: u32 = 1;
    let c: u16 = 1;
    let d: u8 = 1;
    let e: b256 = ZERO;
    let f: str[11] = "Fuel rocks!";

    let test_array = [42u16;
    3];

    let test_struct = TestStruct {
        field_1: false,
        field_2: 11,
    };

    assert(!is_reference_type(42u64));
    assert(!is_reference_type(42u32));
    assert(!is_reference_type(42u16));
    assert(!is_reference_type(11u8));
    assert(is_reference_type(test_array));
    assert(is_reference_type(test_struct));
    assert(is_reference_type((true, 11, ZERO, 255u8)));
    assert(is_reference_type(e));
    assert(is_reference_type(f));

    assert(size_of(a) == 8);
    assert(size_of(b) == 8);
    assert(size_of(c) == 8);
    assert(size_of(d) == 8);
    assert(size_of(e) == 32);
    assert(size_of(f) == 16);

    true
}
