contract;

abi MyContract {
    #[storage(write)] // Or any other storage annotation
    fn foo() -> u64;
}

impl MyContract for Contract {
    #[storage(write)] // Or any other storage annotation
    fn foo() -> u64 {
        0
    }
}