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
    let contract_id = 0x535d1bbf19a6c7e5e2e2906d09b8c959bd211391e7a3c7602c5dab0455f78305;
    let caller = abi(MyContract, contract_id);
    let result = caller.test_function {}();
    assert(result == false)
}

#[test]
fn test_success() {
    let contract_id = 0x535d1bbf19a6c7e5e2e2906d09b8c959bd211391e7a3c7602c5dab0455f78305;
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
