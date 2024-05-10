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
    let contract_id = 0x22f840cd4af724e7257dd01ce315d907d694e417778b5f16b205a918baee092e;
    let caller = abi(MyContract, contract_id);
    let result = caller.test_function {}();
    assert(result == false)
}

#[test]
fn test_success() {
    let contract_id = 0x22f840cd4af724e7257dd01ce315d907d694e417778b5f16b205a918baee092e;
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
