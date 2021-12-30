contract;

use std::context::*;
use balance_test_abi::BalanceTest;

impl BalanceTest for Contract {
    fn get_42(gas: u64, coins: u64, asset_id: b256, input: ()) -> u64 {
        42
    }
}
