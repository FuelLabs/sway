contract;

use std::logging::log;

abi MyContract {
    fn test_function() -> bool;
}

impl MyContract for Contract {
    fn test_function() -> bool {
        revert(0);
        true
    }
}

#[test]
fn test_foo() {
    assert(true);
}

#[test]
fn test_bar() {
    let meaning = 6 * 7;
    log(meaning);
    assert(meaning == 42);
}
