script;

use std::mem::*;
use std::alloc::*;
use std::intrinsics::*;
use std::assert::assert;

struct TestStruct {
    boo: bool,
    uwu: u64,
}

struct ExtendedTestStruct {
    boo: bool,
    uwu: u64,
    kek: bool,
    bur: u64,
}

fn main() -> bool {
    // Create a struct
    let foo = TestStruct {
        boo: true,
        uwu: 42,
    };
    let foo_len = size_of_val(foo);
    assert(foo_len == 16);

    // Get a pointer to it
    let foo_ptr = addr_of(foo);
    assert(foo_ptr == asm(r1: foo) {
        r1: u64
    });

    // Get another pointer to it and compare
    let foo_ptr_2 = addr_of(foo);
    assert(foo_ptr_2 == foo_ptr);

    // Copy the struct into a buffer
    let buf_ptr = alloc(16);
    copy(buf_ptr, foo_ptr, 16);
    assert(eq(buf_ptr, foo_ptr, 16));
    assert(asm(r1: buf_ptr, r2: foo_ptr, r3: foo_len) {
        meq r1 r1 r2 r3;
        r1: bool
    });

    true
}
