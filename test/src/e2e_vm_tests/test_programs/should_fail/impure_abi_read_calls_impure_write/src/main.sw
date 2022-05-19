contract;

abi MyContract {
    #[storage(read)]
    fn test_function() -> bool;
}

impl MyContract for Contract {
    #[storage(read)]
    fn test_function() -> bool {
        f()
    }
}

#[storage(write)]
fn f() -> bool {
    true
}
