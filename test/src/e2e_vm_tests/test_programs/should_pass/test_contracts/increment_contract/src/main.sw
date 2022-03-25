contract;

use increment_abi::Incrementor;
use std::storage::{get, store};

const KEY = 0x0000000000000000000000000000000000000000000000000000000000000000;

impl Incrementor for Contract {
    fn initialize(initial_value: u64) -> u64 {
        store(KEY, initial_value);
        initial_value
    }
    fn increment(increment_by: u64) -> u64 {
        let new_val = get::<u64>(KEY) + increment_by;
        // check that monomorphization doesn't overwrite the type of the above
        let dummy = get::<u32>(KEY) + increment_by;
        store(KEY, new_val);
        new_val
    }
    fn get() -> u64 {
        get::<u64>(KEY)
    }
}
