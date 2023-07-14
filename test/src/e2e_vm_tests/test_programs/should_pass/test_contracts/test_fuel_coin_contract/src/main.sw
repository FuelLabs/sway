contract;

use std::{constants::ZERO_B256, token::{burn, force_transfer_to_contract, mint}};
use test_fuel_coin_abi::*;

impl TestFuelCoin for Contract {
    // TODO add event logging
    fn mint(mint_amount: u64) {
        mint(ZERO_B256, mint_amount);
    }

    fn burn(burn_amount: u64) {
        burn(ZERO_B256, burn_amount);
    }

    fn force_transfer(coins: u64, asset_id: b256, c_id: ContractId) {
        force_transfer_to_contract(c_id, asset_id, coins)
    }
}
