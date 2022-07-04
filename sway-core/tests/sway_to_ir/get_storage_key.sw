contract;

struct Empty {}

impl Empty {
    fn bar(self) -> b256 {
        __get_storage_key()
    }
}

storage {
    e1: Empty,
    e2: Empty,
}

abi GetStorageKeyTest {
    fn foo1() -> b256;
    fn foo2() -> b256;
}

impl GetStorageKeyTest for Contract {
    fn foo1() -> b256 {
        storage.e1.bar()
    }
    fn foo2() -> b256 {
        storage.e2.bar()
    }
}