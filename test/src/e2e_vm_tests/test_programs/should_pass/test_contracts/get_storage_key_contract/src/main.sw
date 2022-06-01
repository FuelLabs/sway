contract;

use get_storage_key_abi::TestContract;

struct Foo {
}

impl Foo {
    fn foo() -> b256 {
        __get_storage_key()
    }
}

storage {
    x: u64,
    f1: Foo,
    f2: Foo,
    y: u64,
    f3: Foo,
    f4: Foo,
}

impl TestContract for Contract {
    fn from_f1() -> b256 {
        storage.f1.foo()
    }
    fn from_f2() -> b256 {
        storage.f2.foo()
    }
    fn from_f3() -> b256 {
        storage.f3.foo()
    }
    fn from_f4() -> b256 {
        storage.f4.foo()
    }
}
