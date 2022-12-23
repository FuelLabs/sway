contract;

abi MyContract {
    fn payable();
}

impl MyContract for Contract {
    // extra #[payable] attribute (not mentioned in the ABI)
    #[payable]
    fn payable() {
    }
}
