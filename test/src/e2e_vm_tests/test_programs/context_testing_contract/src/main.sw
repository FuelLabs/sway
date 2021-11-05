contract;
use std::context::Context;
use context_testing_abi::ContextTesting;


impl ContextTesting for Contract {
    fn returns_id(gas: u64, coins: u64, color: b256, input: ()) -> b256 {
        let context: Context = ~Context::new();
        context.id()
    }
}