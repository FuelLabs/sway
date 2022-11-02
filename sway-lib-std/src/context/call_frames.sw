//! Helper functions for accessing data from call frames.
//! Call frames store metadata across untrusted inter-contract calls:
//! https://fuellabs.github.io/fuel-specs/master/vm#call-frames
library call_frames;

use ::context::registers::frame_ptr;
use ::contract_id::ContractId;
use ::intrinsics::is_reference_type;

// Note that everything when serialized is padded to word length.
//
// Call Frame        :  saved registers offset         = 8
// Reserved Registers:  previous frame pointer offset  = 6
const SAVED_REGISTERS_OFFSET: u64 = 8;
const PREV_FRAME_POINTER_OFFSET: u64 = 6;
/// Where 73 is the current offset in words from the start of the call frame.
const FIRST_PARAMETER_OFFSET: u64 = 73;
/// Where 74 (73 + 1) is the current offset in words from the start of the call frame.
const SECOND_PARAMETER_OFFSET: u64 = 74;

///////////////////////////////////////////////////////////
//  Accessing the current call frame
///////////////////////////////////////////////////////////
/// Get the current contract's id when called in an internal context.
/// **Note !** If called in an external context, this will **not** return a contract ID.
// @dev If called externally, will actually return a pointer to the transaction ID.
pub fn contract_id() -> ContractId {
    ContractId::from(asm() { fp: b256 })
}

/// Get the asset_id of coins being sent from the current call frame.
pub fn msg_asset_id() -> ContractId {
    ContractId::from(asm(asset_id) {
        addi asset_id fp i32;
        asset_id: b256
    })
}

/// Get the code size in bytes (padded to word alignment) from the current call frame.
pub fn code_size() -> u64 {
    asm(size, ptr, offset: 576) {
        add size fp offset;
        size: u64
    }
}

/// Get the first parameter from the current call frame.
pub fn first_param() -> u64 {
    frame_ptr().add::<u64>(FIRST_PARAMETER_OFFSET).read()
}

/// Get the second parameter from the current call frame.
pub fn second_param<T>() -> T {
    if !is_reference_type::<T>() {
        frame_ptr().add::<u64>(SECOND_PARAMETER_OFFSET).read::<T>()
    } else {
        frame_ptr().add::<u64>(SECOND_PARAMETER_OFFSET).read::<raw_ptr>().read::<T>()
    }
}

///////////////////////////////////////////////////////////
//  Accessing arbitrary call frames by pointer
///////////////////////////////////////////////////////////
/// get a pointer to the previous (relative to the 'frame_pointer' param) call frame using offsets from a pointer.
pub fn get_previous_frame_pointer(frame_pointer: raw_ptr) -> raw_ptr {
    let offset = frame_pointer.add::<u64>(SAVED_REGISTERS_OFFSET + PREV_FRAME_POINTER_OFFSET);
    asm(res, ptr: offset) {
        lw res ptr i0;
        res: raw_ptr
    }
}

/// get the value of `ContractId` from any call frame on the stack.
pub fn get_contract_id_from_call_frame(frame_pointer: raw_ptr) -> ContractId {
    ContractId::from(asm(res, ptr: frame_pointer) { ptr: b256 })
}
