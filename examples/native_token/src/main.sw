contract;

use std::{constants::ZERO_B256, context::*, token::*};

abi NativeAssetToken {
    fn mint_coins(mint_amount: u64);
    fn burn_coins(burn_amount: u64);
    fn force_transfer_coins(coins: u64, asset_id: b256, target: ContractId);
    fn transfer_coins_to_output(coins: u64, asset_id: b256, recipient: Address);
    fn deposit();
    fn get_balance(target: ContractId, asset_id: b256) -> u64;
    fn mint_and_send_to_contract(amount: u64, destination: ContractId);
    fn mint_and_send_to_address(amount: u64, recipient: Address);
}

impl NativeAssetToken for Contract {
    /// Mint an amount of this contracts native asset to the contracts balance.
    fn mint_coins(mint_amount: u64) {
        mint(ZERO_B256, mint_amount);
    }

    /// Burn an amount of this contracts native asset.
    fn burn_coins(burn_amount: u64) {
        burn(ZERO_B256, burn_amount);
    }

    /// Transfer coins to a target contract.
    fn force_transfer_coins(coins: u64, asset_id: b256, target: ContractId) {
        force_transfer_to_contract(target, asset_id, coins);
    }

    /// Transfer coins to a transaction output to be spent later.
    fn transfer_coins_to_output(coins: u64, asset_id: b256, recipient: Address) {
        transfer_to_address(recipient, asset_id, coins);
    }

    /// Get the internal balance of a specific coin at a specific contract.
    fn get_balance(target: ContractId, asset_id: b256) -> u64 {
        balance_of(target, asset_id)
    }

    /// Deposit tokens back into the contract.
    fn deposit() {
        assert(msg_amount() > 0);
    }

    /// Mint and send this contracts native token to a destination contract.
    fn mint_and_send_to_contract(amount: u64, destination: ContractId) {
        mint_to_contract(destination, ZERO_B256, amount);
    }

    /// Mind and send this contracts native token to a destination address.
    fn mint_and_send_to_address(amount: u64, recipient: Address) {
        mint_to_address(recipient, ZERO_B256, amount);
    }
}
