contract;

use std::context::*;
use context_testing_abi::*;

impl ContextTesting for Contract {

    fn get_id(gas: u64, coins: u64, color: b256, input: ()) -> b256 {
        contract_id()
    }

    fn get_this_balance(gas: u64, coins: u64, color: b256, token_id: b256) -> u64 {
        this_balance(token_id)
    }

    fn get_balance_of_contract(gas: u64, coins: u64, color: b256, params: ParamsContractBalance) -> u64 {
        balance_of_contract(params.token_id, params.contract_id)
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