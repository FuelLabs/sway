contract;

abi MyContract {
    fn test_function() -> bool;
}

impl MyContract for Contract {
    fn test_function() -> bool {
        asm() {
            blob i91000;
        }
        true
    }
}
