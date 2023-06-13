contract;

storage {
    x: u64 = 64,
}

abi MyContract {
    #[storage(read)]
    fn get_value() -> u64;
}

impl MyContract for Contract {
    #[storage(read)]
    fn get_value() -> u64 {
        storage.x.read()
    }
}
