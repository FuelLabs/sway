contract;

use std::token::*;
use test_fuel_coin_abi::*;

impl TestFuelCoin for Contract {

    // TODO add event logging
    fn mint(gas: u64, coins: u64, asset_id: b256, mint_amount: u64) {
        mint(coins);
    }

    fn burn(gas: u64, coins: u64, asset_id: b256, burn_amount: u64) {
        burn(coins);
    }

    fn force_transfer(gas: u64, coins: u64, asset_id: b256, params: ParamsForceTransfer) {
        force_transfer(params.coins, params.asset_id, params.c_id)
    }
}
