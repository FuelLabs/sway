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

    let id = 0x69653340d655b4144ac0282e137e3907d5c4807803aacc3e66054fafb85d64d3;
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
