//! Functions to work with external contracts.
library;

use ::constants::ZERO_B256;
use ::contract_id::ContractId;

/// Get the root of the bytecode of the contract at 'contract_id'.
///
/// # Arguments
///
/// * `contract_id`: [ContractId] - The contract of which the the bytecode should be returned.
///
/// # Returns
///
/// * [b256] - The bytecode root of the contract.
///
/// # Examples
///
/// ```sway
/// use std::{external::bytecode_root, call_frames::contract_id, constants::ZERO_B256};
///
/// fn foo() {
///     let root_of_this_contract = bytecode_root(contract_id());
///     assert(root_of_this_contract != ZERO_B256);
/// }
/// ```
pub fn bytecode_root(contract_id: ContractId) -> b256 {
    let root: b256 = ZERO_B256;

    asm(root_addr: root, target: contract_id.value) {
        croo root_addr target;
        root_addr: b256
    }
}
