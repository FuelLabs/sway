contract;

abi MyContract {
    #[storage(read, write)]
    fn foo() -> u64;
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn foo() -> u64 {
        0
    }
}
