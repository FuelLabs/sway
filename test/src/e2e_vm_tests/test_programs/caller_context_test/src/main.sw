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
    let deployed_contract_id = ~ContractId::from(0x27b323db2cfa318890a8be57b223f40fb364419ba1999cb59eda061aea40730c);

    let other_contract_id = ~ContractId::from(0x111323db2cfa318890a855555223f40fb364222ba1999cb59eda061ae3337123);

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
