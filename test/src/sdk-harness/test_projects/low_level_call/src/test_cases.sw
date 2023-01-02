library test_cases;

use std::low_level_call::call_with_function_selector;
use std::constants::BASE_ASSET_ID;
use std::bytes::Bytes;

abi CalledContract {
    #[storage(read)]
    fn get_value() -> u64;
    fn get_b256_value() -> b256;
}

pub fn test_u64(target: ContractId) -> bool {

    // https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/fn_selector_encoding.md#function-selector-encoding=
    // function selector is calculated as the first 4 bytes of sha256("set_value(u64)"), left padded to 8 bytes
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
    // calldata is 42u64, (8 bytes)
    let calldata_arr= [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 42u8];
    let mut calldata = Bytes::new();
    i = 0;
    while i < 8 {
        calldata.push(calldata_arr[i]);
        i += 1;
    };
    
    // Calling "set_value(u64)" with argument "42" should set the value to 42
    call_with_function_selector(target, function_selector, calldata, 0, BASE_ASSET_ID, 100_000);

    // Get value from called contract and return
    let called_contract = abi(CalledContract, target.into());
    let return_value = called_contract.get_value();

   return_value == 42

}

pub fn test_b256(target: ContractId) -> bool {

    // 6c 5e 2f e2
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

    call_with_function_selector(target, function_selector, calldata, 0, BASE_ASSET_ID, 2_000_000);

    let called_contract = abi(CalledContract, target.into());
    let return_value = called_contract.get_b256_value();

    return_value == 0x0101010101010101010101010101010101010101010101010101010101010101
}
