script;

use std::{alloc::alloc, hash::*, intrinsics::{size_of, size_of_val}};

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
    let foo_ptr = __addr_of(foo);
    assert(foo_ptr == asm(r1: foo) { r1: raw_ptr });

    // Get another pointer to it and compare
    let foo_ptr_2 = __addr_of(foo);
    assert(foo_ptr_2 == foo_ptr);

    // Copy the struct into a buffer
    let buf_ptr = alloc::<u64>(2);
    foo_ptr.copy_to::<u64>(buf_ptr, 2);
    assert(asm(r1: buf_ptr, r2: foo_ptr, r3: foo_len, res) {
        meq res r1 r2 r3;
        res: bool
    });

    // Read the pointer as a TestStruct
    let foo: TestStruct = buf_ptr.read();
    assert(foo.boo == true);
    assert(foo.uwu == 42);

    // Read fields of the struct
    let uwu_ptr = buf_ptr.add_uint_offset(8); // struct fields are aligned to words
    let uwu: u64 = uwu_ptr.read();
    assert(uwu == 42);
    let boo_ptr = uwu_ptr.sub_uint_offset(8);
    let boo: bool = boo_ptr.read();
    assert(boo == true);

    // Write values into a buffer
    let buf_ptr = alloc::<u64>(2);
    buf_ptr.write(true);
    buf_ptr.add_uint_offset(8).write(42);
    let foo: TestStruct = buf_ptr.read();
    assert(foo.boo == true);
    assert(foo.uwu == 42);

    // Write structs into a buffer
    let buf_ptr = alloc::<u64>(4);
    buf_ptr.write(foo);
    buf_ptr.add::<TestStruct>(1).write(foo);
    let bar: ExtendedTestStruct = buf_ptr.read();
    assert(bar.boo == true);
    assert(bar.uwu == 42);
    assert(bar.kek == true);
    assert(bar.bur == 42);

    // Make sure that reading a memory location into a variable and then
    // overriding the same memory location does not change the variable read.
    let buf_ptr = alloc::<u64>(1);
    let small_string_1 = __to_str_array("fuel");
    let small_string_2 = __to_str_array("labs");
    buf_ptr.write(small_string_1);
    let read_small_string_1 = buf_ptr.read::<str[4]>();
    buf_ptr.write(small_string_2);
    let read_small_string_2 = buf_ptr.read::<str[4]>();
    assert(sha256_str_array(small_string_1) == sha256_str_array(read_small_string_1));
    assert(sha256_str_array(small_string_2) == sha256_str_array(read_small_string_2));

    let buf_ptr = alloc::<u64>(2);
    let large_string_1 = __to_str_array("fuelfuelfuel");
    let large_string_2 = __to_str_array("labslabslabs");
    buf_ptr.write(large_string_1);
    let read_large_string_1 = buf_ptr.read::<str[12]>();
    buf_ptr.write(large_string_2);
    let read_large_string_2 = buf_ptr.read::<str[12]>();
    assert(sha256_str_array(large_string_1) == sha256_str_array(read_large_string_1));
    assert(sha256_str_array(large_string_2) == sha256_str_array(read_large_string_2));

    true
}
