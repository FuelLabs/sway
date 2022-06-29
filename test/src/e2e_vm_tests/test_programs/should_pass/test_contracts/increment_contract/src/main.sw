contract;

use increment_abi::Incrementor;
use std::{constants::ZERO_B256, storage::{get, store}};

impl Incrementor for Contract {
    #[storage(write)]
    fn initialize(initial_value: u64) -> u64 {
        store(ZERO_B256, initial_value);
        initial_value
    }
    #[storage(read, write)]
    fn increment(increment_by: u64) -> u64 {
        let new_val = get::<u64>(ZERO_B256) + increment_by;
        // check that monomorphization doesn't overwrite the type of the above
        let dummy = get::<u32>(ZERO_B256) + increment_by;
        store(ZERO_B256, new_val);
        new_val
    }
    #[storage(read)]
    fn get() -> u64 {
        get::<u64>(ZERO_B256)
    }
}
