//! A reentrancy guard for use in Sway contracts.
//! Note that this only works in internal contexts.

library reentrancy;

use ::context::contract_id;
use ::auth::caller_is_external;
use ::option::*;
use ::constants::{SAVED_REGISTERS_OFFSET,CALL_FRAME_OFFSET};

/// Returns `true` if the reentrancy pattern is detected, and `false` otherwise.
pub fn is_reentrant() -> bool {
    let mut reentrancy = false;
    let mut internal = !caller_is_external();

    let mut call_frame_pointer = get_current_frame_pointer();

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

// get a pointer to the current call frame
fn get_current_frame_pointer() -> u64 {
    asm() {
        fp: u64
    };
}

// get a pointer to the previous (relative to the 'frame_pointer' param) call frame using offsets from a pointer.
fn get_previous_frame_pointer(frame_pointer: u64) -> u64 {
    let offset = SAVED_REGISTERS_OFFSET + CALL_FRAME_OFFSET;
    asm(res, ptr: frame_pointer, offset: offset) {
        add res ptr offset;
        res: u64
    }
}

// get the value of the previous `ContractId` from the previous call frame on the stack
fn get_previous_contract_id(previous_frame_ptr: u64) -> ContractId {
    ~ContractId::from(asm(res, ptr: previous_frame_ptr) {
        ptr: b256
    })
}
