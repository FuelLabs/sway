contract;
// use std::*;
use std::context::Context;
use std::context::Msg;
use context_testing_abi::ContextTesting;

impl ContextTesting for Contract {


    fn get_id(gas: u64, coins: u64, color: b256, input: ()) -> b256 {
        let context: Context = ~Context::new();
        context::id()
    }

    fn get_value(gas: u64, coins: u64, color: b256, input: ()) -> u64 {
        let msg: Msg = ~Msg::new();
        msg::value()
    }

    fn get_token_id(gas: u64, coins: u64, color: b256, input: ()) -> b256 {
        let msg: Msg = ~Msg::new();
        msg::token_id()
    }
}