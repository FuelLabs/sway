script;

use context_testing_abi::ContextTesting;

fn main() -> bool {
    let gas: u64 = 1000;
    let value: u64 = 11;
    let test_token_id: b256 = 0x000000000000000000000000000000000000000000000000000000000000002A;
    // @todo update this !
    let deployed_id = 0xad6aaaa1d6fd78f91693ee2cc124fd43d25bd1c015b88b675ee43d6b5e140586;
    let test_contract = abi(ContextTesting, deployed_id);

    let returned_id = test_contract.get_id(gas, 0, ETH_COLOR, ());
    let t1 = returned_id == deployed_id;

    let returned_value = test_contract.get_value(gas, value, ETH_COLOR, ());
    let t2 = returned_value == value;

    let returned_token_id = test_contract.get_token_id(gas, value, test_token_id, ());
    let t3 = returned_token_id == test_token_id;

    t1 && t2 && t3
}