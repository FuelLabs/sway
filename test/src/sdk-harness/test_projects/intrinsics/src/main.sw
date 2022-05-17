script;

use std::intrinsics::*;
use std::assert::assert;
use std::constants::ZERO;


struct TestStruct {
    field_1: bool,
    field_2: u64,
}

fn main() -> bool {
    let test_array = [42u16; 3];
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

    true
}
