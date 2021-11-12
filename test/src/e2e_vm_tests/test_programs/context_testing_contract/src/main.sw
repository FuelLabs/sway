contract;

use std::context::this_id;
use std::context::msg_value;
use std::context::msg_token_id;
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
}