contract;

abi ContractIdTest {
    fn this_contract_id() -> ContractId;
}

impl ContractIdTest for Contract {
    fn this_contract_id() -> ContractId {
        ContractId::this()
    }
}

#[test]
fn contract_id_this() {
    let expected_contract_id = ContractId::from(CONTRACT_ID);
    let contract_abi = abi(ContractIdTest, expected_contract_id.bits());

    let result_contract_id = contract_abi.this_contract_id();
    assert(result_contract_id == expected_contract_id);
}
