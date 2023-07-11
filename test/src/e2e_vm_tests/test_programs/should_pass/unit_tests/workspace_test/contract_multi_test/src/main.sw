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
    let contract_id = 0x5e0a7976e5b870e6412cb9ed5f038a0db4feb328c5c7fa595b4f6b8d3b42f55f;
    let caller = abi(MyContract, contract_id);
    let result = caller.test_function {}();
    assert(result == false)
}

#[test]
fn test_success() {
    let contract_id = 0x5e0a7976e5b870e6412cb9ed5f038a0db4feb328c5c7fa595b4f6b8d3b42f55f;
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
