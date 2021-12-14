script;
use std::constants::ETH_ID;
use std::chain::assert;
use context_testing_abi::ContextTesting;

fn main() -> bool {
    let gas: u64 = 1000;
    let amount: u64 = 11;
    let test_token_id: b256 = 0x000000000000000000000000000000000000000000000000000000000000002A;
    let deployed_contract_id = 0x14f4c2c919e7a2c387131f6b2e67b7560af16c728af79aadad925d612ea02d0b;

    let test_contract = abi(ContextTesting, deployed_contract_id);

    // test Context::this_id():
    let returned_contract_id = test_contract.get_id(gas, 0, ETH_ID, ());
    assert(returned_contract_id == deployed_contract_id);

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
