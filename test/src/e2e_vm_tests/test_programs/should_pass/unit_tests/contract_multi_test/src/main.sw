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
    assert(true);
}

#[test(should_revert)]
fn test_fail() {
    let caller = abi(MyContract, CONTRACT_ID);
    let result = caller.test_function {}();
    assert(result == false)
}

#[test]
fn test_success() {
    let caller = abi(MyContract, CONTRACT_ID);
    let result = caller.test_function {}();
    assert(result == true)
}

#[test]
fn test_bar() {
    let meaning = 6 * 7;
    log(meaning);
    assert(meaning == 42);
}
