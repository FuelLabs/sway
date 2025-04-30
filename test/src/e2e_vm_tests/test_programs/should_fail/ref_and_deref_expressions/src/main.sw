contract;

abi MyContract {
    fn test_function() -> bool;
}

impl MyContract for Contract {
    fn test_function() -> bool {
        foo(ref Vec::new()); // TODO: (REFERENCES) Improve  message for these cases (during the implementation of passing references to functions).
        true
    }
}

pub fn foo(ref mut vec: Vec<u64>) {
    vec.push(1);
}
