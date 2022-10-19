contract;

use std::contract_id::ContractId;
use std::external::bytecode_root;

abi TestBytecodeContract {
    fn get_contract_bytecode_root(id: ContractId) -> b256;
}

impl TestBytecodeContract for Contract {
    fn get_contract_bytecode_root(id: ContractId) -> b256 {
        bytecode_root(id)
    }
}
