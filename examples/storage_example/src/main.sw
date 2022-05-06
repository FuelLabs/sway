contract;

use std::storage::{get, store};

abi StorageExample {
    fn store_something(amount: u64);
    fn get_something() -> u64;
}

const STORAGE_KEY: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

impl StorageExample for Contract {
    fn store_something(amount: u64) {
        store(STORAGE_KEY, amount);
    }

    fn get_something() -> u64 {
        let value = get::<u64>(STORAGE_KEY);
        value
    }
}
