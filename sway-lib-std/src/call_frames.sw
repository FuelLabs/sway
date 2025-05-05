//! Helper functions for accessing data from call frames.
//! [Call frames](https://fuellabs.github.io/fuel-specs/master/vm#call-frames) store metadata across untrusted inter-contract calls.
library;

use ::asset_id::AssetId;
use ::contract_id::ContractId;
use ::intrinsics::is_reference_type;
use ::registers::frame_ptr;
use ::raw_ptr::*;
use ::codec::*;
use ::debug::*;

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

/// Get the `asset_id` of coins being sent from the current call frame.
///
/// # Returns
///
/// * [AssetId] - The asset included in the current call frame.
///
/// # Examples
///
/// ```sway
/// use std::call_frames::msg_asset_id;
///
/// fn foo() {
///     let asset = msg_asset_id();
///     assert(asset == AssetId::base());
/// }
/// ```
pub fn msg_asset_id() -> AssetId {
    AssetId::from(asm(asset_id) {
        addi asset_id fp i32;
        asset_id: b256
    })
}

/// Get the code size in bytes (padded to word alignment) from the current call frame.
///
/// # Additional Information
///
/// More information on data from call frames can be found in the Fuel Specs.
/// https://docs.fuel.network/docs/specs/fuel-vm/#call-frames
///
/// # Returns
///
/// * [u64] - The code size of the current call frame.
///
/// # Examples
///
/// ```sway
/// use std::call_frames::code_size;
///
/// fn foo() {
///     let size = code_size();
///     assert(size != 0);
/// }
/// ```
pub fn code_size() -> u64 {
    let ptr = asm(size, ptr, offset: 576) {
        add size fp offset;
        size: raw_ptr
    };
    ptr.read::<u64>()
}

/// Get the first parameter from the current call frame.
///
/// # Additional Information
///
/// More information on data from call frames can be found in the Fuel Specs.
/// https://docs.fuel.network/docs/specs/fuel-vm/#call-frames
///
/// # Returns
///
/// * [u64] - The first parameter of the current call frame.
///
/// # Examples
///
/// ```sway
/// use std::call_frames::first_param;
///
/// fn foo() {
///     let param = first_param();
///     assert(param != 0);
/// }
/// ```
pub fn first_param() -> u64 {
    frame_ptr().add::<u64>(FIRST_PARAMETER_OFFSET).read()
}

/// Get the second parameter from the current call frame.
///
/// # Additional Information
///
/// More information on data from call frames can be found in the Fuel Specs.
/// https://docs.fuel.network/docs/specs/fuel-vm/#call-frames
///
/// # Returns
///
/// * [u64] - The second parameter of the current call frame.
///
/// # Examples
///
/// ```sway
/// use std::call_frames::second_param;
///
/// fn foo() {
///     let param: u64 = second_param();
///     assert(param != 0);
/// }
/// ```
pub fn second_param() -> u64 {
    frame_ptr().add::<u64>(SECOND_PARAMETER_OFFSET).read()
}

/// Get the called method name from the current call frame.
pub fn called_method() -> str {
    use ::codec::decode_first_param;
    decode_first_param::<str>()
}

/// Get the called arguments from the current call frame.
pub fn called_args<T>() -> T
where
    T: AbiDecode,
{
    use ::codec::decode_second_param;
    decode_second_param::<T>()
}

//  Accessing arbitrary call frames by pointer
//
/// Get a pointer to the previous (relative to the `frame_pointer` parameter) call frame using offsets from a pointer.
///
/// # Additional Information
///
/// More information on data from call frames can be found in the Fuel Specs.
/// https://docs.fuel.network/docs/specs/fuel-vm/#call-frames
///
/// # Arguments
///
/// * `frame_pointer`: [raw_ptr] - The call frame reference directly before the returned call frame pointer.
///
/// # Returns
///
/// * [raw_ptr] - The memory location of the previous call frame data.
///
/// # Examples
///
/// ```sway
/// use std::{call_frames::get_previous_frame_pointer, registers::frame_ptr};
///
/// fn foo() {
///     let current_call_frame = frame_ptr();
///     let previous_call_frame = get_previous_frame_pointer(current_call_frame);
///     assert(!previous_call_frame.is_null());
/// }
/// ```
pub fn get_previous_frame_pointer(frame_pointer: raw_ptr) -> raw_ptr {
    let offset = frame_pointer.add::<u64>(SAVED_REGISTERS_OFFSET + PREV_FRAME_POINTER_OFFSET);
    asm(res, ptr: offset) {
        lw res ptr i0;
        res: raw_ptr
    }
}

/// Get the value of `ContractId` from any call frame on the stack.
///
/// # Arguments
///
/// * `frame_pointer`: [raw_ptr] - The call frame for which the Contract Id is to be returned.
///
/// # Returns
///
/// * [ContractId] - The Contract Id of for the call frame.
///
/// # Examples
///
/// ```sway
/// use std::{call_frames::get_contract_id_from_call_frame, registers::frame_ptr};
///
/// fn foo() {
///     let current_call_frame = frame_ptr();
///     let contract_id = get_contract_id_from_call_frame(current_call_frame);
/// }
/// ```
pub fn get_contract_id_from_call_frame(frame_pointer: raw_ptr) -> ContractId {
    ContractId::from(asm(res, ptr: frame_pointer) {
        ptr: b256
    })
}
