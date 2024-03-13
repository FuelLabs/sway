contract;

storage {
    b: b256 = std::constants::ZERO_B256,
}

abi MyContract {
    #[storage(write)]
    fn test_function(b: ContractId) -> bool;
}

impl MyContract for Contract {
    #[storage(write)]
    fn test_function(b: ContractId) -> bool {
        storage.b.write(b.into());

        true
    }
}