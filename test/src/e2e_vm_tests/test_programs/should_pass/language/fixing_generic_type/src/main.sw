script;

dep pointer;
dep buffer;

use buffer::*;

use std::assert::assert;

struct TestStruct {
    boo: bool,
    uwu: u64,
}

struct Data<T> {
    value: T,
}

impl<T> Data<T> {
    fn noop<F>(other: F) -> F {
        other
    }
}

fn main() -> bool {
    // Create a struct
    let foo = TestStruct {
        boo: true,
        uwu: 42,
    };
    let foo_len = __size_of::<TestStruct>();
    assert(foo_len == 16);

    // Create a clone of the struct
    let buf = Buffer::alloc(foo_len);
    buf.write(true, 0);
    buf.write(42, __size_of::<bool>());
    // ^^ This parameter was declared as type bool, but argument of type u64 was provided.
    let foo: TestStruct = buf.into_unchecked();
    assert(foo.boo == true);
    assert(foo.uwu == 42);

    let data = Data::<bool>::noop::<u64>(1u64);

    true
}
