script;

use std::{intrinsics::{size_of, size_of_val}};

struct TestStruct {
    boo: bool,
    uwu: u64,
}

fn main() -> raw_slice {
    // Create an empty slice
    let mut buf = raw_slice::new();
    assert(buf.ptr().is_null());
    assert(buf.len() == 0);

    // Grow it to 32 bytes
    buf.resize(32);
    assert(!buf.ptr().is_null());
    assert(buf.len() == 32);

    // Keep it the same size
    let old_ptr = buf.ptr();
    buf.resize(32);
    assert(buf.ptr() == old_ptr);
    assert(buf.len() == 32);

    // Truncate it to 16 bytes
    let old_ptr = buf.ptr();
    buf.resize(16);
    assert(buf.ptr() == old_ptr);
    assert(buf.len() == 16);

    // Allocate 32 bytes
    let mut buf = raw_slice::alloc(32);
    assert(!buf.ptr().is_null());
    assert(buf.len() == 32);

    // Truncate it to 0 bytes
    let old_ptr = buf.ptr();
    buf.resize(0);
    assert(buf.ptr() == old_ptr);
    assert(buf.len() == 0);

    // Grow it
    let old_ptr = buf.ptr();
    buf.grow();
    assert(buf.ptr() != old_ptr);
    assert(buf.len() == 1);

    // Grow it even more
    let old_ptr = buf.ptr();
    buf.grow();
    assert(buf.ptr() != old_ptr);
    assert(buf.len() == 2);

    // Grow it beyond comprehension
    let old_ptr = buf.ptr();
    buf.grow();
    buf.grow();
    assert(buf.ptr() != old_ptr);
    assert(buf.len() == 8);

    // Write a byte to it
    let old_ptr = buf.ptr();
    let old_len = buf.len();
    buf.write(1, 7);
    assert(buf.ptr() == old_ptr);
    assert(buf.len() == old_len);
    assert(buf.ptr().read_t::<u64>() == 1);
    
    // Read a byte from it
    let val = buf.read(7);
    assert(val == 1);

    // Read a u64 from it
    let val = buf.read_t::<u64>(0);
    assert(val == 1);

    // Write a u64 to it
    buf.write_t::<u64>(257, 0);
    assert(buf.read(7) == 1);
    assert(buf.read(6) == 1);
    assert(buf.read(5) == 0);

    // Create a struct
    let foo = TestStruct {
        boo: true,
        uwu: 42,
    };
    let foo_len = size_of_val(foo);
    assert(foo_len == 16);

    // Get a slice to it
    let foo_ptr = __addr_of(foo);
    let buf_len = foo_len / size_of::<u64>();
    let foo_buf = raw_slice::from_ptr_t::<u64>(foo_ptr, buf_len);
    assert(foo_buf.ptr() == foo_ptr);
    assert(foo_buf.len_t::<u64>() == 2);

    // Convert to a vector
    let foo_vec: Vec<u64> = Vec::<u64>::from(foo_buf);
    assert(foo_vec.len() == 2);

    // Return it
    foo_vec.as_raw_slice()
}
