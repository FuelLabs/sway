contract;

use std::contract_id::ContractId;
use std::contract_call::{call, CallData};

abi ContractCallTest {
    fn make_contract_call(call_data: CallData, amount: u64, asset: ContractId, gas: u64);
}

impl ContractCallTest for Contract {
    fn make_contract_call(call_data: CallData, amount: u64, asset: ContractId, gas: u64) {
        call(call_data, amount, asset, gas);
    }
}
