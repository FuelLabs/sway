script;
use std::constants::ETH_COLOR;
use std::chain::assert;
use std::address::Address;
use token_ops_abi::*;

fn main() -> bool {
    let test_recipient = ~Address::from(0x3333333333333333333333333333333333333333333333333333333333333333);
    let test_constract_id = ~Address::from(0x2222222222222222222222222222222222222222222222222222222222222222);

    let transfer_to_output_args = ParamsTransferToOutput {
        coins: 5,
        color: ETH_COLOR,
        recipient: test_recipient,
    };

    let force_transfer_args = ParamsForceTransfer {
        coins: 5,
        color: ETH_COLOR,
        contract_id: test_constract_id,
    };

    let id = 0x69653340d655b4144ac0282e137e3907d5c4807803aacc3e66054fafb85d64d3;
    let gas = 1000;
    let coins = 0;
    let color = ETH_COLOR;

    let caller = abi(TokenOps, id);

    // @todo add total supply modification checks once balance opcode lands.
    caller.mint(gas, coins, color, 11);
    caller.burn(gas, coins, color, 7);
    caller.transfer_to_output(gas, coins, color, transfer_to_output_args);
    caller.force_transfer(gas, coins, color, force_transfer_args);

    true
}
