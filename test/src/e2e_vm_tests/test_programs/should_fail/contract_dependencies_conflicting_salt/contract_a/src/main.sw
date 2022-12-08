contract;

abi MyContract {
    fn test_function();
}

impl MyContract for Contract {
    fn test_function() {
        let contract_b_id = ContractId::from(contract_b::CONTRACT_B);
        let contract_c_id = ContractId::from(contract_c::CONTRACT_ID);
    }
}
