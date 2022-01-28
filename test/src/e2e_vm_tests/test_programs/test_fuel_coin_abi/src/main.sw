library test_fuel_coin_abi;

use std::{address::Address, contract_id::ContractId};

/// Parameters for `force_transfer` function.
pub struct ParamsForceTransfer {
    coins: u64,
    asset_id: ContractId,
    c_id: ContractId,
}

/// Parameters for `transfer_to_output` function.
pub struct ParamsTransferToOutput {
    coins: u64,
    asset_id: ContractId,
    recipient: Address,
}

abi TestFuelCoin {
    fn mint_coins(gas: u64, coins: u64, asset_id: b256, mint_amount: u64);
    fn burn_coins(gas: u64, coins: u64, asset_id: b256, burn_amount: u64);
    fn force_transfer_coins(gas: u64, coins: u64, asset_id: b256, params: ParamsForceTransfer);
    fn transfer_coins_to_output(gas: u64, coins: u64, asset_id: b256, params: ParamsTransferToOutput);
}
