library external;

use ::contract_id::ContractId;

/// Get the root of the bytecode of the contract at 'contract_id'.
pub fn bytecode_root(contract_id: ContractId) -> b256 {

    asm(buffer, target: contract_id.value) {
        move buffer sp;
        cfei i32;
        croo buffer target;
        buffer: b256
    }
}
