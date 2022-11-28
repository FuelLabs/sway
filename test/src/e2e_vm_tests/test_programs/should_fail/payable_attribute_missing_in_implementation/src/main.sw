contract;

abi MyContract {
    #[payable]
    fn payable();
}

impl MyContract for Contract {
    // missing #[payable] attribute
    fn payable() {
    }
}
