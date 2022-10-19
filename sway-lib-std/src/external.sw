library external;

use ::contract_id::ContractId;

/// Get the root of the bytecode of the contract at 'target'.
pub fn bytecode_root(target: ContractId) -> b256 {
    asm(root, contract_id: target.value) {
        croo root contract_id;
        root: b256
    }
}
