contract;

use std::{bytes::Bytes, context::balance_of, message::send_message, token::*};

abi TestFuelCoin {
    fn mint_coins(mint_amount: u64);
    fn burn_coins(burn_amount: u64);
    fn force_transfer_coins(coins: u64, asset_id: ContractId, target: ContractId);
    fn transfer_coins_to_address(coins: u64, asset_id: ContractId, to: Address);
    fn get_balance(asset_id: ContractId, target: ContractId) -> u64;
    fn mint_and_send_to_contract(amount: u64, to: ContractId);
    fn mint_and_send_to_address(amount: u64, to: Address);
    fn generic_mint_to(amount: u64, to: Identity);
    fn generic_transfer(amount: u64, asset_id: ContractId, to: Identity);
    fn send_message(recipient: b256, msg_data: Vec<u64>, coins: u64);

    fn address_mint_to(recipient: Address, amount: u64);
    fn address_transfer(recipient: Address, asset_id: ContractId, amount: u64);
    fn contract_mint_to(recipient: ContractId, amount: u64);
    fn contract_transfer(recipient: ContractId, asset_id: ContractId, amount: u64);
    fn identity_mint_to(recipient: Identity, amount: u64);
    fn identity_transfer(recipient: Identity, asset_id: ContractId, amount: u64);
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

    fn get_balance(asset_id: ContractId, target: ContractId) -> u64 {
        balance_of(asset_id, target)
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

    fn send_message(recipient: b256, msg_data: Vec<u64>, coins: u64) {
        let mut data = Bytes::new();
        if msg_data.len() > 0 {
            data.push(msg_data.get(0).unwrap().try_as_u8().unwrap());
            data.push(msg_data.get(1).unwrap().try_as_u8().unwrap());
            data.push(msg_data.get(2).unwrap().try_as_u8().unwrap());
        }

        send_message(recipient, data, coins);
    }

    fn address_mint_to(recipient: Address, amount: u64) {
        recipient.mint_to(amount);
    }

    fn address_transfer(recipient: Address, asset_id: ContractId, amount: u64) {
        recipient.transfer(amount, asset_id);
    }

    fn contract_mint_to(recipient: ContractId, amount: u64) {
        recipient.mint_to(amount);
    }

    fn contract_transfer(recipient: ContractId, asset_id: ContractId, amount: u64) {
        recipient.transfer(amount, asset_id);
    }

    fn identity_mint_to(recipient: Identity, amount: u64) {
        recipient.mint_to(amount);
    }

    fn identity_transfer(recipient: Identity, asset_id: ContractId, amount: u64) {
        recipient.transfer(amount, asset_id);
    }
}
