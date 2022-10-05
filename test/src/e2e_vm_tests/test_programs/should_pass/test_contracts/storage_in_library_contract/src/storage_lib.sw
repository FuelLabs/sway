library storage_lib;

storage {
    foo: u64 = 42,
}

#[storage(write)]
pub fn mutate_foo() {
    storage.foo = 69;
}
