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
    let contract_id = 0x76a777c5bb9177b85895f4172fb4159472da6f65e62eba9502484a38c0246228;
    let caller = abi(MyContract, contract_id);
    let result = caller.test_function {}();
    assert(result == false)
}

#[test]
fn test_success() {
    let contract_id = 0x76a777c5bb9177b85895f4172fb4159472da6f65e62eba9502484a38c0246228;
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
