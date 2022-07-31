contract;

abi MyContract {
    fn test_function() -> bool;
}

impl MyContract for Contract {
    pub fn test_function() -> bool {
        true
    }
}