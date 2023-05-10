contract;

use std::storage::storage_api::{read, write};

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
        write(STORAGE_KEY, 0, amount);
    }

    #[storage(read)]
    fn get_something() -> u64 {
        let value: Option<u64> = read::<u64>(STORAGE_KEY, 0);
        value.unwrap_or(0)
    }
}
