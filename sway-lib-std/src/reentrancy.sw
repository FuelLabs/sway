//! A reentrancy check for use in Sway contracts.
//! Note that this only works in internal contexts.
//! to prevent reentrancy: `assert(!is_reentrant);

library reentrancy;

use ::context::call_frames::*;
use ::assert::assert;
use ::chain::auth::caller_is_external;
use ::option::*;
use ::contract_id::ContractId;
use ::context::registers::frame_ptr;


pub fn reentrancy_guard() {
    assert(!is_reentrant());
}

/// Returns `true` if the reentrancy pattern is detected, and `false` otherwise.
pub fn is_reentrant() -> bool {
    let mut reentrancy = false;
    let mut internal = !caller_is_external();

    let mut call_frame_pointer = frame_ptr();

    // Get our current contract ID
    let target_id = contract_id();

    // Seeing as reentrancy cannot happen in an external context, if not detectred by the time we get to an external context in the stack then the reentrancy pattern is not present.
    while internal {
        let previous_id = get_previous_contract_id(call_frame_pointer);

        if previous_id == target_id {
            reentrancy = true;
            internal = false;
        } else {
            internal = !caller_is_external();

            call_frame_pointer = get_previous_frame_pointer(call_frame_pointer);
        };
    }
    reentrancy
}
