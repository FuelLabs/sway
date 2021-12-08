contract;

use std::context::*;
use context_testing_abi::ContextTesting;

impl ContextTesting for Contract {


    fn get_id(gas: u64, coins: u64, color: b256, input: ()) -> b256 {
        contract_id()
    }

    fn get_this_balance(gas: u64, coins: u64, color: b256, token_id: b256) -> b256 {
        this_balance()
    }

    fn get_balance_of_contract(gas: u64, coins: u64, color: b256, token_id: b256, contract_id: ContractId) -> b256 {
        balance_of_contract()
    }

    fn get_amount(gas: u64, coins: u64, color: b256, input: ()) -> u64 {
        msg_amount()
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