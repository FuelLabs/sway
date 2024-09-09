script;

use std::bytes::Bytes;
use std::low_level_call::{call_with_function_selector, CallParams};

fn main(
    target: ContractId,
    function_selector: Bytes,
    calldata: Bytes,
    single_value_type_arg: bool,
) {
    let call_params = CallParams {
        coins: 0,
        asset_id: AssetId::base(),
        gas: 10_000_000,
    };

    call_with_function_selector(
        target,
        function_selector,
        calldata,
        single_value_type_arg,
        call_params,
    );
}
