contract;

abi MethodTest {
    fn test_fn() -> bool;
}

impl MethodTest for Contract {
    fn test_fn() -> u64 {
        return 1;
    }
}
