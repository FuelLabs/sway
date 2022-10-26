contract;

use std::contract_id::ContractId;
use std::external::{bytecode_root, bytecode_size, read_from_bytecode};
use std::hash::sha256;

abi ContractBytecodeTest {
    fn get_contract_bytecode_root(contract_id: ContractId) -> b256;
    fn get_contract_bytecode_size(contract_id: ContractId) -> u64;
    fn read_from_bytecode_and_hash(contract_id: ContractId, offset: u64, length: u64) -> b256;
}

impl ContractBytecodeTest for Contract {
    fn get_contract_bytecode_root(contract_id: ContractId) -> b256 {
        bytecode_root(contract_id)
    }

    fn get_contract_bytecode_size(contract_id: ContractId) -> u64 {
        bytecode_size(contract_id)
    }

    // Note: returning hash of bytes since returning `Vec` not yet supported: https://github.com/FuelLabs/fuels-rs/issues/602
    fn read_from_bytecode_and_hash(contract_id: ContractId, offset: u64, length: u64) -> b256 {
        let bytes = read_from_bytecode(contract_id, offset, length);
        sha256(bytes)
    }
}
