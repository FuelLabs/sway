contract;

use std::token::*;
use token_ops_abi::TokenOps;

impl TokenOps for Contract {
    fn mint(gas: u64, coins: u64, color: b256, input: ()) {
        mint(coins);
    }

    fn burn(gas: u64, coins: u64, color: b256, input: ()) {
        burn(coins);
    }

    fn transfer_to_output(gas: u64, coins: u64, color: b256, input: ()) {
        transfer_to_output();
    }
}
