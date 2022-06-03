contract;

use std::{address::Address, assert::assert, context::*, contract_id::ContractId, token::*};

abi NativeAssetToken {
    fn mint_coins(mint_amount: u64);
    fn burn_coins(burn_amount: u64);
    fn force_transfer_coins(coins: u64, asset_id: ContractId, target: ContractId);
    fn transfer_coins_to_output(coins: u64, asset_id: ContractId, recipient: Address);
    fn deposit();
    fn get_balance(target: ContractId, asset_id: ContractId) -> u64;
    fn mint_and_send_to_contract(amount: u64, destination: ContractId);
    fn mint_and_send_to_address(amount: u64, recipient: Address);
}

impl NativeAssetToken for Contract {
    /// Mint an amount of this contracts native asset to the contracts balance.
    fn mint_coins(mint_amount: u64) {
        mint(mint_amount);
    }

    /// Burn an amount of this contracts native asset.
    fn burn_coins(burn_amount: u64) {
        burn(burn_amount);
    }

    /// Transfer coins to a target contract.
    fn force_transfer_coins(coins: u64, asset_id: ContractId, target: ContractId) {
        force_transfer_to_contract(coins, asset_id, target);
    }

    /// Transfer coins to a transaction output to be spent later.
    fn transfer_coins_to_output(coins: u64, asset_id: ContractId, recipient: Address) {
        transfer_to_output(coins, asset_id, recipient);
    }

    /// Get the internal balance of a specific coin at a specific contract.
    fn get_balance(target: ContractId, asset_id: ContractId) -> u64 {
        balance_of(target, asset_id)
    }

    /// Deposit tokens back into the contract.
    fn deposit() {
        assert(msg_amount() > 0);
    }

    /// Mint and send this contracts native token to a destination contract.
    fn mint_and_send_to_contract(amount: u64, destination: ContractId) {
        mint_to_contract(amount, destination);
    }

    /// Mind and send this contracts native token to a destination address.
    fn mint_and_send_to_address(amount: u64, recipient: Address) {
        mint_to_address(amount, recipient);
    }
}
