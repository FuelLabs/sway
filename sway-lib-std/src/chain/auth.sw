library auth;
//! Functionality for determining who is calling a contract.

use ::address::Address;
use ::assert::assert;
use ::b512::B512;
use ::contract_id::ContractId;
use ::option::*;
use ::result::Result;
use ::tx::*;

pub enum AuthError {
    InputsNotAllOwnedBySameAddress: (),
}

pub enum Sender {
    Address: Address,
    ContractId: ContractId,
}

/// Returns `true` if the caller is external (i.e. a script).
/// Otherwise, returns `false`.
/// ref: https://github.com/FuelLabs/fuel-specs/blob/master/specs/vm/opcodes.md#gm-get-metadata
pub fn caller_is_external() -> bool {
    asm(r1) {
        gm r1 i1;
        r1: bool
    }
}

/// If caller is internal, returns the contract ID of the caller.
/// Otherwise, undefined behavior.
pub fn caller_contract_id() -> ContractId {
    ~ContractId::from(asm(r1) {
        gm r1 i2;
        r1: b256
    })
}

/// Get the `Sender` (i.e. `Address` or `ContractId`) from which a call was made.
/// Returns a `Result::Ok(Sender)`, or `Result::Err(AuthError)` if a sender cannot be determined.
pub fn msg_sender() -> Result<Sender, AuthError> {
    if caller_is_external() {
        let sender_res = get_coins_owner();
        if let Result::Ok(sender) = sender_res {
            Result::Ok(sender)
        } else {
            sender_res
        }
    } else {
        // Get caller's `ContractId`.
        Result::Ok(Sender::ContractId(caller_contract_id()))
    }
}

/// Get the owner of the inputs (of type `InputCoin`) to a TransactionScript,
/// if they all share the same owner.
fn get_coins_owner() -> Result<Sender, AuthError> {
    let target_input_type = 0u8;
    let inputs_count = tx_inputs_count();

    let mut candidate = Option::None::<Address>();
    let mut i = 0u64;

    while i < inputs_count {
        let input_pointer = tx_input_pointer(i);
        let input_type = tx_input_type(input_pointer);
        if input_type != target_input_type {
            // type != InputCoin
            // Continue looping.
            i = i + 1;
        } else {
            // type == InputCoin
            let input_owner = Option::Some(tx_input_coin_owner(input_pointer));
            if candidate.is_none() {
                // This is the first input seen of the correct type.
                candidate = input_owner;
                i = i + 1;
            } else {
                // Compare current coin owner to candidate.
                // `candidate` and `input_owner` must be `Option::Some` at this point,
                // so can unwrap safely.
                if input_owner.unwrap() == candidate.unwrap() {
                    // Owners are a match, continue looping.
                    i = i + 1;
                } else {
                    // Owners don't match. Return Err.
                    i = inputs_count;
                    return Result::Err(AuthError::InputsNotAllOwnedBySameAddress);
                };
            };
        };
    }

    // `candidate` must be `Option::Some` at this point, so can unwrap safely.
    // Note: `inputs_count` is guaranteed to be at least 1 for any valid tx.
    Result::Ok(Sender::Address(candidate.unwrap()))
}
