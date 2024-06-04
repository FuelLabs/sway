contract;

use abi_with_tuples::{MyContract, Person, Location, some_module::SomeStruct};

impl MyContract for Contract {
    fn bug1(_param: (Person, u64)) -> bool {
        true
    }

    fn bug2(_param: (Location, u64)) -> bool {
        true
    }

    fn struct_at_return() -> (SomeStruct,) {
        (SomeStruct { data: 1 },)
    }
} 
