script;

use std::low_level_call::{call_with_function_selector_vec, CallParams};

fn main(
    target: ContractId,
    function_selector: Vec<u8>,
    calldata: Vec<u8>,
    single_value_type_arg: bool,
) {
    let call_params = CallParams {
        coins: 0,
        asset_id: AssetId::base(),
        gas: 10_000_000,
    };

    call_with_function_selector_vec(target, function_selector, calldata, single_value_type_arg, call_params);
}
