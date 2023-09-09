//! Functionality for determining who is calling a contract.
library;

use ::address::Address;
use ::contract_id::ContractId;
use ::identity::Identity;
use ::option::Option::{*, self};
use ::result::Result::{*, self};
use ::inputs::{Input, input_count, input_owner, input_type};

/// The error type used when an `Identity` cannot be determined.
pub enum AuthError {
    /// The caller is external, but the inputs to the transaction are not all owned by the same address.
    InputsNotAllOwnedBySameAddress: (),
}

/// Returns `true` if the caller is external (i.e. a `script`).
/// Otherwise, if the caller is a contract, returns `false`.
///
/// # Additional Information
///
/// For more information refer to the [VM Instruction Set](https://fuellabs.github.io/fuel-specs/master/vm/instruction_set#gm-get-metadata).
///
/// # Returns
///
/// * [bool] - `true` if the caller is external, `false` otherwise.
///
/// # Examples
///
/// ```sway
/// use std::auth::caller_is_external;
///
/// fn foo() {
///     if caller_is_external() {
///         log("Caller is external.")
///     } else {
///         log("Caller is a contract.")
///     }
/// }
/// ```
pub fn caller_is_external() -> bool {
    asm(r1) {
        gm   r1 i1;
        r1: bool
    }
}

/// If the caller is internal, returns the contract ID of the caller.
///
/// # Additional Information
///
/// External calls result in undefined behaviour.
///
/// # Returns
///
/// * [ContractId] - The contract ID of the caller.
///
/// # Examples
///
/// ```sway
/// use std::auth::{caller_is_external, caller_contract_id};
///
/// fn foo() {
///     if !caller_is_external() {
///         let caller_contract_id = caller_contract_id();
///         log(caller_contract_id);
///     }
/// }
/// ```
pub fn caller_contract_id() -> ContractId {
    ContractId::from(asm(r1) {
        gm   r1 i2;
        r1: b256
    })
}

/// Get the `Identity` (i.e. `Address` or `ContractId`) from which a call was made.
/// Returns a `Ok(Identity)`, or `Err(AuthError)` if an identity cannot be determined.
///
/// # Additional Information
///
/// Returns a Err if the caller is external and the inputs to the transaction are not all owned by the same address.
///
/// # Returns
///
/// * [Result<Identity, AuthError>] - `Ok(Identity)` if the identity can be determined, `Err(AuthError)` otherwise.
///
/// # Examples
///
/// ```sway
/// fn foo() {
///     match msg_sender() {
///         Ok(Identity::Address(address)) => log(address),
///         Ok(Identity::ContractId(contract_id)) => log(contract_id),
///         Err(AuthError::InputsNotAllOwnedBySameAddress) => log("Inputs not all owned by same address."),
///     }
/// }
/// ```
pub fn msg_sender() -> Result<Identity, AuthError> {
    if caller_is_external() {
        inputs_owner()
    } else {
        // Get caller's `ContractId`.
        Ok(Identity::ContractId(caller_contract_id()))
    }
}

/// Get the owner of the inputs (of type `Input::Coin` or `Input::Message`) to a
/// `TransactionScript` if they all share the same owner.
///
/// # Additional Information
///
/// Will never return a Ok(Identity::ContractId).
///
/// # Returns
///
/// * [Result<Identity, AuthError>] - `Ok(Identity)` if the owner can be determined, `Err(AuthError)` otherwise.
///
/// # Examples
///
/// ```sway
/// use std::auth::inputs_owner;
///
/// fn foo() {
///     match inputs_owner() {
///         Ok(Identity::Address(address)) => log(address),
///         Ok(Identity::ContractId(_)) => log("Hell froze over."),
///         Err(AuthError::InputsNotAllOwnedBySameAddress) => log("Inputs not all owned by same address."),
///     }
/// }
/// ```
fn inputs_owner() -> Result<Identity, AuthError> {
    let inputs = input_count();
    let mut candidate = None;
    let mut i = 0u8;

    // Note: `inputs_count` is guaranteed to be at least 1 for any valid tx.
    while i < inputs {
        let type_of_input = input_type(i.as_u64());
        match type_of_input {
            Input::Coin => (),
            Input::Message => (),
            _ => {
                // type != InputCoin or InputMessage, continue looping.
                i += 1u8;
                continue;
            }
        }

        // type == InputCoin or InputMessage.
        let owner_of_input = input_owner(i.as_u64());
        if candidate.is_none() {
            // This is the first input seen of the correct type.
            candidate = owner_of_input;
            i += 1u8;
            continue;
        }

        // Compare current input owner to candidate.
        // `candidate` and `input_owner` must be `Some`.
        // at this point, so we can unwrap safely.
        if owner_of_input.unwrap() == candidate.unwrap() {
            // Owners are a match, continue looping.
            i += 1u8;
            continue;
        }

        // Owners don't match. Return Err.
        return Err(AuthError::InputsNotAllOwnedBySameAddress);
    }

    // `candidate` must be `Some` at this point, so can unwrap safely.
    Ok(Identity::Address(candidate.unwrap()))
}
