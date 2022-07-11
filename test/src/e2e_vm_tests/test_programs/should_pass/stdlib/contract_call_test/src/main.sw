script;

use std::contract_call::{contract_call, CallData};

abi Asset {
    fn mint_and_send_to_address(amount: u64, recipient: Identity);
}

fn main() {
    let call_data = CallData {
        arguments: 0,
        function_selector: 0,
        id: 0,
    }
    contract_call(call_data, 0, 0, 10000);
}
