contract;

use std::{call_frames::{contract_id, msg_asset_id}, context::{balance_of, gas, msg_amount, this_balance}, contract_id::ContractId, registers::global_gas};
use context_testing_abi::*;

impl ContextTesting for Contract {
    fn get_id() -> ContractId {
        contract_id()
    }

    fn get_this_balance(asset_id: ContractId) -> u64 {
        this_balance(asset_id)
    }

    fn get_balance_of_contract(asset_id: ContractId, cid: ContractId) -> u64 {
        balance_of(asset_id, cid)
    }

    fn get_amount() -> u64 {
        msg_amount()
    }

    fn get_asset_id() -> ContractId {
        msg_asset_id()
    }

    fn get_gas() -> u64 {
        gas()
    }

    fn get_global_gas() -> u64 {
        global_gas()
    }
}
