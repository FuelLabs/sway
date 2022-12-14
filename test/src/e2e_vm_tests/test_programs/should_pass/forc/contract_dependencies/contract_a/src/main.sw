contract;

use contract_b::CONTRACT_ID as CONTRACT_B_ID;
use contract_c::CONTRACT_ID as CONTRACT_C_ID;

abi MyContract {
    fn test_function();
}

impl MyContract for Contract {
    fn test_function() {
        let CONTRACT_B = CONTRACT_B_ID;
        let contract_b_id = ContractId::from(CONTRACT_B);
        let contract_c_id = ContractId::from(contract_c::CONTRACT_ID);
    }
}
