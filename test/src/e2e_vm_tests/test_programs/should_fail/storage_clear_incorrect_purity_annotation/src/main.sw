contract;

abi MyContract {
    fn test_function();
}

impl MyContract for Contract {
    fn test_function() {
        std::storage::clear(0x0000000000000000000000000000000000000000000000000000000000000000);
    }
}
