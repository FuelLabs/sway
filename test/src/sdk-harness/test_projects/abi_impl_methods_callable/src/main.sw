contract;

abi MyAbi {
    fn interface_method();
} {
    fn impl_method() -> u64 {
        42
    }
}

impl MyAbi for Contract {
    fn interface_method() {}
}
