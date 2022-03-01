contract;

use std::{contract_id::ContractId, token::*};
use test_fuel_coin_abi::*;

pub fn balance() -> u64 {
    asm() {
        bal
    }
}

pub fn context_gas() -> u64 {
    asm() {
        cgas
    }
}

pub fn frame_ptr() -> u64 {
    asm() {
        fp
    }
}

impl TestFuelCoin for Contract {
    // TODO add event logging
    fn mint(mint_amount: u64) {
        mint(balance());
    }

    fn burn(burn_amount: u64) {
        burn(balance());
    }

    fn force_transfer(coins: u64, asset_id: ContractId, c_id: ContractId) {
        force_transfer(coins, asset_id, c_id)
    }
}
