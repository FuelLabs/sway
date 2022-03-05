contract;

use std::{context::*, contract_id::ContractId};
use context_testing_abi::*;

impl ContextTesting for Contract {
    fn get_id() -> b256 {
        contract_id()
    }

    fn get_this_balance(asset_id: b256) -> u64 {
        this_balance(asset_id)
    }

    fn get_balance_of_contract(asset_id: b256, cid: ContractId) -> u64 {
        balance_of_contract(asset_id, cid)
    }

    fn get_amount() -> u64 {
        msg_amount()
    }

    fn get_asset_id() -> b256 {
        msg_asset_id()
    }

    fn get_gas() -> u64 {
        gas()
    }

    fn get_global_gas() -> u64 {
        global_gas()
    }
}
