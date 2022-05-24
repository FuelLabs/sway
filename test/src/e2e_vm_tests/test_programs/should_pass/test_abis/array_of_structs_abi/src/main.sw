library array_of_structs_abi;

pub struct Id {
    number: u64,
}

pub struct Wrapper {
    id: Id,
}

abi TestContract {
    fn return_array_of_structs(param: [Wrapper; 2]) -> [Wrapper; 2];
}
