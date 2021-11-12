script;
use std::constants::ETH_COLOR;
use context_testing_abi::ContextTesting;

fn main() -> bool {
    let gas: u64 = 1000;
    let value: u64 = 11;
    let test_token_id: b256 = 0x000000000000000000000000000000000000000000000000000000000000002A;
    let deployed_id = 0x27b323db2cfa318890a8be57b223f40fb364419ba1999cb59eda061aea40730c;

    let test_contract = abi(ContextTesting, deployed_id);

    // test Contest::id():
    let returned_id = test_contract.get_id(gas, 0, ETH_COLOR, ());
    let t1 = returned_id == deployed_id;

    // test Msg::value():
    let returned_value = test_contract.get_value(gas, value, ETH_COLOR, ());
    let t2 = returned_value == value;

    // test Msg::token_id():
    let returned_token_id = test_contract.get_token_id(gas, value, test_token_id, ());
    let t3 = returned_token_id == test_token_id;

    // expect all results to be true:
    t1 && t2 && t3
}