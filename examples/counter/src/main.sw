contract;

use std::storage::{get, store};

const KEY = 0x0000000000000000000000000000000000000000000000000000000000000000;

abi Incrementor {
    fn initialize(initial_value: u64) -> u64;
    fn increment(initial_value: u64) -> u64;
}

impl Incrementor for Contract {
    fn initialize(initial_value: u64) -> u64 {
        store(KEY, initial_value);
        initial_value
    }
    fn increment(increment_by: u64) -> u64 {
        let new_val = get::<u64>(KEY) + increment_by;
        store(KEY, new_val);
        new_val
    }
}
