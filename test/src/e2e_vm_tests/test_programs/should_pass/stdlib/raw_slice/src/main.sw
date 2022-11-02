script;

use std::{
    alloc::alloc,
    assert::assert,
    hash::sha256,
    intrinsics::{
        size_of,
        size_of_val,
    },
    logging::log,
    tx::{
        tx_script_data,
    },
};

struct TestStruct {
    boo: bool,
    uwu: u64,
}

fn main() -> raw_slice {
    // Create a struct
    let foo = TestStruct {
        boo: true,
        uwu: 42,
    };
    let foo_len = size_of_val(foo);
    assert(foo_len == 16);

    // Get a slice to it
    let foo_buf = raw_slice::from_raw_parts(__addr_of(foo), size_of_val(foo));
    assert(foo_buf.ptr() == asm(r1: foo) { r1: raw_ptr });
    assert(foo_buf.len::<u64>() == 2);

    // Get another slice to it and compare
    let foo_buf_2 = raw_slice::from_raw_parts(__addr_of(foo), size_of_val(foo));
    assert(foo_buf_2 == foo_buf);

    // Copy the struct into a buffer
    let buf = alloc::<TestStruct>(1);
    foo_buf.copy_to::<TestStruct>(buf);
    assert(asm(r1: buf.ptr(), r2: foo_buf.ptr(), r3: 16, res) {
        meq res r1 r2 r3;
        res: bool
    });

    // Return
    foo_buf
}
