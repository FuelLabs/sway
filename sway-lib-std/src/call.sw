library call;

use ::bytes::Bytes;
use ::revert::require;
use ::contract_id::ContractId;

pub fn contract_id_to_bytes(contract_id: ContractId) -> Bytes {
    // Artificially create target bytes with capacity and len
    let mut target_bytes = Bytes::with_capacity(32);
    target_bytes.len = 32;

    // Copy bytes from contract_id into the buffer of the target bytes
    asm(r1: target_bytes.buf, r2: contract_id){
        mcpi r1 r2 i32;
    };

    target_bytes
}


// Call a target contract with an already-encoded payload
pub fn call_with_raw_payload(payload: Bytes, coins: u64, asset_id: ContractId, gas: u64) {
    asm(r1: payload, r2: coins, r3: asset_id, r4: gas) {
        call r1 r2 r3 r4;
    };
    // TODO : Should return the value return from the call ? 
}


// Enocode a payload from the function selection and calldata, and call the target contract
pub fn create_payload(target:ContractId, function_selector: Bytes, calldata: Bytes, coins: u64, asset_id: ContractId, gas: u64) -> Bytes {

    // packs args according to spec (https://github.com/FuelLabs/fuel-specs/blob/master/src/vm/instruction_set.md#call-call-contract) :
    /*
    bytes	type	    value	description
    32	    byte[32]	to	    Contract ID to call.
    8	    byte[8]	    param1	(Pointer to?) First parameter (function selector).
    8	    byte[8]	    param2	(Pointer to?) Second parameter (abi-encoded calldata).
    */

    require(function_selector.len() == 8, "function selector must be 8 bytes");

    let mut payload = Bytes::new();

    payload = payload.join(contract_id_to_bytes(target)); 
    payload = payload.join(function_selector);
    payload = payload.join(calldata);
    payload
}


pub fn call_with_function_selector(target: ContractId, function_selector: Bytes, calldata: Bytes, coins: u64, asset_id: ContractId, gas: u64) {

    let payload = create_payload(target, function_selector, calldata, coins, asset_id, gas);
    call_with_raw_payload(payload, coins, asset_id, gas);
    
}
