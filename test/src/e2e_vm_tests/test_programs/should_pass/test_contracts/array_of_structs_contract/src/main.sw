contract;

use array_of_structs_abi::{TestContract, Wrapper};

impl TestContract for Contract {
    fn return_array_of_structs(param: [Wrapper;
    2]) -> [Wrapper;
    2] {
        param
    }

    fn return_element_of_array_of_structs(param: [Wrapper;
    2]) -> Wrapper {
        param[0]
    }

    fn return_element_of_array_of_strings(param: [str[3];
    3]) -> str[3] {
        param[0]
    }
}
