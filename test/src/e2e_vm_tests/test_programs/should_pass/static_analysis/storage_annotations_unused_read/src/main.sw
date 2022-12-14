contract;

abi MyContract {
    #[storage(read)]
    fn foo() -> u64;
}

impl MyContract for Contract {
    #[storage(read)]
    fn foo() -> u64 {
        0
    }
}
