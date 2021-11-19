script;
use std::constants::ETH_COLOR;
use std::chain::assert;
use context_testing_abi::ContextTesting;

fn main() -> bool {
    let gas: u64 = 1000;
    let value: u64 = 11;
    let test_color: b256 = 0x000000000000000000000000000000000000000000000000000000000000002A;
    let deployed_id = 0x27b323db2cfa318890a8be57b223f40fb364419ba1999cb59eda061aea40730c;

    let test_contract = abi(ContextTesting, deployed_id);

    // test Context::this_id():
    let returned_id = test_contract.get_id(gas, 0, ETH_COLOR, ());
    assert(returned_id == deployed_id);

    // test Context::msg_value():
    let returned_value = test_contract.get_value(gas, value, ETH_COLOR, ());
    assert(returned_value == value);

    // test Context::msg_color():
    let returned_color = test_contract.get_color(gas, value, test_color, ());
    assert(returned_color == test_color);

    // test Context::msg_gas():
    let msg_gas = test_contract.get_gas(gas, value, test_color, ());
    assert(msg_gas == 1000);

    // test Context::global_gas():
    let global_gas = test_contract.get_global_gas(gas, value, test_color, ());
    assert(global_gas == 1000);

    true
}
