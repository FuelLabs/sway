library;

use std::intrinsics::{size_of, size_of_val};

struct TestStruct {
    #[allow(dead_code)]
    boo: bool,
    #[allow(dead_code)]
    uwu: u64,
}

#[test]
fn raw_slice_len_zero() {
    let content_ptr = std::alloc::alloc_bytes(0);
    let slice = asm(ptr: (content_ptr, 0)) { ptr: raw_slice };

    assert_eq(slice.len::<u8>(), 0);
    assert_eq(slice.len::<u64>(), 0);
    assert_eq(slice.len::<TestStruct>(), 0);
}

#[test]
fn raw_slice_len() {
    let content_ptr = std::alloc::alloc_bytes(32);
    let slice = asm(ptr: (content_ptr, 32)) { ptr: raw_slice };

    assert_eq(slice.len::<u8>(), 32);
    assert_eq(slice.len::<u64>(), 4);
    assert_eq(slice.len::<TestStruct>(), 2);
}

#[test]
fn raw_slice_len_size_smaller_than_type_size() {
    let content_ptr = std::alloc::alloc_bytes(2);
    let slice = asm(ptr: (content_ptr, 2)) { ptr: raw_slice };

    assert_eq(slice.len::<u64>(), 0);
    assert_eq(slice.len::<TestStruct>(), 0);
}

#[test]
fn raw_slice_len_size_not_multiple_of_type_size() {
    let content_ptr = std::alloc::alloc_bytes(33);
    let slice = asm(ptr: (content_ptr, 33)) { ptr: raw_slice };

    assert_eq(slice.len::<u64>(), 4);
    assert_eq(slice.len::<TestStruct>(), 2);
}

#[test]
fn raw_slice_from_parts_and_to_vec() {
    let foo = TestStruct {
        boo: true,
        uwu: 42,
    };
    let foo_len = size_of_val(foo);
    assert_eq(foo_len, 16);

    // Get a slice to it.
    let foo_ptr = __addr_of(foo);
    let buf_len = foo_len / size_of::<u64>();
    let foo_buf = raw_slice::from_parts::<u64>(foo_ptr, buf_len);
    assert(foo_buf.ptr() == foo_ptr);
    assert_eq(foo_buf.len::<u64>(), 2);

    // Convert to a vector.
    let foo_vec: Vec<u64> = Vec::<u64>::from(foo_buf);
    assert_eq(foo_vec.len(), 2);

    // Convert back to a `raw_slice`.
    let foo_slice = foo_vec.as_raw_slice();
    assert_eq(foo_slice.len::<u64>(), 2);
}
