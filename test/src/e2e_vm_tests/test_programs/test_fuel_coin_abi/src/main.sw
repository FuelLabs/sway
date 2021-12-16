library test_fuel_coin_abi;

use std::address::Address;
use std::contract_id::ContractId;

/// Parameters for `transfer_to_output` function.
pub struct ParamsTransferToOutput {
    coins: u64,
    token_id: ContractId,
    recipient: Address,
}

/// Parameters for `force_transfer` function.
pub struct ParamsForceTransfer {
    coins: u64,
    token_id: ContractId,
    c_id: ContractId,
}

abi TestFuelCoin {
    fn mint(gas: u64, coins: u64, token_id: b256, mint_amount: u64);
    fn burn(gas: u64, coins: u64, token_id: b256, burn_amount: u64);
    fn transfer_to_output(gas: u64, coins: u64, token_id: b256, params: ParamsTransferToOutput);
    fn force_transfer(gas: u64, coins: u64, token_id: b256, params: ParamsForceTransfer);
    fn name(gas: u64, coins: u64, token_id: b256, input: ()) -> str[14];

}
