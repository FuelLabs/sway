contract;

use std::contract_id::ContractId;
use std::external::{bytecode_root, bytecode_size, b256_from_bytecode};

abi ContractBytecodeTest {
    fn get_contract_bytecode_root(contract_id: ContractId) -> b256;
    fn get_contract_bytecode_size(contract_id: ContractId) -> u64;
    fn get_b256_from_bytecode(pointer: u64, contract_id: ContractId) -> b256;
}

impl ContractBytecodeTest for Contract {
    fn get_contract_bytecode_root(contract_id: ContractId) -> b256 {
        bytecode_root(contract_id)
    }

    fn get_contract_bytecode_size(contract_id: ContractId) -> u64 {
        bytecode_size(contract_id)
    }

    fn get_b256_from_bytecode(pointer: u64, contract_id: ContractId) -> b256 {
        b256_from_bytecode(pointer, contract_id)
    }
}
