contract;

use std::token::*;
use token_ops_abi::*;

impl TokenOps for Contract {


    fn mint(gas: u64, coins: u64, color: b256, mint_amount: u64) {
        mint(coins);
    }

    fn burn(gas: u64, coins: u64, color: b256, burn_amount: u64) {
        burn(coins);
    }

    fn transfer_to_output(gas: u64, coins: u64, color: b256, params: ParamsTRO) {
        transfer_to_output(params.coins, params.color, params.recipient);
    }
}
