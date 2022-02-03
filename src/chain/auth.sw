library auth;
//! Functionality for determining who is calling an ABI method

use ::result::Result;
use ::address::Address;
use ::contract_id::ContractId;

// TODO: use an enum instead of loose contants for these once match statements work with enum.
/// tracked here: https://github.com/FuelLabs/sway/issues/579
const IS_CALLER_EXTERNAL = 1;
const GET_CALLER = 2;

pub enum AuthError {
    ContextError: (),
}

pub enum Sender {
    Address: Address,
    Id: ContractId,
}

/// Returns `true` if the caller is external (ie: a script or predicate).
pub fn caller_is_external() -> bool {
    asm(r1, r2: IS_CALLER_EXTERNAL) {
        gm r1 r2;
        r1: bool
    }
}

/// Get the `Sender` (ie: `Address`| ContractId) from which a call was made.
/// Returns a Result::Ok(Sender) or Result::Error.
// NOTE: Currently only returns Result::Ok variant if the parent context is Internal.
pub fn msg_sender() -> Result<Sender, AuthError> {
    if caller_is_external() {
        // TODO: Add call to get_coins_owner() here when implemented,
        Result::Err(AuthError::ContextError)
    } else {
        // Get caller's contract ID
        let id = ~ContractId::from(asm(r1, r2: GET_CALLER) {
            gm r1 r2;
            r1: b256
        });
        Result::Ok(Sender::Id(id))
    }
}
