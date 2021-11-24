contract;

use std::context::*;
use context_testing_abi::ContextTesting;

impl ContextTesting for Contract {


    fn get_id(gas: u64, coins: u64, color: b256, input: ()) -> b256 {
        this_id()
    }

    fn get_value(gas: u64, coins: u64, color: b256, input: ()) -> u64 {
        msg_value()
    }

    fn get_token_id(gas: u64, coins: u64, color: b256, input: ()) -> b256 {
        msg_token_id()
    }

    fn get_gas(gas: u64, coins: u64, color: b256, input: ()) -> u64 {
        gas()
    }

    fn get_global_gas(gas: u64, coins: u64, color: b256, input: ()) -> u64 {
        global_gas()
    }
}