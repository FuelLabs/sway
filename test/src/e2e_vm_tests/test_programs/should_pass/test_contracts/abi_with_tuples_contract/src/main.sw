contract;

use abi_with_tuples::{MyContract, Person, Location};

impl MyContract for Contract {
    fn bug1(param: (Person, u64)) -> bool {
        true
    }

    fn bug2(param: (Location, u64)) -> bool {
        true
    }
} 
