library storage_lib;

storage {
    foo: u64 = 0,
}

#[storage(write)]
pub fn update_library_storage() {
    storage.foo = 69;
}

#[storage(read)]
pub fn get_library_storage() -> u64 {
    storage.foo
}
