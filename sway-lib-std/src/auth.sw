//! Functionality for determining who is calling a contract.
library auth;

use ::address::Address;
use ::contract_id::ContractId;
use ::identity::Identity;
use ::option::Option;
use ::result::Result;
use ::inputs::{Input, input_count, input_owner, input_type};

pub enum AuthError {
    InputsNotAllOwnedBySameAddress: (),
}

/// Returns `true` if the caller is external (i.e. a script).
/// Otherwise, returns `false`.
/// ref: https://fuellabs.github.io/fuel-specs/master/vm/instruction_set#gm-get-metadata
pub fn caller_is_external() -> bool {
    asm(r1) {
        gm r1 i1;
        r1: bool
    }
}

/// If caller is internal, returns the contract ID of the caller.
/// Otherwise, undefined behavior.
pub fn caller_contract_id() -> ContractId {
    ContractId::from(asm(r1) {
        gm r1 i2;
        r1: b256
    })
}

/// Get the `Identity` (i.e. `Address` or `ContractId`) from which a call was made.
/// Returns a `Result::Ok(Identity)`, or `Result::Err(AuthError)` if an identity cannot be determined.
pub fn msg_sender() -> Result<Identity, AuthError> {
    if caller_is_external() {
        inputs_owner()
    } else {
        // Get caller's `ContractId`.
        Result::Ok(Identity::ContractId(caller_contract_id()))
    }
}

/// Get the owner of the inputs (of type `InputCoin` or `InputMessage`) to a
/// TransactionScript if they all share the same owner.
fn inputs_owner() -> Result<Identity, AuthError> {
    let inputs = input_count();
    let mut candidate = Option::None::<Address>();
    let mut i = 0u8;

    // Note: `inputs_count` is guaranteed to be at least 1 for any valid tx.
    while i < inputs {
        let type_of_input = input_type(i);
        match type_of_input {
            Input::Coin => (),
            Input::Message => (),
            _ => {
                // type != InputCoin or InputMessage, continue looping.
                i += 1u8;
                continue;
            }
        }

        // type == InputCoin or InputMessage
        let owner_of_input = input_owner(i);
        if candidate.is_none() {
            // This is the first input seen of the correct type.
            candidate = owner_of_input;
            i += 1u8;
            continue;
        }

        // Compare current input owner to candidate.
        // `candidate` and `input_owner` must be `Option::Some`
        // at this point, so we can unwrap safely.
        if owner_of_input.unwrap() == candidate.unwrap() {
            // Owners are a match, continue looping.
            i += 1u8;
            continue;
        }

        // Owners don't match. Return Err.
        return Result::Err(AuthError::InputsNotAllOwnedBySameAddress);
    }

    // `candidate` must be `Option::Some` at this point, so can unwrap safely.
    Result::Ok(Identity::Address(candidate.unwrap()))
}
