contract;

abi MyContract {
    fn test_function() -> bool;
}

impl MyContract for Contract {
    fn test_function() -> bool {
        true
    }
}

#[test]
fn test_foo() {
    assert(true)
}

#[test(should_revert)]
fn test_fail() {
    let contract_id = 0xaff6151370c3ddd774601146cf743dff5dc0b1bc485a46da875be31f2b3e7493;
    let caller = abi(MyContract, contract_id);
    let result = caller.test_function {}();
    assert(result == false)
}

#[test]
fn test_success() {
    let contract_id = 0xaff6151370c3ddd774601146cf743dff5dc0b1bc485a46da875be31f2b3e7493;
    let caller = abi(MyContract, contract_id);
    let result = caller.test_function {}();
    assert(result == true)
}

#[test]
fn test_bar() {
    let meaning = 6 * 7;
    log(meaning);
    assert(meaning == 42)
}
