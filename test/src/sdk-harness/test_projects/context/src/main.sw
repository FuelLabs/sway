contract;

use std::{call_frames::*, context::*, registers::*};
use context_testing_abi::*;

impl ContextTesting for Contract {
    #[payable]
    fn get_this_balance(asset: b256) -> u64 {
        let asset = AssetId::from(asset);
        this_balance(asset)
    }

    #[payable]
    fn get_balance_of_contract(asset: b256, r#contract: ContractId) -> u64 {
        let asset = AssetId::from(asset);
        balance_of(r#contract, asset)
    }

    #[payable]
    fn get_amount() -> u64 {
        msg_amount()
    }

    #[payable]
    fn get_asset_id() -> b256 {
        msg_asset_id().bits()
    }

    #[payable]
    fn get_gas() -> u64 {
        context_gas()
    }

    #[payable]
    fn get_global_gas() -> u64 {
        global_gas()
    }

    #[payable]
    fn receive_coins() {}
}
