contract;

use std::token::*;
use test_fuel_coin_abi::*;

// Name of the coin managed by this contract
const name: str[14] = "Test Fuel Coin";

impl TestFuelCoin for Contract {


    // @todo add event logging
    fn mint(gas: u64, coins: u64, token_id: b256, mint_amount: u64) {
        mint(coins);
    }

    fn burn(gas: u64, coins: u64, token_id: b256, burn_amount: u64) {
        burn(coins);
    }

    fn transfer_to_output(gas: u64, coins: u64, token_id: b256, params: ParamsTransferToOutput) {
        transfer_to_output(params.coins, params.token_id.value, params.recipient);
    }

    fn force_transfer(gas: u64, coins: u64, token_id: b256, params: ParamsForceTransfer) {
        force_transfer(params.coins, params.token_id.value, params.c_id.value)
    }

    fn name(gas: u64, coins: u64, token_id: b256, input: ()) -> str[14] {
        name
    }
}