contract;

use std::{address::Address, context::balance_of_contract, contract_id::ContractId, token::*};

abi TestFuelCoin {
    fn mint_coins(mint_amount: u64);
    fn burn_coins(burn_amount: u64);
    fn force_transfer_coins(coins: u64, asset_id: ContractId, target: ContractId);
    fn transfer_coins_to_output(coins: u64, asset_id: ContractId, recipient: Address);
    fn get_balance(target: b256, asset_id: ContractId) -> u64;
}

impl TestFuelCoin for Contract {
    fn mint_coins(mint_amount: u64) {
        mint(mint_amount);
    }

    fn burn_coins(burn_amount: u64) {
        burn(burn_amount);
    }

    fn force_transfer_coins(coins: u64, asset_id: ContractId, target: ContractId) {
        force_transfer(coins, asset_id, target);
    }

    fn transfer_coins_to_output(coins: u64, asset_id: ContractId, recipient: Address) {
        transfer_to_output(coins, asset_id, recipient);
    }

    fn get_balance(target: b256, asset_id: ContractId) -> u64 {
        balance_of_contract(target, asset_id)
    }
}
