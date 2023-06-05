contract;

abi MyContract {
    fn test_false() -> bool;
}

impl MyContract for Contract {
    fn test_false() -> bool {
    	false
    }
}
