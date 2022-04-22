contract;

abi Storage {
    fn test_function() -> bool;
}

impl Storage for Contract {
    fn test_function() -> bool {
        true
    }
}
