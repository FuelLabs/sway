contract;
use std::context::Context;
use std::context::Msg;
use context_testing_abi::ContextTesting;


impl ContextTesting for Contract {
    let context: Context = ~Context::new();
    let msg: Msg = ~Msg::new();

    fn returns_id(gas: u64, coins: u64, color: b256, input: ()) -> b256 {
        context::id()
    }

    fn returns_value(gas: u64, coins: u64, color: b256, input: ()) -> u64 {
        msg::value()
    }

    fn returns_token_id(gas: u64, coins: u64, color: b256, input: ()) -> b256 {
        msg::token_id();
    }
}