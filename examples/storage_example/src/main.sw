contract;

use std::storage::{get, store};

abi StorageExample {
    #[storage(write)]
    fn store_something(amount: u64);
    #[storage(read)]
    fn get_something() -> u64;
}

const STORAGE_KEY: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;

impl StorageExample for Contract {
    #[storage(write)]
    fn store_something(amount: u64) {
        store(STORAGE_KEY, amount);
    }

    #[storage(read)]
    fn get_something() -> u64 {
        let value = get::<u64>(STORAGE_KEY);
        value
    }
}
