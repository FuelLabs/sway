contract;

abi MyContract {
    #[storage(read)]
    fn test_function() -> bool;
}

impl MyContract for Contract {
    #[storage(write)]
    fn test_function() -> bool {
        true
    }
}
