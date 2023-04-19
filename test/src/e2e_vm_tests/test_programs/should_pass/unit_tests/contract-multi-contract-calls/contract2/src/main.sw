contract;

abi MyContract2 {
    fn test_false() -> bool;
}

impl MyContract2 for Contract {
    fn test_false() -> bool {
    	false
    }
}
