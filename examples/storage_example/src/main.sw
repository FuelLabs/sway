contract;

use std::{constants::ZERO_B256, storage::{get, store}};

abi StorageExample {
    #[storage(write)]fn store_something(amount: u64);
    #[storage(read)]fn get_something() -> u64;
}

impl StorageExample for Contract {
    #[storage(write)]fn store_something(amount: u64) {
        store(ZERO_B256, amount);
    }

    #[storage(read)]fn get_something() -> u64 {
        let value = get::<u64>(ZERO_B256);
        value
    }
}
