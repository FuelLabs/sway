contract;
use std::contract_id::ContractId;

abi MyContract {
    fn test_function() -> bool;
}

impl MyContract for Contract {
    fn test_function() -> bool {
        true
    }
}

fn caller(address: ContractId) -> ContractCaller<_> {
  abi(MyContract, address.value)
}
