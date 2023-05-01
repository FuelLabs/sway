script;

use std::{intrinsics::{size_of, size_of_val}};

fn addr_of<T>(val: T) -> __ptr[u64] {
    asm(r1: val) { r1: __ptr[u64] }
}
fn alloc<T>(count: u64) -> __ptr[u64] {
    asm(size: (size_of::<T>() * count) + 1, ptr) {
        aloc size;
        addi ptr hp i1;
        ptr: __ptr[u64]
    }
}

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
    assert(foo_ptr == asm(r1: foo) { r1: __ptr[u64] });

    // Get another pointer to it and compare
    let foo_ptr_2 = addr_of(foo);
    assert(foo_ptr_2 == foo_ptr);

    // Copy the struct into a buffer
    let buf_ptr = alloc::<u64>(2);
    foo_ptr.copy_to(buf_ptr, 2);
    assert(asm(r1: buf_ptr, r2: foo_ptr, r3: foo_len, res) {
        meq res r1 r2 r3;
        res: bool
    });

    // Read the pointer as a TestStruct
    let foo: TestStruct = buf_ptr.read();
    assert(foo.boo == true);
    assert(foo.uwu == 42);

    // Read fields of the struct
    let uwu_ptr = buf_ptr.add(1);
    let uwu: u64 = uwu_ptr.read();
    assert(uwu == 42);
    let boo_ptr = uwu_ptr.sub(1);
    let boo: bool = boo_ptr.read();
    assert(boo == true);

    // Write values into a buffer
    let buf_ptr = alloc::<u64>(2);
    // buf_ptr.write(true);
    // buf_ptr.add(1).write(42);
    let foo: TestStruct = buf_ptr.read();
    assert(foo.boo == true);
    assert(foo.uwu == 42);

    // Write structs into a buffer
    let buf_ptr = alloc::<u64>(4);
    buf_ptr.write(foo);
    buf_ptr.add(2).write(foo);
    let bar: ExtendedTestStruct = buf_ptr.read();
    assert(bar.boo == true);
    assert(bar.uwu == 42);
    assert(bar.kek == true);
    assert(bar.bur == 42);

    true
}
