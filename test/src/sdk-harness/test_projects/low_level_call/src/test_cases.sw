library test_cases;

use std::bytes::Bytes;
use std::constants::BASE_ASSET_ID;
use std::hash::sha256;
use std::low_level_call::{call_with_function_selector, CallParams};

abi CalledContract {
    #[storage(read)]
    fn get_value() -> u64;
    fn get_b256_value() -> b256;
    fn get_str_value() -> str[4];
    fn get_bool_value() -> bool;
}

pub fn test_u64(target: ContractId) -> bool {

    // https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/fn_selector_encoding.md#function-selector-encoding=
    // sha256("set_value(u64)")
    // hex : 00 00 00 00 e0  ff  38 8f
    // dec : 00 00 00 00 224 255 56 143
    let function_selector_arr = [0u8, 0u8, 0u8, 0u8, 224u8, 255u8, 56u8, 143u8];
    let mut function_selector = Bytes::new();
    let mut i = 0;
    while i < 8 {
        function_selector.push(function_selector_arr[i]);
        i += 1;
    };

    // https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/argument_encoding.md
    // calldata is 42u64 (8 bytes)
    let calldata_arr= [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 42u8];
    let mut calldata = Bytes::new();
    i = 0;
    while i < 8 {
        calldata.push(calldata_arr[i]);
        i += 1;
    };
    
    // Calling "set_value(u64)" with argument "42" should set the value to 42
    call_with_function_selector(target, function_selector, calldata, CallParams{coins: 0, asset_id: BASE_ASSET_ID, gas: 2_000_000}, true);

    // Get value from called contract and return
    let called_contract = abi(CalledContract, target.into());
    let return_value = called_contract.get_value();

   return_value == 42

}

pub fn test_b256(target: ContractId) -> bool {
    // sha256("set_b256_value(b256)")
    // 0x6c 0x5e 0x2f 0xe2
    // 108 94 47 226
    let function_selector_arr = [0u8, 0u8, 0u8, 0u8, 108u8, 94u8, 47u8, 226u8];
    let mut function_selector = Bytes::new();
    let mut i = 0;
    while i < 8 {
        function_selector.push(function_selector_arr[i]);
        i += 1;
    };

    // 0x0101010101010101010101010101010101010101010101010101010101010101
    let mut calldata = Bytes::new();
    let mut i = 0;
    while i < 32 {
        calldata.push(1u8);
        i += 1;
    };

    call_with_function_selector(target, function_selector, calldata, CallParams{coins: 0, asset_id: BASE_ASSET_ID, gas: 2_000_000}, false);

    let called_contract = abi(CalledContract, target.into());
    let return_value = called_contract.get_b256_value();

    return_value == 0x0101010101010101010101010101010101010101010101010101010101010101
}

pub fn test_multiple_args_simple(target: ContractId) -> bool {

    let function_selector_arr = [0u8, 0u8, 0u8, 0u8, 112u8, 224u8, 73u8, 19u8];
    let mut function_selector = Bytes::new();
    let mut i = 0;
    while i < 8 {
        function_selector.push(function_selector_arr[i]);
        i += 1;
    };

    let calldata_arr= [
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 23u8,
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 42u8
        ];
    let mut calldata = Bytes::new();
    i = 0;
    while i < 16 {
        calldata.push(calldata_arr[i]);
        i += 1;
    };
    
    call_with_function_selector(target, function_selector, calldata, CallParams{coins: 0, asset_id: BASE_ASSET_ID, gas: 2_000_000}, false);

    // Get value from called contract and return
    let called_contract = abi(CalledContract, target.into());
    let return_value = called_contract.get_value();

   return_value == 65

}

pub fn test_multiple_args_complex(target: ContractId) -> bool {

    // sha256("set_value_multiple_complex(s(bool,a[u64;3]),str[4])") 
    // 0x62 0xc3 0x1a 0x4c
    // 00 00 00 00 98 195 26 76
    let function_selector_arr = [0u8, 0u8, 0u8, 0u8, 98u8, 195u8, 26u8, 76u8];
    
    let mut function_selector = Bytes::new();
    let mut i = 0;
    while i < 8 {
        function_selector.push(function_selector_arr[i]);
        i += 1;
    };

    let calldata_arr= [
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, // true for MyStruct.a
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, // 1u64 for MyStruct.b[0]
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 2u8, // 2u64 for MyStruct.b[1]
        0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 3u8, // 3u64 for MyStruct.b[2]
        102u8, 117u8, 101u8, 108u8, 0u8, 0u8, 0u8, 0u8 // "fuel" (0x6675656c) for str[4]  (note right padding)
        ];

    let mut calldata = Bytes::new();
    i = 0;
    while i < 40 {
        calldata.push(calldata_arr[i]);
        i += 1;
    };
    
    call_with_function_selector(target, function_selector, calldata, CallParams{coins: 0, asset_id: BASE_ASSET_ID, gas: 2_000_000}, false);

    // Get value from called contract and return
    let called_contract = abi(CalledContract, target.into());
    
    let return_uint = called_contract.get_value();
    let return_bool = called_contract.get_bool_value();
    let return_str = called_contract.get_str_value();
    
    return_uint == 2 && return_bool == true && sha256(return_str) == sha256("fuel")

}

