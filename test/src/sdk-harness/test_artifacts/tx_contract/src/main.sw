contract;

use std::{
    address::Address,
    b512::B512,
    inputs::{
        Input,
        input_amount,
        input_owner,
        input_count,
        input_message_data,
        input_message_data_length,
        input_message_msg_id,
        input_message_nonce,
        input_message_predicate,
        input_predicate_length,
        input_predicate_data_length,
        input_message_recipient,
        input_message_sender,
        input_witness_index,
        input_pointer,
        input_predicate_data_pointer,
        input_type,
    },
    option::Option,
    outputs::{
        Output,
        output_count,
        output_amount,
        output_pointer,
        output_type,
    },
    revert::revert,
    tx::{
        Transaction,
        tx_gas_limit,
        tx_gas_price,
        tx_id,
        tx_maturity,
        tx_receipts_root,
        tx_script_bytecode_hash,
        tx_script_data_length,
        tx_script_data_start_pointer,
        tx_script_length,

        tx_script_start_pointer,
        tx_type,
        tx_witnesses_count,
        tx_witness_data,
        tx_witness_data_length,
        tx_witness_pointer,
    },
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
    fn get_tx_witness_pointer(index: u64) -> u64;
    fn get_tx_witness_data_length(index: u64) -> u64;
    fn get_tx_witness_data(index: u64) -> B512;
    fn get_tx_receipts_root() -> b256;
    fn get_tx_script_start_pointer() -> u64;
    fn get_tx_script_data_start_pointer() -> u64;
    fn get_tx_id() -> b256;
    fn get_tx_script_bytecode_hash() -> b256;

    fn get_input_type(index: u64) -> Input;
    fn get_tx_input_pointer(index: u64) -> u64;
    fn get_input_owner(index: u64) -> Address;
    fn get_input_amount(index: u64) -> u64;
    fn get_tx_input_predicate_data_pointer(index: u64) -> u64;
    fn get_input_message_msg_id(index: u64) -> b256;
    fn get_input_message_sender(index: u64) -> Address;
    fn get_input_message_recipient(index: u64) -> Address;
    fn get_input_message_nonce(index: u64) -> u64;
    fn get_input_witness_index(index: u64) -> u8;
    fn get_input_message_data_length(index: u64) -> u16;
    fn get_input_predicate_length(index: u64) -> u16;
    fn get_input_predicate_data_length(index: u64) -> u16;
    fn get_input_message_data(index: u64, offset: u64) -> u64;
    fn get_input_message_predicate(index: u64) -> u64;

    fn get_tx_output_pointer(index: u64) -> u64;
    fn get_output_type(ptr: u64) -> Output;
    fn get_tx_output_amount(index: u64) -> u64;
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
    fn get_tx_witness_pointer(index: u64) -> u64 {
        tx_witness_pointer(index)
    }
    fn get_tx_witness_data_length(index: u64) -> u64 {
        tx_witness_data_length(index)
    }
    fn get_tx_witness_data(index: u64) -> B512 {
        tx_witness_data(index)
    }
    fn get_tx_receipts_root() -> b256 {
        tx_receipts_root()
    }
    fn get_tx_script_start_pointer() -> u64 {
        tx_script_start_pointer()
    }
    fn get_tx_script_data_start_pointer() -> u64 {
        tx_script_data_start_pointer()
    }
    fn get_tx_id() -> b256 {
        tx_id()
    }
    fn get_tx_script_bytecode_hash() -> b256 {
        tx_script_bytecode_hash()
    }
    fn get_tx_input_pointer(index: u64) -> u64 {
        input_pointer(index)
    }
    fn get_input_type(index: u64) -> Input {
        input_type(index)
    }
    fn get_input_owner(index: u64) -> Address {
        input_owner(index).unwrap()
    }
    fn get_input_amount(index: u64) -> u64 {
        input_amount(index).unwrap()
    }
    fn get_tx_input_predicate_data_pointer(index: u64) -> u64 {
        input_predicate_data_pointer(index).unwrap()
    }
    fn get_input_message_msg_id(index: u64) -> b256 {
        input_message_msg_id(index)
    }
    fn get_input_message_sender(index: u64) -> Address {
        input_message_sender(index)
    }
    fn get_input_message_recipient(index: u64) -> Address {
        input_message_recipient(index)
    }
    fn get_input_message_nonce(index: u64) -> u64 {
        input_message_nonce(index)
    }
    fn get_input_witness_index(index: u64) -> u8 {
        input_witness_index(index).unwrap()
    }
    fn get_input_message_data_length(index: u64) -> u16 {
        input_message_data_length(index)
    }
    fn get_input_predicate_length(index: u64) -> u16 {
        input_predicate_length(index).unwrap()
    }
    fn get_input_predicate_data_length(index: u64) -> u16 {
        input_predicate_data_length(index).unwrap()
    }
    // @review return type ! (lib func is generic)
    fn get_input_message_data(index: u64, offset: u64) -> u64 {
        input_message_data::<u64>(index, offset)
    }
    // @review return type ! (lib func is generic)
    fn get_input_message_predicate(index: u64) -> u64 {
        input_message_predicate::<u64>(index)
    }
    fn get_tx_output_pointer(index: u64) -> u64 {
        output_pointer(index)
    }
    fn get_output_type(ptr: u64) -> Output {
        output_type(ptr)
    }
    fn get_tx_output_amount(index: u64) -> u64 {
        output_amount(index)
    }
}
