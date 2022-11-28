contract;

use std::{
    context::*,
    call_frames::*,
    registers::*,
};
use context_testing_abi::*;

impl ContextTesting for Contract {
    #[payable]
    fn get_this_balance(asset: ContractId) -> u64 {
        this_balance(asset)
    }

    #[payable]
    fn get_balance_of_contract(asset: ContractId, r#contract: ContractId) -> u64 {
        balance_of(asset, r#contract)
    }

    #[payable]
    fn get_amount() -> u64 {
        msg_amount()
    }

    #[payable]
    fn get_asset_id() -> ContractId {
        msg_asset_id()
    }

    #[payable]
    fn get_gas() -> u64 {
        gas()
    }

    #[payable]
    fn get_global_gas() -> u64 {
        global_gas()
    }

    #[payable]
    fn receive_coins() {}
}
