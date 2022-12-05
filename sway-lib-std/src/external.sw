library external;

use ::constants::ZERO_B256;
use ::contract_id::ContractId;

/// Get the root of the bytecode of the contract at 'contract_id'.
pub fn bytecode_root(contract_id: ContractId) -> b256 {
    let root: b256 = ZERO_B256;

    asm(root_addr: root, target: contract_id.value) {
        croo root_addr target;
        root_addr: b256
    }
}
