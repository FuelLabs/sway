library low_level_call;

use ::assert::assert;
use ::bytes::Bytes;
use ::contract_id::ContractId;
use ::option::Option;
use ::revert::require;
use ::vec::Vec;

pub struct CallParams {
    coins: u64,
    asset_id: ContractId,
    gas: u64,
}

// TODO : Replace with `from` when implemented
/// Represent a contract ID as a `Bytes`, so it can be concatenated with a payload.
fn contract_id_to_bytes(contract_id: ContractId) -> Bytes {
    let mut target_bytes = Bytes::with_capacity(32);
    target_bytes.len = 32;

    __addr_of(contract_id).copy_bytes_to(target_bytes.buf.ptr, 32);

    target_bytes
}

/// Represent a raw pointer as a `Bytes`, so it can be concatenated with a payload.
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
/// `payload` : The encoded payload to be called.
fn call_with_raw_payload(payload: Bytes, call_params: CallParams) {
    asm(r1: payload.buf.ptr, r2: call_params.coins, r3: call_params.asset_id, r4: call_params.gas) {
        call r1 r2 r3 r4;
    };
}

/// Encode a payload from the function selection and calldata.
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

    let mut payload = Bytes::new().join(contract_id_to_bytes(target)).join(function_selector);

    if (single_value_type_arg) {
        payload = payload.join(calldata); // When calldata is copy type, just pass calldata
    } else {
        payload = payload.join(ptr_as_bytes(calldata.buf.ptr)); // When calldata is reference type, need to get pointer as bytes
    };

    payload
}

/// Call a target contract with a function selector and calldata, provided as `Bytes`.
/// `target`               : The contract ID of the contract to be called.
/// `function_selector`    : The function selector of the function to be called, i.e. the first 8 bytes of `sha256("my_func(u64)")`.
/// `calldata`             : The encoded arguments with which to call the function.
/// `single_value_type_arg`: Whether the function being called takes a single value-type argument.
/// `call_params`          : The amount and color of coins to forward, and the gas to forward.
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
pub fn call_with_function_selector_vec(
    target: ContractId,
    function_selector: Vec<u8>,
    calldata: Vec<u8>,
    single_value_type_arg: bool,
    call_params: CallParams,
) {
    let mut function_selector = function_selector;
    let mut calldata = calldata;

    call_with_function_selector(target, Bytes::from_vec_u8(function_selector), Bytes::from_vec_u8(calldata), single_value_type_arg, call_params);
}
