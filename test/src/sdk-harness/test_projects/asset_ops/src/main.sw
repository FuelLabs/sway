contract;

use std::{
    asset::*,
    bytes::Bytes,
    context::balance_of,
    message::send_message,
    primitive_conversions::u64::*,
};

abi TestFuelCoin {
    fn mint_coins(mint_amount: u64, sub_id: b256);
    fn burn_coins(burn_amount: u64, sub_id: b256);
    fn force_transfer_coins(coins: u64, asset_id: b256, target: ContractId);
    fn transfer_coins_to_address(coins: u64, asset_id: b256, to: Address);
    fn get_balance(asset_id: b256, target: ContractId) -> u64;
    fn mint_and_send_to_contract(amount: u64, to: ContractId, sub_id: b256);
    fn mint_and_send_to_address(amount: u64, to: Address, sub_id: b256);
    fn generic_mint_to(amount: u64, to: Identity, sub_id: b256);
    fn generic_transfer(amount: u64, asset_id: b256, to: Identity);
    fn send_message(recipient: b256, msg_data: Vec<u64>, coins: u64);
}

impl TestFuelCoin for Contract {
    fn mint_coins(mint_amount: u64, sub_id: b256) {
        mint(sub_id, mint_amount);
    }

    fn burn_coins(burn_amount: u64, sub_id: b256) {
        burn(sub_id, burn_amount);
    }

    fn force_transfer_coins(coins: u64, asset_id: b256, target: ContractId) {
        let asset_id = AssetId::from(asset_id);
        transfer(Identity::ContractId(target), asset_id, coins);
    }

    fn transfer_coins_to_address(coins: u64, asset_id: b256, to: Address) {
        let asset_id = AssetId::from(asset_id);
        transfer(Identity::Address(to), asset_id, coins);
    }

    fn get_balance(asset_id: b256, target: ContractId) -> u64 {
        let asset_id = AssetId::from(asset_id);
        balance_of(target, asset_id)
    }

    fn mint_and_send_to_contract(amount: u64, to: ContractId, sub_id: b256) {
        mint_to(Identity::ContractId(to), sub_id, amount);
    }

    fn mint_and_send_to_address(amount: u64, to: Address, sub_id: b256) {
        mint_to(Identity::Address(to), sub_id, amount);
    }

    fn generic_mint_to(amount: u64, to: Identity, sub_id: b256) {
        mint_to(to, sub_id, amount);
    }

    fn generic_transfer(amount: u64, asset_id: b256, to: Identity) {
        let asset_id = AssetId::from(asset_id);
        transfer(to, asset_id, amount)
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
}
