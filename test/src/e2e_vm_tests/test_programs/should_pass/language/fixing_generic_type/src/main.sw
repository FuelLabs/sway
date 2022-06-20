script;

use std::mem::*;
use std::intrinsics::*;
use std::assert::assert;

struct TestStruct {
    boo: bool,
    uwu: u64
}

fn main() -> bool {
    // Create a struct
    let foo = TestStruct { boo: true, uwu: 42 };
    let foo_len = size_of::<TestStruct>();
    assert(foo_len == 16);

    // Create a clone of the struct
    let buf = ~Buffer::alloc(foo_len);
    buf.write(true, 0);
    buf.write(42, size_of::<bool>());
    // ^^ This parameter was declared as type bool, but argument of type u64 was provided.
    let foo: TestStruct = buf.into_unchecked();
    assert(foo.boo == true);
    assert(foo.uwu == 42);

    true
}
