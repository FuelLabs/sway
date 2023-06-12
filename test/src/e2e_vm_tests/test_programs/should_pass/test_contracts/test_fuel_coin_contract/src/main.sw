contract;

use std::{constants::ZERO_B256, token::{burn, force_transfer_to_contract, mint}};
use test_fuel_coin_abi::*;

impl TestFuelCoin for Contract {
    // TODO add event logging
    fn mint(mint_amount: u64) {
        mint(mint_amount, ZERO_B256);
    }

    fn burn(burn_amount: u64) {
        burn(burn_amount, ZERO_B256);
    }

    fn force_transfer(coins: u64, asset_id: b256, c_id: ContractId) {
        force_transfer_to_contract(coins, asset_id, c_id)
    }
}
