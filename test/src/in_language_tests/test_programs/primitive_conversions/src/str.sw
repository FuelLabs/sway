library;

use std::primitive_conversions::str::*;

#[cfg(experimental_str_array_no_padding = false)]
#[test]
fn str_slice_to_str_array() {
    let a = "abcd";
    let b: str[4] = a.try_as_str_array().unwrap();
    assert_eq(__size_of_str_array::<str[4]>(), a.len());
    assert_eq(__size_of_val(b), 8);

    let c = from_str_array(b);

    assert_eq(a, c);
}

#[cfg(experimental_str_array_no_padding = true)]
#[test]
fn str_slice_to_str_array() {
    let a = "abcd";
    let b: str[4] = a.try_as_str_array().unwrap();
    assert_eq(__size_of_str_array::<str[4]>(), a.len());
    assert_eq(__size_of_val(b), a.len());

    let c = from_str_array(b);

    assert_eq(a, c);
}
