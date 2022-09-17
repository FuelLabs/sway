
//! A reentrancy check for use in Sway contracts.
//! Note that this only works in internal contexts.
//! to prevent reentrancy: `assert(!is_reentrant);
library reentrancy;

use ::assert::assert;
use ::context::{call_frames::*, registers::frame_ptr};

pub fn reentrancy_guard() {
    assert(!is_reentrant());
}

/// Returns `true` if the reentrancy pattern is detected, and `false` otherwise.
pub fn is_reentrant() -> bool {
    // Get our current contract ID
    let this_id = contract_id();

    // Reentrancy cannot occur in an external context. If not detected by the time we get to the
    // bottom of the call_frame stack, then no reentrancy has occured.
    let mut call_frame_pointer = if frame_ptr() == 0 {
        0
    } else {
        get_previous_frame_pointer(frame_ptr())
    };
    while call_frame_pointer != 0 {
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
