contract;

abi MyContract {
    #[storage(write)]
    fn increment_value();

    #[storage(read)]
    fn get_value() -> u64;
}

storage {
    value: u64 = 0,
}

impl MyContract for Contract {
    #[storage(write)]
    fn increment_value() {
        let current_value = storage.value.read();
        storage.value.write(current_value + 1);
    }

    #[storage(read)]
    fn get_value() -> u64 {
        storage.value.read()
    }
}

#[test]
fn test_increment_value() {
    let caller = abi(MyContract, CONTRACT_ID);

    caller.increment_value();
    caller.increment_value();
    caller.increment_value();
    caller.increment_value();
    caller.increment_value();
    caller.increment_value();
    caller.increment_value();
    caller.increment_value();
    caller.increment_value();

    // Get the value and assert it is correct
    let result = caller.get_value();
    assert(result == 10);
}
