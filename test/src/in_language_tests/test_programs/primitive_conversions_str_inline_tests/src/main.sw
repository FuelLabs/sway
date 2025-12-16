library;

use std::primitive_conversions::str::*;

#[test]
fn str_slice_to_str_array() {
    let a = "abcd";
    let b: str[4] = a.try_as_str_array().unwrap();
    assert(__size_of_str_array::<str[4]>() == a.len() && __size_of_val(b) == 4);

    let c = from_str_array(b);

    assert(a == c);
}
