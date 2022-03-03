contract;

use std::{address::Address, context::balance_of_contract, contract_id::ContractId, token::*};

/// Parameters for `force_transfer` function.
pub struct ParamsForceTransfer {
    coins: u64,
    asset_id: ContractId,
    target: ContractId,
}

/// Parameters for `transfer_to_output` function.
pub struct ParamsTransferToOutput {
    coins: u64,
    asset_id: ContractId,
    recipient: Address,
}

/// Parameters for `get_balance` function.
pub struct ParamsGetBalance {
    target: b256,
    asset_id: ContractId,
    salt: u64, // temp, see:https://github.com/FuelLabs/fuels-rs/issues/89
}

abi TestFuelCoin {
    fn mint_coins(gas_: u64, amount_: u64, color_: b256, mint_amount: u64);
    fn burn_coins(gas_: u64, amount_: u64, color_: b256, burn_amount: u64);
    fn force_transfer_coins(gas_: u64, amount_: u64, color_: b256, params: ParamsForceTransfer);
    fn transfer_coins_to_output(gas_: u64, amount_: u64, color_: b256, params: ParamsTransferToOutput);
    fn get_balance(gas_: u64, amount_: u64, color_: b256, params: ParamsGetBalance) -> u64;
}

impl TestFuelCoin for Contract {
    fn mint_coins(gas_: u64, amount_: u64, color_: b256, mint_amount: u64) {
        mint(mint_amount);
    }

    fn burn_coins(gas_: u64, amount_: u64, color_: b256, burn_amount: u64) {
        burn(burn_amount);
    }

    fn force_transfer_coins(gas_: u64, amount_: u64, color_: b256, params: ParamsForceTransfer) {
        force_transfer(params.coins, params.asset_id, params.target);
    }

    fn transfer_coins_to_output(gas_: u64, amount_: u64, color_: b256, params: ParamsTransferToOutput) {
        transfer_to_output(params.coins, params.asset_id, params.recipient);
    }

    fn get_balance(gas_: u64, amount_: u64, color_: b256, params: ParamsGetBalance) -> u64 {
        balance_of_contract(params.target, params.asset_id)
    }
}
