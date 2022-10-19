contract;

abi BytecodeContract {
    fn test_function() -> bool;
}

impl BytecodeContract for Contract {
    fn test_function() -> bool {
        true
    }
}
