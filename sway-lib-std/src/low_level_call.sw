library low_level_call;

use ::assert::assert;
use ::bytes::Bytes;
use ::revert::require;
use ::contract_id::ContractId;
use ::logging::log;
use ::option::Option;

pub struct CallParams {
    coins: u64,
    asset_id: ContractId,
    gas: u64,
}

// TODO : Replace with `pack` when implemented
fn contract_id_to_bytes(contract_id: ContractId) -> Bytes {

    let mut target_bytes = Bytes::with_capacity(32);
    target_bytes.len = 32;

    __addr_of(contract_id).copy_bytes_to(target_bytes.buf.ptr, 32);

    target_bytes
}



pub struct Pointer{
    value: raw_ptr,
}   

fn ptr_as_bytes(ptr: raw_ptr) -> Bytes {
    
    let ptr_in_struct = Pointer{value: ptr};

    let mut bytes = Bytes::with_capacity(8);
    bytes.len = 8;

    __addr_of(ptr_in_struct).copy_bytes_to(bytes.buf.ptr, 8);

    bytes
}


// Call a target contract with an already-encoded payload
fn call_with_raw_payload(payload: Bytes, coins: u64, asset_id: ContractId, gas: u64) {
    asm(r1: payload.buf.ptr, r2: coins, r3: asset_id, r4: gas) {
        call r1 r2 r3 r4;
    };
}


// Enocode a payload from the function selection and calldata, and call the target contract
fn create_payload(target: ContractId, function_selector: Bytes, calldata: Bytes, single_value_type_arg: bool) -> Bytes {

    // packs args according to spec (https://github.com/FuelLabs/fuel-specs/blob/master/src/vm/instruction_set.md#call-call-contract) :
    /*
    bytes	type	    value	description
    32	    byte[32]	to	    Contract ID to call.
    8	    byte[8]	    param1	First parameter (function selector).
    8	    byte[8]	    param2	Second parameter (abi-encoded calldata: value if value type, otherwise pointer to reference type).
    */

    require(function_selector.len() == 8, "function selector must be 8 bytes");


    let mut payload = Bytes::new()
    .join(contract_id_to_bytes(target))
    .join(function_selector);

    if (single_value_type_arg) {
        payload = payload.join(calldata); // When calldata is copy type, just pass calldata
    } else {
        payload = payload.join(ptr_as_bytes(calldata.buf.ptr)); // When calldata is reference type, need to get pointer as bytes
    };

    payload
}


pub fn call_with_function_selector(target: ContractId, function_selector: Bytes, calldata: Bytes, call_params: CallParams, single_value_type_arg: bool) {

    let payload = create_payload(target, function_selector, calldata, single_value_type_arg);
    call_with_raw_payload(payload, call_params.coins, call_params.asset_id, call_params.gas);
    
}
