script;
use std::constants::ETH_ID;
use std::chain::assert;
use std::address::Address;
use token_ops_abi::*;

fn main() -> bool {
    let test_recipient = ~Address::from(0x3333333333333333333333333333333333333333333333333333333333333333);
    let test_contract_id = 0x2222222222222222222222222222222222222222222222222222222222222222;

    let transfer_to_output_args = ParamsTransferToOutput {
        coins: 5,
        token_id: ETH_ID,
        recipient: test_recipient,
    };

    let force_transfer_args = ParamsForceTransfer {
        coins: 5,
        token_id: ETH_ID,
        contract_id: test_contract_id,
    };

    let id = 0x314143b15215da1248f0c09eba442764f1324a44c3bfcca022ddd1fc2008c542;
    let gas = 1000;
    let coins = 0;
    let token_id = ETH_ID;

    let caller = abi(TokenOps, id);

    // @todo add total supply modification checks for force_transfer. mint & burn once balance() is added to stdlib lands.
    caller.mint(gas, coins, token_id, 11);
    caller.burn(gas, coins, token_id, 7);
    caller.transfer_to_output(gas, coins, token_id, transfer_to_output_args);
    caller.force_transfer(gas, coins, token_id, force_transfer_args);

    true
}
