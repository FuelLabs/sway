//! Utilities to help with low level calls.
library;

use ::assert::assert;
use ::bytes::Bytes;
use ::contract_id::{AssetId, ContractId};
use ::option::Option;
use ::revert::require;
use ::vec::Vec;

/// A struct representing the call parameters of a function call.
pub struct CallParams {
    /// Amount of the asset to transfer.
    coins: u64,
    /// AssetId of the asset to transfer.
    asset_id: AssetId,
    /// Gas to forward.
    gas: u64,
}

// TODO : Replace with `from` when implemented
/// Represent a contract ID as a `Bytes`, so it can be concatenated with a payload.
///
/// # Additional Information
///
/// It is recommended to use the `call_with_function_selector` function directly, unless you know what you are doing.
///
/// # Arguments
///
/// * `contract_id`: [ContractId] - The contract ID to be represented as a `Bytes`.
///
/// # Returns
///
/// * [Bytes] - The input ContractId represented as a `Bytes`.
///
/// # Examples
///
/// ```sway
/// use std::low_level_call::{contract_id_to_bytes, call_with_raw_payload, CallParams};
///
/// fn call_with_copy_type_arg(target: ContractId, function_selector: Bytes, calldata: Bytes, call_params: CallParams) {
///     let mut payload = Bytes::new();
///     payload.append(contract_id_to_bytes(target));
///     payload.append(function_selector);
///     payload.append(calldata);
///
///     call_with_raw_payload(payload, call_params);
/// }
/// ```
///
/// ```sway
/// use std::low_level_call::{bytes::Bytes, contract_id_to_bytes, call_with_raw_payload, CallParams, ptr_as_bytes};
///
/// fn call_with_reference_type_arg(target: ContractId, function_selector: Bytes, calldata: Bytes, call_params: CallParams) {
///     let mut payload = Bytes::new();
///     payload.append(contract_id_to_bytes(target));
///     payload.append(function_selector);
///     payload.append(ptr_as_bytes(calldata.buf.ptr));
///
///     call_with_raw_payload(payload, call_params);
/// }
/// ```
fn contract_id_to_bytes(contract_id: ContractId) -> Bytes {
    let mut target_bytes = Bytes::with_capacity(32);
    target_bytes.len = 32;

    __addr_of(contract_id).copy_bytes_to(target_bytes.buf.ptr, 32);

    target_bytes
}

/// Represent a raw pointer as a `Bytes`, so it can be concatenated with a payload.
///
/// # Additional Information
///
/// It is recommended to use the `call_with_function_selector` function directly, unless you know what you are doing.
///
/// # Arguments
///
/// * `ptr`: [raw_ptr] - The raw pointer to be represented as a `Bytes`.
///
/// # Returns
///
/// * [Bytes] - The input raw pointer represented as a `Bytes`.
///
/// # Examples
///
/// ```sway
/// use std::low_level_call::{bytes::Bytes, contract_id_to_bytes, call_with_raw_payload, CallParams, ptr_as_bytes};
///
/// fn call_with_reference_type_arg(target: ContractId, function_selector: Bytes, calldata: Bytes, call_params: CallParams) {
///     let mut payload = Bytes::new();
///     payload.append(contract_id_to_bytes(target));
///     payload.append(function_selector);
///     payload.append(ptr_as_bytes(calldata.buf.ptr));
///
///     call_with_raw_payload(payload, call_params);
/// }
/// ```
fn ptr_as_bytes(ptr: raw_ptr) -> Bytes {
    let mut bytes = Bytes::with_capacity(8);
    bytes.len = 8;

    // Need to copy pointer to heap so it has an address and can be copied onto the bytes buffer
    let mut ptr_on_heap = Vec::new();
    ptr_on_heap.push(ptr);
    ptr_on_heap.buf.ptr.copy_bytes_to(bytes.buf.ptr, 8);

    bytes
}

/// Call a target contract with an already-encoded payload.
///
/// # Additional Information
///
/// It is recommended to use the `call_with_function_selector` function directly, unless you know what you are doing.
///
/// The payload needs to be encoded according to the [Fuel VM specification](https://github.com/FuelLabs/fuel-specs/blob/master/src/vm/instruction_set.md#call-call-contract):
///
/// bytes   type        value   description
/// 32	    byte[32]    to      Contract ID to call.
/// 8	    byte[8]	    param1  First parameter (function selector).
/// 8	    byte[8]	    param2  Second parameter (abi-encoded calldata: value if value type, otherwise pointer to reference type).
///
/// # Arguments
///
/// * `payload` : [Bytes] - The encoded payload to be called.
/// * `call_params` : [CallParams] - The call parameters of the function call.
///
/// # Examples
///
/// ```sway
/// use std::low_level_call::{bytes::Bytes, contract_id_to_bytes, call_with_raw_payload, CallParams};
///
/// fn call_with_copy_type_arg(target: ContractId, function_selector: Bytes, calldata: Bytes, call_params: CallParams) {
///     let mut payload = Bytes::new();
///     payload.append(contract_id_to_bytes(target));
///     payload.append(function_selector);
///     payload.append(calldata);
///
///     call_with_raw_payload(payload, call_params);
/// }
/// ```
fn call_with_raw_payload(payload: Bytes, call_params: CallParams) {
    asm(r1: payload.buf.ptr, r2: call_params.coins, r3: call_params.asset_id, r4: call_params.gas) {
        call r1 r2 r3 r4;
    };
}

/// Encode a payload from the function selection and calldata.
///
/// # Additional Information
///
/// It is recommended to use the `call_with_function_selector` function directly, unless you know what you are doing.
///
/// # Arguments
///
/// * `target` : [ContractId] - The ContractId of the contract to be called.
/// * `function_selector` : [Bytes] - The function selector of the function to be called, i.e. the first 8 bytes of `sha256("my_func(u64)")`.
/// * `calldata` : [Bytes] - The encoded arguments with which to call the function.
/// * `single_value_type_arg` : [bool] - Whether the function being called takes a single value-type argument.
///
/// # Returns
///
/// * [Bytes] - The encoded payload.
///
/// # Examples
///
/// ```sway
/// use std::low_level_call::{bytes::Bytes, create_payload, call_with_raw_payload, CallParams};
///
/// fn call_contract(target: ContractId, function_selector: Bytes, calldata: Bytes, call_params: CallParams, single_value_type_arg: bool) {
///     let payload = create_payload(target, function_selector, calldata, single_value_type_arg);
///
///     call_with_raw_payload(payload, call_params);
/// }
/// ```
fn create_payload(
    target: ContractId,
    function_selector: Bytes,
    calldata: Bytes,
    single_value_type_arg: bool,
) -> Bytes {
    /*
    packs args according to spec (https://github.com/FuelLabs/fuel-specs/blob/master/src/vm/instruction_set.md#call-call-contract) :

    bytes   type        value   description
    32	    byte[32]    to      Contract ID to call.
    8	    byte[8]	    param1  First parameter (function selector).
    8	    byte[8]	    param2  Second parameter (abi-encoded calldata: value if value type, otherwise pointer to reference type).
    */
    require(function_selector.len() == 8, "function selector must be 8 bytes");

    // let mut payload = Bytes::new().append(contract_id_to_bytes(target)).append(function_selector);
    let mut payload = Bytes::new();
    payload.append(contract_id_to_bytes(target));
    payload.append(function_selector);

    if (single_value_type_arg) {
        payload.append(calldata); // When calldata is copy type, just pass calldata
    } else {
        payload.append(ptr_as_bytes(calldata.buf.ptr)); // When calldata is reference type, need to get pointer as bytes
    };

    payload
}

/// Call a target contract with a function selector and calldata, provided as `Bytes`.
///
/// # Arguments
///
/// * `target` : [ContractId] - The ContractId of the contract to be called.
/// * `function_selector` : [Bytes] - The function selector of the function to be called, i.e. the first 8 bytes of `sha256("my_func(u64)")`.
/// * `calldata` : [Bytes] - The encoded arguments with which to call the function.
/// * `single_value_type_arg` : [bool] - Whether the function being called takes a single value-type argument.
/// * `call_params` : [CallParams] - The amount and color of coins to forward, and the gas to forward.
///
/// # Examples
///
/// ```sway
/// use std::low_level_call::{bytes::Bytes, call_with_function_selector, CallParams};
///
/// fn call_contract(target: ContractId, function_selector: Bytes, calldata: Bytes, call_params: CallParams, single_value_type_arg: bool) {
///     call_with_function_selector(target, function_selector, calldata, single_value_type_arg, call_params);
/// }
/// ```
pub fn call_with_function_selector(
    target: ContractId,
    function_selector: Bytes,
    calldata: Bytes,
    single_value_type_arg: bool,
    call_params: CallParams,
) {
    let payload = create_payload(target, function_selector, calldata, single_value_type_arg);
    call_with_raw_payload(payload, call_params);
}

// TO DO: Deprecate when SDK supports Bytes
/// Call a target contract with a function selector and calldata, provided as `Vec<u8>`.
///
/// # Additional Information
///
/// It is recommended to use the `call_with_function_selector` function as this function will be deprecated in the future.
///
/// # Arguments
///
/// * `target` : [ContractId] - The ContractId of the contract to be called.
/// * `function_selector` : [Vec<u8>] - The function selector of the function to be called, i.e. the first 8 bytes of `sha256("my_func(u64)")`.
/// * `calldata` : [Vec<u8>] - The encoded arguments with which to call the function.
/// * `single_value_type_arg` : [bool] - Whether the function being called takes a single value-type argument.
/// * `call_params` : [CallParams] - The amount and color of coins to forward, and the gas to forward.
///
/// # Examples
///
/// ```sway
/// use std::low_level_call::{call_with_function_selector_vec, CallParams};
///
/// fn call_contract(target: ContractId, function_selector: Vec<u8>, calldata: Vec<u8>, call_params: CallParams, single_value_type_arg: bool) {
///     call_with_function_selector_vec(target, function_selector, calldata, single_value_type_arg, call_params);
/// }
/// ```
pub fn call_with_function_selector_vec(
    target: ContractId,
    function_selector: Vec<u8>,
    calldata: Vec<u8>,
    single_value_type_arg: bool,
    call_params: CallParams,
) {
    let mut function_selector = function_selector;
    let mut calldata = calldata;

    call_with_function_selector(target, Bytes::from(function_selector), Bytes::from(calldata), single_value_type_arg, call_params);
}
