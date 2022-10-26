library external;

use ::constants::ZERO_B256;
use ::contract_id::ContractId;
use ::intrinsics::size_of;
use ::vec::Vec;
use ::alloc::alloc;
use ::assert::assert;

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

/// Read `length` bytes from `offset` of the bytecode of the contract at 'contract_id' into a Vec
pub fn read_from_bytecode(contract_id: ContractId, offset: u64, length: u64) -> Vec<u8> {
    
    // Create a Vec large enough to contain the bytes to be copied
    let result: Vec<u8> = ~Vec::with_capacity(length);

    // Copy bytes from the target contract into memory, starting from the Vec's internal RawVec pointer
    asm(ptr: result.buf.ptr, offset: offset, length: length, target: contract_id.value) {
        ccp ptr target offset length;
    };

    result
}
