library token_ops_abi;

use std::address::Address;

/// Parameters for `transfer_to_output` function.
pub struct ParamsTransferToOutput {
    coins: u64,
    token_id: b256,
    recipient: Address,
}

/// Parameters for `force_transfer` function.
pub struct ParamsForceTransfer {
    coins: u64,
    token_id: b256,
    contract_id: b256,
}

abi TokenOps {
    fn mint(gas: u64, coins: u64, token_id: b256, mint_amount: u64);
    fn burn(gas: u64, coins: u64, token_id: b256, burn_amount: u64);
    fn transfer_to_output(gas: u64, coins: u64, token_id: b256, params: ParamsTransferToOutput);
    fn force_transfer(gas: u64, coins: u64, token_id: b256, params: ParamsForceTransfer);
}
