contract;

abi MyContract {
    fn test_function() -> bool;
}

impl MyContract for Contract {
    fn test_function() -> bool {
        foo(ref Vec::new());
        true
    }
}

pub fn foo(ref mut vec: Vec<u64>) {
    vec.push(1);
}
