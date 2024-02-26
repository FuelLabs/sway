contract;

use std::storage::storage_api::{read, write};

// ANCHOR: storage_namespace
#[namespace(example_namespace)]
storage {
    // ANCHOR_END: storage_namespace
    foo: u64 = 0,
}

abi StorageNamespaceExample {
    #[storage(write)]
    fn store_something(amount: u64);

    #[storage(read)]
    fn get_something() -> u64;
}

impl StorageNamespaceExample for Contract {
    #[storage(write)]
    fn store_something(amount: u64) {
        storage.foo.write(amount);
    }

    #[storage(read)]
    fn get_something() -> u64 {
        storage.foo.try_read().unwrap_or(0)
    }
}
