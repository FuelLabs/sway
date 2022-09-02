script;

use std::{alloc::alloc, assert::assert, hash::sha256, intrinsics::{size_of, size_of_val}, mem::*};

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
        r1: raw_ptr
    });

    // Get another pointer to it and compare
    let foo_ptr_2 = addr_of(foo);
    assert(foo_ptr_2 == foo_ptr);

    // Copy the struct into a buffer
    let buf_ptr = alloc(16);
    copy(foo_ptr, buf_ptr, 16);
    assert(eq(buf_ptr, foo_ptr, 16));
    assert(asm(r1: buf_ptr, r2: foo_ptr, r3: foo_len) {
        meq r1 r1 r2 r3;
        r1: bool
    });

    // Read the pointer as a TestStruct
    let foo: TestStruct = read(buf_ptr);
    assert(foo.boo == true);
    assert(foo.uwu == 42);

    // Read fields of the struct
    let uwu_ptr = ptr_offset(buf_ptr, size_of::<bool>());
    let uwu: u64 = read(uwu_ptr);
    assert(uwu == 42);
    let boo_ptr = buf_ptr;
    let boo: bool = read(boo_ptr);
    assert(boo == true);

    // Write values into a buffer
    let buf_ptr = alloc(16);
    write(buf_ptr, true);
    write(ptr_offset(buf_ptr, size_of::<bool>()), 42);
    let foo: TestStruct = read(buf_ptr);
    assert(foo.boo == true);
    assert(foo.uwu == 42);

    // Write structs into a buffer
    let buf_ptr = alloc(32);
    write(buf_ptr, foo);
    write(ptr_offset(buf_ptr, size_of::<TestStruct>()), foo);
    let bar: ExtendedTestStruct = read(buf_ptr);
    assert(bar.boo == true);
    assert(bar.uwu == 42);
    assert(bar.kek == true);
    assert(bar.bur == 42);

    // Make sure that reading a memory location into a variable and then
    // overriding the same memory location does not change the variable read.
    let buf_ptr = alloc(8);
    let small_string_1 = "fuel";
    let small_string_2 = "labs";
    write(buf_ptr, small_string_1);
    let read_small_string_1 = read::<str[4]>(buf_ptr);
    write(buf_ptr, small_string_2);
    let read_small_string_2 = read::<str[4]>(buf_ptr);
    assert(sha256(small_string_1) == sha256(read_small_string_1));
    assert(sha256(small_string_2) == sha256(read_small_string_2));

    let buf_ptr = alloc(16);
    let large_string_1 = "fuelfuelfuel";
    let large_string_2 = "labslabslabs";
    write(buf_ptr, large_string_1);
    let read_large_string_1 = read::<str[12]>(buf_ptr);
    write(buf_ptr, large_string_2);
    let read_large_string_2 = read::<str[12]>(buf_ptr);
    assert(sha256(large_string_1) == sha256(read_large_string_1));
    assert(sha256(large_string_2) == sha256(read_large_string_2));

    true
}
