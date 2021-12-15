script;
use std::constants::ETH_ID;
use std::constants::ZERO;
use std::chain::assert;
use std::contract_id::ContractId;
use context_testing_abi::ContextTesting;

fn main() -> bool {
    let gas: u64 = 1000;
    let amount: u64 = 11;
    let test_token_id: b256 = 0x000000000000000000000000000000000000000000000000000000000000002A;
    let deployed_contract_id = 0x9f03de8ad53cfcdc5b58e7630c78076a132f434fe74e6b355ac86cd4d0c75e2f;

    let test_contract = abi(ContextTesting, deployed_contract_id);

    // test Context::contract_id():
    let returned_contract_id = test_contract.get_id(gas, 0, ETH_ID, ());
    assert(returned_contract_id == deployed_contract_id);

    // @todo set up a test contract to mint some tokens for testing balances.
    // test Context::this_balance():
    let returned_this_balance = test_contract.get_this_balance(gas, 0, ETH_ID, ETH_ID);
    assert(returned_this_balance == ZERO;

    // test Context::balance_of_contract():
    let returned_contract_balance = test_contract.get_balance_of_contract(gas, 0, ETH_ID, ETH_ID, other_contract_id);
    assert(returned_contract_balance == ZERO;

    // test Context::msg_value():
    let returned_amount = test_contract.get_amount(gas, amount, ETH_ID, ());
    assert(returned_amount == amount);

    // test Context::msg_color():
    let returned_token_id = test_contract.get_token_id(gas, amount, test_token_id, ());
    assert(returned_token_id == test_token_id);

    // test Context::msg_gas():
    // @todo expect the correct gas here... this should fail using `1000`
    let gas = test_contract.get_gas(gas, amount, test_token_id, ());
    assert(gas == 1000);

    // test Context::global_gas():
    // @todo expect the correct gas here... this should fail using `1000`
    let global_gas = test_contract.get_global_gas(gas, amount, test_token_id, ());
    assert(global_gas == 1000);

    true
}
