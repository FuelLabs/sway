script;
use std::constants::ETH_COLOR;
use std::chain::assert;
use std::address::Address;
use token_ops_abi::*;

fn main() -> bool {
    let test_recipient = ~Address::from(0x3333333333333333333333333333333333333333333333333333333333333333);

    let transfer_args = ParamsTRO {
        coins: 5,
        color: ETH_COLOR,
        recipient: test_recipient
    };

    let id = 0xb425088803674c94ca9df119c8ac429905ab45d69285534cfbc4e7ffab9bbd5f;
    let gas = 1000;
    let coins = 0;
    let color = ETH_COLOR;

    let token_ops_contract = abi(TokenOps, id);
    // @todo add total supply modification checks once balance opcode lands.
    token_ops_contract.mint(gas, coins, color, 11);
    token_ops_contract.burn(gas, coins, color, 7);
    token_ops_contract.transfer_to_output(gas, coins, color, transfer_args);

    true
}
