contract;

abi MyContract {
    fn test_function(p: u64);
}

impl MyContract for Contract {
    fn test_function(ref mut p: u64) {

    }
}
