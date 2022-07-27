script;

use std::{alloc::alloc, assert::assert, hash::sha256, intrinsics::{size_of, size_of_val}, mem::*};

fn main() -> bool {

    // Write values into a buffer
    let buf_ptr = alloc(16);
    write(buf_ptr, true);

    true
}
