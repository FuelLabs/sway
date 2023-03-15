contract;

abi CalleeContract {
    fn test_true() -> bool;
}

impl CalleeContract for Contract {
    fn test_true() -> bool {
        true
    }
}
