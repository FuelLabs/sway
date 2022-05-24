contract;

use array_of_structs_abi::{TestContract, Wrapper};

impl TestContract for Contract {
    fn return_array_of_structs(param: [Wrapper; 2]) -> [Wrapper; 2] {
        param
    }
}
