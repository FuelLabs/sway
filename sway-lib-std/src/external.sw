library external;

use ::constants::ZERO_B256;
use ::contract_id::ContractId;

/// Get the root of the bytecode of the contract at 'contract_id'.
pub fn bytecode_root(contract_id: ContractId) -> b256 {
    let root: b256 = ZERO_B256;

    asm(root: root, target: contract_id.value) {
        croo root target;
        root: b256
    }
}

/// Get the size (in bytes) of the bytecode of the contract at 'contract_id'.
pub fn bytecode_size(contract_id: ContractId) -> u64 {
    asm(size, target: contract_id.value) {
        csiz size target;
        size: u64
    }
}


// TO DO: b256 as proof of concept. Is there a way to make this generic? 
pub fn b256_from_bytecode(contract_id: ContractId, offset: u64) -> b256 {
    let result: b256 = ZERO_B256;

    asm(result: result, offset: offset, size: 32, target: contract_id.value) {
        ccp result target offset size;
        result: b256
    }
}
