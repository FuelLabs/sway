contract;

use std::{
    context::*,
    call_frames::*,
    registers::*,
};
use context_testing_abi::*;

impl ContextTesting for Contract {
    fn get_this_balance(asset: ContractId) -> u64 {
        this_balance(asset)
    }

    fn get_balance_of_contract(asset: ContractId, r#contract: ContractId) -> u64 {
        balance_of(asset, r#contract)
    }

    fn get_amount() -> u64 {
        msg_amount()
    }

    fn get_asset_id() -> ContractId {
        msg_asset_id()
    }

    fn get_gas() -> u64 {
        context_gas()
    }

    fn get_global_gas() -> u64 {
        global_gas()
    }

    fn receive_coins() {}
}
