script;

use std::call::call_with_function_selector;
use std::bytes::Bytes;
use std::constants::BASE_ASSET_ID;

abi CalledContract {
    #[storage(read)]
    fn get_value() -> u8;
}

fn main(target: ContractId) -> u8 {
    let mut function_selector = Bytes::new();
    let mut calldata = Bytes::new();

    // https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/fn_selector_encoding.md#function-selector-encoding=
    // function selector is calculated as the first 4 bytes of sha256("set_value(u8)"), left padded to 8 bytes
    // hex : 00 00 00 00 8c  e9  59 49
    // dec : 00 00 00 00 140 233 89 73
    function_selector.push(0u8);
    function_selector.push(0u8);
    function_selector.push(0u8);
    function_selector.push(0u8);
    function_selector.push(140u8);
    function_selector.push(233u8);
    function_selector.push(89u8);
    function_selector.push(73u8);

    // https://github.com/FuelLabs/fuel-specs/blob/master/src/protocol/abi/argument_encoding.md
    // calldata is 42u8, left padded to 8 bytes
    // dec : 00 00 00 00 00 00 00 42
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
    called_contract.get_value()
}
