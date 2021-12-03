library token_ops_abi;

use std::address::Address;

/// Parameters for `transfer_to_output` method.
pub struct ParamsTRO {
    coins: u64,
    color: b256,
    recipient: Address
}

abi TokenOps {
    fn mint(gas: u64, coins: u64, color: b256, mint_amount: u64);
    fn burn(gas: u64, coins: u64, color: b256, burn_amount: u64);
    fn transfer_to_output(gas: u64, coins: u64, color: b256, params: ParamsTRO);
}
