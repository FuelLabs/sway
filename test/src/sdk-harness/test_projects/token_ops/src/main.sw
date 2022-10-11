contract;

use std::{
    context::balance_of,
    token::*,
    message::send_message,
    vec::Vec,
};

abi TestFuelCoin {
    fn mint_coins(mint_amount: u64);
    fn burn_coins(burn_amount: u64);
    fn force_transfer_coins(coins: u64, asset_id: ContractId, target: ContractId);
    fn transfer_coins_to_address(coins: u64, asset_id: ContractId, to: Address);
    fn get_balance(target: ContractId, asset_id: ContractId) -> u64;
    fn mint_and_send_to_contract(amount: u64, to: ContractId);
    fn mint_and_send_to_address(amount: u64, to: Address);
    fn generic_mint_to(amount: u64, to: Identity);
    fn generic_transfer(amount: u64, asset_id: ContractId, to: Identity);
    fn send_message(coins: u64, msg_data: Vec<u8>, recipient: b256);
}

impl TestFuelCoin for Contract {
    fn mint_coins(mint_amount: u64) {
        mint(mint_amount);
    }

    fn burn_coins(burn_amount: u64) {
        burn(burn_amount);
    }

    fn force_transfer_coins(coins: u64, asset_id: ContractId, target: ContractId) {
        force_transfer_to_contract(coins, asset_id, target);
    }

    fn transfer_coins_to_address(coins: u64, asset_id: ContractId, to: Address) {
        transfer_to_address(coins, asset_id, to);
    }

    fn get_balance(target: ContractId, asset_id: ContractId) -> u64 {
        balance_of(target, asset_id)
    }

    fn mint_and_send_to_contract(amount: u64, to: ContractId) {
        mint_to_contract(amount, to);
    }

    fn mint_and_send_to_address(amount: u64, to: Address) {
        mint_to_address(amount, to);
    }

    fn generic_mint_to(amount: u64, to: Identity) {
        mint_to(amount, to);
    }

    fn generic_transfer(amount: u64, asset_id: ContractId, to: Identity) {
        transfer(amount, asset_id, to)
    }

    fn send_message(coins: u64, msg_data: Vec<u8>, recipient: b256) {
        send_message(coins, msg_data, recipient);
    }
}
