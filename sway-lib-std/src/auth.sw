//! Functionality for determining who is calling a contract.
library;

use ::address::Address;
use ::contract_id::ContractId;
use ::identity::Identity;
use ::option::Option::{self, *};
use ::result::Result::{self, *};
use ::inputs::{
    Input,
    input_coin_owner,
    input_count,
    input_message_recipient,
    input_message_sender,
    input_type,
};
use ::revert::revert;

/// The error type used when an `Identity` cannot be determined.
pub enum AuthError {
    /// The caller is external, but the inputs to the transaction are not all owned by the same address.
    InputsNotAllOwnedBySameAddress: (),
    /// The caller is internal, but the `caller_address` function was called.
    CallerIsInternal: (),
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
        gm r1 i1;
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
        gm r1 i2;
        r1: b256
    })
}

/// Get the `Identity` (i.e. `Address` or `ContractId`) from which a call was made.
/// Returns a `Ok(Identity)`, or `Err(AuthError)` if an identity cannot be determined.
///
/// # Additional Information
///
/// Returns an `AuthError::InputsNotAllOwnedBySameAddress` if the caller is external and the inputs to the transaction are not all owned by the same address.
/// Should not return an `AuthError::CallerIsInternal` under any circumstances.
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
///         Err(AuthError::CallerIsInternal) => log("Hell froze over."),
///     }
/// }
/// ```
pub fn msg_sender() -> Result<Identity, AuthError> {
    if caller_is_external() {
        match caller_address() {
            Err(err) => Err(err),
            Ok(owner) => Ok(Identity::Address(owner)),
        }
    } else {
        // Get caller's `ContractId`.
        Ok(Identity::ContractId(caller_contract_id()))
    }
}

/// Get the owner of the inputs (of type `Input::Coin` or `Input::Message`) to a
/// `TransactionScript` if they all share the same owner.
///
/// # Returns
///
/// * [Result<Address, AuthError>] - `Ok(Address)` if the owner can be determined, `Err(AuthError)` otherwise.
///
/// # Examples
///
/// ```sway
/// use std::auth::inputs_owner;
///
/// fn foo() {
///     match inputs_owner() {
///         Ok(address) => log(address),
///         Err(AuthError::InputsNotAllOwnedBySameAddress) => log("Inputs not all owned by same address."),
///     }
/// }
/// ```
pub fn caller_address() -> Result<Address, AuthError> {
    let inputs = input_count().as_u64();
    let mut candidate = None;
    let mut itter = 0;

    // Note: `inputs_count` is guaranteed to be at least 1 for any valid tx.
    while itter < inputs {
        let type_of_input = input_type(itter);
        match type_of_input {
            Some(Input::Coin) => (),
            Some(Input::Message) => (),
            _ => {
                // type != InputCoin or InputMessage, continue looping.
                itter += 1;
                continue;
            }
        }

        // type == InputCoin or InputMessage.
        let owner_of_input = match type_of_input {
            Some(Input::Coin) => {
                input_coin_owner(itter)
            },
            Some(Input::Message) => {
                input_message_sender(itter)
            },
            _ => {
                // type != InputCoin or InputMessage, continue looping.
                itter += 1;
                continue;
            }
        };

        if candidate.is_none() {
            // This is the first input seen of the correct type.
            candidate = owner_of_input;
            itter += 1;
            continue;
        }

        // Compare current input owner to candidate.
        // `candidate` and `input_coin_owner` must be `Some`.
        // at this point, so we can unwrap safely.
        if owner_of_input.unwrap() == candidate.unwrap() {
            // Owners are a match, continue looping.
            itter += 1;
            continue;
        }

        // Owners don't match. Return Err.
        return Err(AuthError::InputsNotAllOwnedBySameAddress);
    }

    // `candidate` must be `Some` if the caller is an address, otherwise it's a contract.
    match candidate {
        Some(address) => Ok(address),
        None => Err(AuthError::CallerIsInternal),
    }
}

/// Get the current predicate's address when called in an internal context.
///
/// # Returns
///
/// * [Option<Address>] - The address of this predicate.
///
/// # Reverts
///
/// * When called outside of a predicate program.
///
/// # Examples
///
/// ```sway
/// use std::auth::predicate_address;
///
/// fn main() {
///     let this_predicate = predicate_address();
///     log(this_predicate);
/// }
/// ```
pub fn predicate_address() -> Option<Address> {
    // Get index of current predicate.
    // i3 = GM_GET_VERIFYING_PREDICATE
    let predicate_index = asm(r1) {
        gm r1 i3;
        r1: u64
    };

    let type_of_input = input_type(predicate_index);

    match type_of_input {
        Some(Input::Coin) => input_coin_owner(predicate_index),
        Some(Input::Message) => input_message_recipient(predicate_index),
        _ => { None }
    }
}
