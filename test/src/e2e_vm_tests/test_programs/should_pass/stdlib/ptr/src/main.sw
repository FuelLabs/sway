script;

use std::ptr::*;
use std::alloc::*;
use std::intrinsics::*;
use std::assert::assert;

// We use this to workaround some generics issues
// See: https://github.com/FuelLabs/sway/issues/1628
fn fix<T, U>(v: U) -> T {
    asm(r1: v) {
        r1: T
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
    let foo_len = size_of::<TestStruct>();
    assert(foo_len == 16);

    // Get a pointer to it
    let foo_ptr = ~RawPointer::from(foo);
    assert(foo_ptr.addr == asm(r1: foo) {
        r1: u64
    });

    // Get another pointer to it and compare
    let foo_ptr_2 = ~RawPointer::from(foo);
    assert(foo_ptr_2 == foo_ptr);

    // Copy the struct into a buffer (copy_from)
    let buf_ptr = ~RawPointer::new(alloc(16));
    buf_ptr.copy_from(foo_ptr, foo_len);
    assert(asm(r1: buf_ptr.addr, r2: foo_ptr.addr, r3: foo_len) {
        meq r1 r1 r2 r3;
        r1: bool
    });

    // Copy the struct into a buffer (copy_to)
    let buf_ptr = ~RawPointer::new(alloc(16));
    foo_ptr.copy_to(buf_ptr, foo_len);
    assert(asm(r1: buf_ptr.addr, r2: foo_ptr.addr, r3: foo_len) {
        meq r1 r1 r2 r3;
        r1: bool
    });

    // Read the pointer as a TestStruct
    let foo: TestStruct = buf_ptr.read();
    assert(foo.boo == true);
    assert(foo.uwu == 42);

    // Read fields of the struct
    let uwu_ptr = buf_ptr.add(size_of::<bool>());
    let uwu: u64 = uwu_ptr.read_u64();
    assert(uwu == 42);
    let boo_ptr = uwu_ptr.sub(size_of::<bool>());
    let boo: bool = boo_ptr.read_bool();
    assert(boo == true);

    // Write values into a buffer
    let buf_ptr = ~RawPointer::new(alloc(16));
    buf_ptr.write_bool(true);
    buf_ptr.add(size_of::<bool>()).write_u64(42);
    let foo: TestStruct = buf_ptr.read();
    assert(foo.boo == true);
    assert(foo.uwu == 42);

    // Write structs into a buffer
    let buf_ptr = ~RawPointer::new(alloc(32));
    buf_ptr.write(foo);
    buf_ptr.add(size_of::<TestStruct>()).write(foo);
    let bar: ExtendedTestStruct = fix::<ExtendedTestStruct, TestStruct>(buf_ptr.read());
    assert(bar.boo == true);
    assert(bar.uwu == 42);
    assert(bar.kek == true);
    assert(bar.bur == 42);

    true
}
