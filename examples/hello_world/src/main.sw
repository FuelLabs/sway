contract;

use std::constants::*;
use std::storage::*;

abi TestContract {
    fn initialize_counter(value: u64) -> u64;
    fn increment_counter(amount: u64) -> u64;
}

const SLOT = 0x0000000000000000000000000000000000000000000000000000000000000000;

impl TestContract for Contract {
    fn initialize_counter(value: u64) -> u64 {
        store(SLOT, value);
        value
    }

    fn increment_counter(amount: u64) -> u64 {
        let storedVal: u64 = get(SLOT);
        let value = storedVal + amount;
        store(SLOT, value);
        value
    }
}
