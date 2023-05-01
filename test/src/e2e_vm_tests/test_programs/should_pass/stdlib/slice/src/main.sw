script;

use std::{intrinsics::{size_of, size_of_val}};

fn addr_of<T>(val: T) -> __ptr[u64] {
    asm(r1: val) { r1: __ptr[u64] }
}

struct TestStruct {
    boo: bool,
    uwu: u64,
}

fn main() -> __slice[T] {
    // Create a struct
    let foo = TestStruct {
        boo: true,
        uwu: 42,
    };
    let foo_len = size_of_val(foo);
    assert(foo_len == 16);

    // Get a slice to it
    let foo_ptr = addr_of(foo);
    let buf_len = foo_len / size_of::<u64>();
    let foo_buf = __slice[T]::from_parts(foo_ptr, buf_len);
    assert(foo_buf.ptr() == foo_ptr);
    assert(foo_buf.len() == 2);

    // Convert to a vector
    let foo_vec: Vec<u64> = Vec::<u64>::from(foo_buf);
    assert(foo_vec.len() == 2);

    // Return it
    foo_vec.as_slice()
}
