//! A reentrancy check for use in Sway contracts.
//! Note that this only works in internal contexts.
//! To prevent reentrancy: `assert(!is_reentrant());`.
library reentrancy;

use ::assert::assert;
use ::call_frames::*;
use ::registers::frame_ptr;

/// Reverts if the reentrancy pattern is detected in the contract in which this is called.
/// Not needed if the Checks-Effects-Interactions (CEI) pattern is followed (as prompted by the compiler).
/// Caution: While this can protect against both single-function reentrancy and cross-function reentrancy attacks, it WILL NOT PREVENT a cross-contract reentrancy attack.
pub fn reentrancy_guard() {
    assert(!is_reentrant());
}

/// Returns `true` if the reentrancy pattern is detected, and `false` otherwise.
pub fn is_reentrant() -> bool {
    // Get our current contract ID
    let this_id = contract_id();

    // Reentrancy cannot occur in an external context. If not detected by the time we get to the
    // bottom of the call_frame stack, then no reentrancy has occured.
    let mut call_frame_pointer = frame_ptr();
    if !call_frame_pointer.is_null() {
        call_frame_pointer = get_previous_frame_pointer(call_frame_pointer);
    };
    while !call_frame_pointer.is_null() {
        // get the ContractId value from the previous call frame
        let previous_contract_id = get_contract_id_from_call_frame(call_frame_pointer);
        if previous_contract_id == this_id {
            return true;
        };
        call_frame_pointer = get_previous_frame_pointer(call_frame_pointer);
    }

    // The current contract ID wasn't found in any contract calls prior to here.
    false
}
