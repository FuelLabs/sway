contract;

abi MyContract {
    fn foo() -> u64;
}

impl MyContract for Contract {
    fn foo() -> u64 {
        42
    }
}
