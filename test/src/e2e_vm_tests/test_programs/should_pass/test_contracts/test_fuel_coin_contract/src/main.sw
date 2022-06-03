contract;

use std::{contract_id::ContractId, token::{burn, force_transfer_to_contract, mint}};
use test_fuel_coin_abi::*;

impl TestFuelCoin for Contract {
    // TODO add event logging
    fn mint(mint_amount: u64) {
        mint(mint_amount);
    }

    fn burn(burn_amount: u64) {
        burn(burn_amount);
    }

    fn force_transfer(coins: u64, asset_id: ContractId, c_id: ContractId) {
        force_transfer_to_contract(coins, asset_id, c_id)
    }
}
