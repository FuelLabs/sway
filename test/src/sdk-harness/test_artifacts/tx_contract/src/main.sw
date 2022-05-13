contract;

use std::address::Address;
use std::tx::*;

abi TxContractTest {
    fn get_tx_type() -> u8;
    fn get_tx_gas_price() -> u64;
    fn get_tx_gas_limit() -> u64;
    fn get_tx_byte_price() -> u64;
    fn get_tx_maturity() -> u32;
    fn get_tx_script_length() -> u64;
    fn get_tx_script_data_length() -> u64;
    fn get_tx_inputs_count() -> u64;
    fn get_tx_outputs_count() -> u64;
    fn get_tx_witnesses_count() -> u64;
    fn get_tx_receipts_root() -> b256;
    fn get_tx_script_start_offset() -> u64;

    fn get_tx_input_pointer(index: u64) -> u32;
    fn get_tx_input_type(ptr: u32) -> u8;
    fn get_tx_input_coin_owner(input_ptr: u32) -> Address;

    fn get_tx_output_pointer(index: u64) -> u32;
    fn get_tx_output_type(ptr: u32) -> u8;
}

impl TxContractTest for Contract {
    fn get_tx_type() -> u8 {
        tx_type()
    }
    fn get_tx_gas_price() -> u64 {
        tx_gas_price()
    }
    fn get_tx_gas_limit() -> u64 {
        tx_gas_limit()
    }
    fn get_tx_byte_price() -> u64 {
        tx_byte_price()
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
        tx_inputs_count()
    }
    fn get_tx_outputs_count() -> u64 {
        tx_outputs_count()
    }
    fn get_tx_witnesses_count() -> u64 {
        tx_witnesses_count()
    }
    fn get_tx_receipts_root() -> b256 {
        tx_receipts_root()
    }
    fn get_tx_script_start_offset() -> u64 {
        tx_script_start_offset()
    }

    fn get_tx_input_pointer(index: u64) -> u32 {
        tx_input_pointer(index)
    }
    fn get_tx_input_type(ptr: u32) -> u8 {
        tx_input_type(ptr)
    }
    fn get_tx_input_coin_owner(input_ptr: u32) -> Address {
        tx_input_coin_owner(input_ptr)
    }

    fn get_tx_output_pointer(index: u64) -> u32 {
        tx_output_pointer(index)
    }
    fn get_tx_output_type(ptr: u32) -> u8 {
        tx_output_type(ptr)
    }
}
