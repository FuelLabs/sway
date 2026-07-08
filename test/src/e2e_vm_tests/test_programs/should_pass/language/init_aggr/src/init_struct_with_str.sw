//! Initialization of structs containing string arrays (`str[N]`) and string
//! slices (`str`). These fields are non-scalar and must be correctly copied
//! by the `init_aggr` lowering.
library;

use ::types::*;

struct StructWithStr {
    f_u64: u64,
    f_str_arr: str[5],
    f_str: str,
}

#[test]
fn test_all_empty() {
    all_empty();
}

#[inline(never)]
pub fn all_empty() {
    let s = StructWithStr {
        f_u64: 0,
        f_str_arr: __to_str_array("     "),
        f_str: " ",
    };

    assert_eq(s.f_u64, 0);
    assert_eq(s.f_str_arr, __to_str_array("     "));
    assert_eq(s.f_str, " ");
}

#[test]
fn test_non_empty() {
    non_empty();
}

#[inline(never)]
pub fn non_empty() {
    let s = StructWithStr {
        f_u64: 42,
        f_str_arr: __to_str_array("hello"),
        f_str: "world",
    };

    assert_eq(s.f_u64, 42);
    assert_eq(s.f_str_arr, __to_str_array("hello"));
    assert_eq(s.f_str, "world");
}
