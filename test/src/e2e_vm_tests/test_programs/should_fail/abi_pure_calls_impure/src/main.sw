contract;

abi MyContract {
    fn test_function() -> bool;
}

impl MyContract for Contract {
    fn test_function() -> bool {
        f()
    }
}

#[storage(read)]
fn f() -> bool {
    true
}
