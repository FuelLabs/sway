contract;

use std::token::*;
use test_fuel_coin_abi::*;

impl TestFuelCoin for Contract {
    fn mint_coins(gas: u64, coins: u64, asset_id: b256, mint_amount: u64) {
        mint(coins);
    }

    fn burn_coins(gas: u64, coins: u64, asset_id: b256, burn_amount: u64) {
        burn(coins);
    }

    fn force_transfer_coins(gas: u64, coins: u64, asset_id: b256, params: ParamsForceTransfer) {
        force_transfer(params.coins, params.asset_id, params.c_id);
    }

    fn transfer_coins_to_output(gas: u64, coins: u64, asset_id: b256, params: ParamsTransferToOutput) {
        transfer_to_output(params.coins, params.asset_id, params.recipient);
    }
}
