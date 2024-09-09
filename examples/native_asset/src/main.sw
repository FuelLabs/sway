contract;

use std::{asset::*, call_frames::msg_asset_id, constants::DEFAULT_SUB_ID, context::*};

abi NativeAsset {
    fn mint_coins(mint_amount: u64);
    fn burn_coins(burn_amount: u64);
    fn transfer_coins(coins: u64, asset_id: AssetId, target: Identity);
    #[payable]
    fn deposit();
    fn get_balance(target: ContractId, asset_id: AssetId) -> u64;
    fn get_msg_amount();
    fn this_balance(asset_id: AssetId) -> u64;
    fn get_msg_asset_id();
    fn mint_coins_to(target_identity: Identity, mint_amount: u64);
}

impl NativeAsset for Contract {
    /// Mint an amount of this contracts native asset to the contracts balance.
    fn mint_coins(mint_amount: u64) {
        // ANCHOR: mint_asset
        mint(DEFAULT_SUB_ID, mint_amount);
        // ANCHOR_END: mint_asset
    }

    fn mint_coins_to(target_identity: Identity, mint_amount: u64) {
        // ANCHOR: mint_to_asset
        mint_to(target_identity, DEFAULT_SUB_ID, mint_amount);
        // ANCHOR_END: mint_to_asset
    }

    /// Burn an amount of this contracts native asset.
    fn burn_coins(burn_amount: u64) {
        // ANCHOR: burn_asset
        burn(DEFAULT_SUB_ID, burn_amount);
        // ANCHOR_END: burn_asset
    }

    /// Transfer coins to a target contract.
    fn transfer_coins(coins: u64, asset_id: AssetId, target: Identity) {
        // ANCHOR: transfer_asset
        transfer(target, asset_id, coins);
        // ANCHOR_END: transfer_asset
    }

    /// Get the internal balance of a specific coin at a specific contract.
    fn get_balance(target_contract: ContractId, asset_id: AssetId) -> u64 {
        // ANCHOR: balance_of
        balance_of(target_contract, asset_id)
        // ANCHOR_END: balance_of
    }

    /// Get the internal balance of a specific coin at a specific contract.
    fn this_balance(asset_id: AssetId) -> u64 {
        // ANCHOR: this_balance
        this_balance(asset_id)
        // ANCHOR_END: this_balance
    }

    /// Deposit coins back into the contract.
    // ANCHOR: payable
    #[payable]
    fn deposit() {
        assert(msg_amount() > 0);
    }
    // ANCHOR_END: payable
    /// Mint and send this contracts native asset to a destination contract.
    fn get_msg_amount() {
        // ANCHOR: msg_amount
        let amount = msg_amount();
        // ANCHOR_END: msg_amount
    }

    /// Mint and send this contracts native asset to a destination contract.
    fn get_msg_asset_id() {
        // ANCHOR: msg_asset_id
        let amount = msg_asset_id();
        // ANCHOR_END: msg_asset_id
    }
}

fn get_base_asset() {
    // ANCHOR: base_asset
    let base_asset: AssetId = AssetId::base();
    // ANCHOR_END: base_asset
}

fn default_asset_id() {
    // ANCHOR: default_asset_id
    let asset_id: AssetId = AssetId::default();
    // ANCHOR_END: default_asset_id
}

fn new_asset_id(my_contract_id: ContractId, my_sub_id: SubId) {
    // ANCHOR: new_asset_id
    let my_contract_id: ContractId = ContractId::from(0x1000000000000000000000000000000000000000000000000000000000000000);
    let my_sub_id: SubId = 0x2000000000000000000000000000000000000000000000000000000000000000;

    let asset_id: AssetId = AssetId::new(my_contract_id, my_sub_id);
    // ANCHOR_END: new_asset_id
}

fn from_asset_id() {
    // ANCHOR: from_asset_id
    let asset_id: AssetId = AssetId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    // ANCHOR_END: from_asset_id
}
