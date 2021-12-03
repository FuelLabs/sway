script;
use std::constants::ETH_COLOR;
use std::chain::assert;
use std::address::Address;
use token_ops_abi::*;

fn main() -> bool {
    // @todo fix import after new constans merged.
    let ETH_ID = ETH_COLOR;
    let test_recipient = ~Address::from(0x3333333333333333333333333333333333333333333333333333333333333333);
    let test_constract_id = ~Address::from(0x2222222222222222222222222222222222222222222222222222222222222222);

    let transfer_to_output_args = ParamsTransferToOutput {
        coins: 5,
        token_id: ETH_ID,
        recipient: test_recipient,
    };

    let force_transfer_args = ParamsForceTransfer {
        coins: 5,
        token_id: ETH_ID,
        contract_id: test_constract_id,
    };

    let id = 0x69653340d655b4144ac0282e137e3907d5c4807803aacc3e66054fafb85d64d3;
    let gas = 1000;
    let coins = 0;
    let token_id = ETH_ID;

    let caller = abi(TokenOps, id);

    // @todo add total supply modification checks once balance opcode lands.
    caller.mint(gas, coins, token_id, 11);
    caller.burn(gas, coins, token_id, 7);
    caller.transfer_to_output(gas, coins, token_id, transfer_to_output_args);
    caller.force_transfer(gas, coins, token_id, force_transfer_args);

    true
}
