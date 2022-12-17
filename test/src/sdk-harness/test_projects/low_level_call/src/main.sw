script;

use std::low_level_call::call_with_function_selector;
use std::bytes::Bytes;
use std::constants::BASE_ASSET_ID;

abi CalledContract {
    #[storage(read)]
    fn get_value() -> u64;
}

fn main(target: ContractId) -> bool {
    let mut function_selector = Bytes::new();
    let mut calldata = Bytes::new();

    // https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/fn_selector_encoding.md#function-selector-encoding=
    // function selector is calculated as the first 4 bytes of sha256("set_value(u64)"), left padded to 8 bytes
    // hex : 00 00 00 00 e0  ff  38 8f
    // dec : 00 00 00 00 224 255 56 143
    function_selector.push(0u8);
    function_selector.push(0u8);
    function_selector.push(0u8);
    function_selector.push(0u8);
    function_selector.push(224u8);
    function_selector.push(255u8);
    function_selector.push(56u8);
    function_selector.push(143u8);

    // https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/argument_encoding.md
    // calldata is 42u64, (8 bytes)
    calldata.push(0u8);
    calldata.push(0u8);
    calldata.push(0u8);
    calldata.push(0u8);
    calldata.push(0u8);
    calldata.push(0u8);
    calldata.push(0u8);
    calldata.push(42u8);

    // Calling "set_value(u64)" with argument "42" should set the value to 42
    call_with_function_selector(target, function_selector, calldata, 0, BASE_ASSET_ID, 100_000);

    // Get value from called contract and return
    let called_contract = abi(CalledContract, target.into());
    let return_value = called_contract.get_value();

    return_value == 42
}
