contract;

use std::address::Address;
use std::option::Option;
use std::tx::*;
use std::inputs::{
    Input,
    input_count,
    input_owner,
    input_pointer,
    input_type,
};
use std::outputs::{
    Output,
    output_count,
    output_pointer,
    output_type,
};

abi TxContractTest {
    fn get_tx_type() -> Transaction;
    fn get_tx_gas_price() -> u64;
    fn get_tx_gas_limit() -> u64;
    fn get_tx_maturity() -> u32;
    fn get_tx_script_length() -> u64;
    fn get_tx_script_data_length() -> u64;
    fn get_tx_inputs_count() -> u64;
    fn get_tx_outputs_count() -> u64;
    fn get_tx_witnesses_count() -> u64;
    fn get_tx_receipts_root() -> b256;
    fn get_tx_script_start_pointer() -> u64;

    fn get_input_type(index: u64) -> Input;
    fn get_tx_input_pointer(index: u64) -> u64;
    fn get_tx_input_coin_owner(index: u64) -> Address;

    fn get_tx_output_pointer(index: u64) -> u64;
    fn get_tx_output_type(ptr: u64) -> Output;
    fn get_tx_id() -> b256;
}

impl TxContractTest for Contract {
    fn get_tx_type() -> Transaction {
        tx_type()
    }
    fn get_tx_gas_price() -> u64 {
        tx_gas_price()
    }
    fn get_tx_gas_limit() -> u64 {
        tx_gas_limit()
    }
    fn get_tx_maturity() -> u32 {
        tx_maturity()
    }
    fn get_tx_script_length() -> u64 {
        tx_script_length()
    }
    fn get_tx_script_data_length() -> u64 {
        tx_script_data_length()
    }
    fn get_tx_inputs_count() -> u64 {
        input_count()
    }
    fn get_tx_outputs_count() -> u64 {
        output_count()
    }
    fn get_tx_witnesses_count() -> u64 {
        tx_witnesses_count()
    }
    fn get_tx_receipts_root() -> b256 {
        tx_receipts_root()
    }
    fn get_tx_script_start_pointer() -> u64 {
        tx_script_start_pointer()
    }
    fn get_tx_input_pointer(index: u64) -> u64 {
        input_pointer(index)
    }
    fn get_input_type(index: u64) -> Input {
        input_type(index)
    }
    // TODO: Add test for getting InputMessage owner when we have InputMessages
    // fn get_tx_input_message_owner(index: u64) -> Address {
    //     tx_input_owner(index)
    // }
    fn get_tx_input_coin_owner(index: u64) -> Address {
        input_owner(index).unwrap()
    }
    fn get_tx_output_pointer(index: u64) -> u64 {
        output_pointer(index)
    }
    fn get_tx_output_type(ptr: u64) -> Output {
        output_type(ptr)
    }
    fn get_tx_id() -> b256 {
        tx_id()
    }
}
