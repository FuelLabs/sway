contract;

use std::token::*;
use token_ops_abi::*;

impl TokenOps for Contract {
    fn mint(gas: u64, coins: u64, token_id: b256, mint_amount: u64) {
        mint(coins);
    }

    fn burn(gas: u64, coins: u64, token_id: b256, burn_amount: u64) {
        burn(coins);
    }

    fn transfer_to_output(gas: u64, coins: u64, token_id: b256, params: ParamsTransferToOutput) {
        transfer_to_output(params.coins, params.token_id, params.recipient);
    }
    fn force_transfer(gas: u64, coins: u64, token_id: b256, params: ParamsForceTransfer) {
        force_transfer(params.coins, params.token_id, params.contract_id)
    }
}
