contract;

abi TestContract {
    fn initialize_counter(value: u64) -> u64;
    fn increment_counter(amount: u64) -> u64;
}

storage {
    counter: u64,
}

impl TestContract for Contract {
    fn initialize_counter(value: u64) -> u64 {
        storage.counter = value;
        value
    }

    fn increment_counter(amount: u64) -> u64 {
        let incremented = storage.counter + amount;
        storage.counter = incremented;
        incremented
    }
}
