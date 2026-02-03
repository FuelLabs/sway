library;

use std::intrinsics::*;

struct TestStruct {
    #[allow(dead_code)]
    field_1: bool,
     #[allow(dead_code)]
    field_2: u64,
}

fn is_ref_type<T>(_param: T) -> bool {
    is_reference_type::<T>()
}

#[cfg(experimental_str_array_no_padding = false)]
fn str_11_size() -> u64 {
    16
}

#[cfg(experimental_str_array_no_padding = true)]
fn str_11_size() -> u64 {
    11
}

#[test]
fn t() {
    let zero = b256::min();
    let a: u64 = 1;
    let b: u32 = 1;
    let c: u16 = 1;
    let d: u8 = 1;
    let e: b256 = zero;
    let f: str[11] = __to_str_array("Fuel rocks!");

    let test_array = [42u16; 3];

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

    __log(str_11_size());

    assert(size_of::<u64>() == 8);
    assert(size_of::<u32>() == 8);
    assert(size_of::<u16>() == 8);
    assert(size_of::<u8>() == 1);
    assert(size_of::<b256>() == 32);
    assert(size_of::<str[11]>() == str_11_size());
    assert(size_of::<[u16; 3]>() == 24);
    assert(size_of::<TestStruct>() == 16);

    assert(size_of_val(a) == 8);
    assert(size_of_val(b) == 8);
    assert(size_of_val(c) == 8);
    assert(size_of_val(d) == 1);
    assert(size_of_val(e) == 32);
    assert(size_of_val(f) == str_11_size());
}
