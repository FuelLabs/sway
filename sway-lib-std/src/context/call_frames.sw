//! Helper functions for accessing data from call frames.
/// Call frames store metadata across untrusted inter-contract calls:
/// https://github.com/FuelLabs/fuel-specs/blob/master/specs/vm/main.md#call-frames
library call_frames;

use ::contract_id::ContractId;

// Note that everything when serialized is padded to word length.
//
// Call Frame        :  saved registers offset         = 8*WORD_SIZE = 8*8 = 64
// Reserved Registers:  previous frame pointer offset  = 6*WORD_SIZE = 6*8 = 48

const SAVED_REGISTERS_OFFSET: u64 = 64;
const PREV_FRAME_POINTER_OFFSET: u64 = 48;

///////////////////////////////////////////////////////////
//  Accessing the current call frame
///////////////////////////////////////////////////////////

/// Get the current contract's id when called in an internal context.
/// **Note !** If called in an external context, this will **not** return a contract ID.
// @dev If called externally, will actually return a pointer to the transaction ID.
pub fn contract_id() -> ContractId {
    ~ContractId::from(asm() {
        fp: b256
    })
}

/// Get the asset_id of coins being sent from the current call frame.
pub fn msg_asset_id() -> ContractId {
    ~ContractId::from(asm(asset_id) {
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
    asm(size, ptr, offset: 584) {
        add size fp offset;
        size: u64
    }
}

/// Get the second parameter from the current call frame.
pub fn second_param() -> u64 {
    asm(size, ptr, offset: 592) {
        add size fp offset;
        size: u64
    }
}

///////////////////////////////////////////////////////////
//  Accessing arbitrary call frames by pointer
///////////////////////////////////////////////////////////

/// get a pointer to the previous (relative to the 'frame_pointer' param) call frame using offsets from a pointer.
pub fn get_previous_frame_pointer(frame_pointer: u64) -> u64 {
    let offset = frame_pointer + SAVED_REGISTERS_OFFSET + PREV_FRAME_POINTER_OFFSET;
    asm(res, ptr: offset) {
        lw res ptr i0;
        res: u64
    }
}

/// get the value of `ContractId` from any call frame on the stack.
pub fn get_contract_id_from_call_frame(frame_pointer: u64) -> ContractId {
    ~ContractId::from(asm(res, ptr: frame_pointer) {
        ptr: b256
    })
}
