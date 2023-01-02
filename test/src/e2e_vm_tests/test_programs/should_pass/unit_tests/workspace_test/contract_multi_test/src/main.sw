contract;

use std::logging::log;

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
    let contract_id = 0xf4f60cccafd4f4fabbb04e867a3aacde7c3aca04b5d8311355e18503427a8191; 
    let caller = abi(MyContract, contract_id);
    let result = caller.test_function{}();
    assert(result == true);
}

#[test]
fn test_bar() {
    let meaning = 6 * 7;
    log(meaning);
    assert(meaning == 42);
}
