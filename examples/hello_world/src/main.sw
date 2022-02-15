contract;

use std::storage::*;
use std::constants::*;

abi TestContract {
    fn initialize_counter(gas_: u64, amount_: u64, coin_: b256, value: u64) -> u64;
    fn increment_counter(gas_: u64, amount_: u64, coin_: b256, amount: u64) -> u64;
}

const SLOT = 0x0000000000000000000000000000000000000000000000000000000000000000;

impl TestContract for Contract {
    fn initialize_counter(gas_: u64, amount_: u64, color_: b256, value: u64) -> u64 {
        store(SLOT, value);
        value
    }

    fn increment_counter(gas_: u64, amount_: u64, color_: b256, amount: u64) -> u64 {
        let storedVal: u64 = get(SLOT);
        let value = storedVal + amount;
        store(SLOT, value);
        value
    }
}
