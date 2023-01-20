contract;

dep inner;

abi MyContract {
    fn foo();
}

impl MyContract for Contract {
    fn foo() { }
}


#[test]
fn test_meaning_of_life() {
    let meaning = 6 * 7;
    assert(meaning == 42);
}

#[test]
fn log_test() {
    std::logging::log(1u32);
}
