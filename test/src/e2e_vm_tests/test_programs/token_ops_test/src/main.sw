script;
use std::constants::ETH_ID;
use std::chain::assert;
use std::address::Address;
use std::contract_id::ContractId;
use test_token_abi::*;

fn main() -> bool {
    let test_recipient = ~Address::from(0x3333333333333333333333333333333333333333333333333333333333333333);
    let test_contract_id = ~ContractId::from(0x2222222222222222222222222222222222222222222222222222222222222222);

    let transfer_to_output_args = ParamsTransferToOutput {
        coins: 5,
        token_id: ETH_ID,
        recipient: test_recipient,
    };

    let test_token_id = ~ContractId::from(<id>);
    let test_token_caller = abi(TestToken, test_token_id);
    let gas = 1000;
    let coins = 0;
    let token_id = ETH_ID;

    // @todo add total supply modification checks for force_transfer,  mint & burn once balance() is added to stdlib lands.
    // use test_contract_id for balance checks
    let mut balance = balance_of_contract(test_contract_id)
    assert(starting_balance == ZERO);

    test_token_caller.mint(gas, coins, token_id, 11);

    balance = balance_of_contract(test_contract_id)
    assert(balance == 11);

    test_token_caller.burn(gas, coins, token_id, 7);

    balance = balance_of_contract(test_contract_id)
    assert(balance == 4);

    let force_transfer_args = ParamsForceTransfer {
        coins: 3,
        token_id: <id>,
        contract_id: test_token_id,
    };
    let mut balance2 = balance_of_contract(test_token_id)
    assert(balance == ZERO);

    test_token_caller.force_transfer(gas, coins, token_id, force_transfer_args);

    balance = balance_of_contract(test_contract_id)
    balance2 = balance_of_contract(test_token_id)
    assert(balance == 1);
    assert(balance2 == 3);

    test_token_caller.transfer_to_output(gas, coins, token_id, transfer_to_output_args);

    true
}
