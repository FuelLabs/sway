contract;

abi MyContract {
    fn test_function();
}

impl MyContract for Contract {
    fn test_function() {
        let _ = __state_clear(b256::zero(), 0);
    }
}
