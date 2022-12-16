library low_level_call;

use ::bytes::Bytes;
use ::revert::require;
use ::contract_id::ContractId;
use ::logging::log;
use ::option::Option;

fn contract_id_to_bytes(contract_id: ContractId) -> Bytes {
    // Artificially create target bytes with capacity and len
    let mut target_bytes = Bytes::with_capacity(32);
    target_bytes.len = 32;

    __addr_of(contract_id).copy_bytes_to(target_bytes.buf.ptr, 32);

    target_bytes
}

fn memory_address_as_bytes<T>(ref_type: T) -> Bytes {

    let mut bytes = Bytes::with_capacity(8);
    bytes.len = 8;

    let address_as_u64 = asm(r1: __addr_of(ref_type)) {r1: u64};

    asm(r1: bytes.buf.ptr, r2: address_as_u64){
        mcpi r1 r2 i8;
    };

    bytes
}


// Call a target contract with an already-encoded payload
fn call_with_raw_payload(payload: Bytes, coins: u64, asset_id: ContractId, gas: u64) {
    asm(r1: __addr_of(payload), r2: coins, r3: asset_id, r4: gas) {
        call r1 r2 r3 r4;
    };
    // TODO : Should return the value return from the call ? 
}


// Enocode a payload from the function selection and calldata, and call the target contract
fn create_payload(target: ContractId, function_selector: Bytes, calldata: Bytes, coins: u64, asset_id: ContractId, gas: u64) -> Bytes {

    // packs args according to spec (https://github.com/FuelLabs/fuel-specs/blob/master/src/vm/instruction_set.md#call-call-contract) :
    /*
    bytes	type	    value	description
    32	    byte[32]	to	    Contract ID to call.
    8	    byte[8]	    param1	First parameter (function selector).
    8	    byte[8]	    param2	Second parameter (pointer to abi-encoded calldata).
    */

    require(function_selector.len() == 8, "function selector must be 8 bytes");

    let mut payload = Bytes::new();

    payload = payload.join(contract_id_to_bytes(target)); 
    payload = payload.join(function_selector);
    payload = payload.join(memory_address_as_bytes(calldata));

    /*
    // TODO DEBUG
    // Log each byte to check packing
    let mut i = 0;
    while i < payload.len() {
        log(payload.get(i).unwrap());
        i = i + 1;
    };
    */

    payload
}


pub fn call_with_function_selector(target: ContractId, function_selector: Bytes, calldata: Bytes, coins: u64, asset_id: ContractId, gas: u64) {

    let payload_ptr = create_payload(target, function_selector, calldata, coins, asset_id, gas);
    call_with_raw_payload(payload_ptr, coins, asset_id, gas);
    
}
