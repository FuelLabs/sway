//! A reentrancy check for use in Sway contracts.
//! Note that this only works in internal contexts.
//! to prevent reentrancy: `assert(!is_reentrant);

library reentrancy;

use ::context::call_frames::*;
use ::constants::ZERO;
use ::assert::assert;
use ::panic::panic;
use ::chain::auth::caller_is_external;
use ::chain::log_u64;
use ::option::*;
use ::contract_id::ContractId;
use ::context::registers::frame_ptr;

pub fn reentrancy_guard() {
    assert(is_reentrant() == false);
}

/// Returns `true` if the reentrancy pattern is detected, and `false` otherwise.
pub fn is_reentrant() -> bool {
    let mut reentrancy = false;
    let mut call_frame_pointer = frame_ptr();
    // Get our current contract ID
    let this_id = contract_id();
    // initially, previous_contract_id == this_id
    let mut previous_contract_id = get_contract_id_from_call_frame(call_frame_pointer);
    let zero_id = ~ContractId::from(ZERO);

    // Seeing as reentrancy cannot happen in an external context, if not detected by the time we get to an external context in the stack then the reentrancy pattern is not present.
    while previous_contract_id != zero_id {
        if previous_contract_id == this_id {
            reentrancy = true;
            previous_contract_id = zero_id;
        } else {
            call_frame_pointer = get_previous_frame_pointer(call_frame_pointer);
            // get the ContractId value from the previous call frame
            previous_contract_id = get_contract_id_from_call_frame(call_frame_pointer);
        };
    }
    reentrancy
}
