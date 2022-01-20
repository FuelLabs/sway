contract;

use std::token::*;
use test_fuel_coin_abi::*;

impl TestFuelCoin for Contract {

    // TODO add event logging
    fn mint(gas: u64, coins: u64, asset_id: b256, mint_amount: u64) {
        mint(coins);
    }
}
