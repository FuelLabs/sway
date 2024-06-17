//! Functions to work with external contracts.
library;

use ::contract_id::ContractId;

/// Get the root of the bytecode of the contract at 'contract_id'.
///
/// # Arguments
///
/// * `contract_id`: [ContractId] - The contract of which the bytecode should be returned.
///
/// # Returns
///
/// * [b256] - The bytecode root of the contract.
///
/// # Examples
///
/// ```sway
/// use std::external::bytecode_root;
///
/// fn foo() {
///     let root_of_this_contract = bytecode_root(ContractId::this());
///     assert(root_of_this_contract != b256::zero());
/// }
/// ```
pub fn bytecode_root(contract_id: ContractId) -> b256 {
    let root = b256::zero();

    asm(root_addr: root, target: contract_id.bits()) {
        croo root_addr target;
        root_addr: b256
    }
}
