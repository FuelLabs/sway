contract;

use std::{
    b512::B512,
    bytes::Bytes,
    inputs::{
        Input,
        input_amount,
        input_count,
        input_message_data,
        input_message_data_length,
        input_message_msg_id,
        input_message_nonce,
        input_message_recipient,
        input_message_sender,
        input_owner,
        input_pointer,
        input_predicate,
        input_predicate_data_length,
        input_predicate_data_pointer,
        input_predicate_length,
        input_type,
        input_witness_index,
    },
    logging::log,
    outputs::{
        Output,
        output_amount,
        output_count,
        output_pointer,
        output_type,
    },
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
        tx_witness_data,
        tx_witness_data_length,
        tx_witness_pointer,
        tx_witnesses_count,
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
    fn get_input_message_data(index: u64, offset: u64, expected: [u8; 3]) -> bool;
    fn get_input_predicate(index: u64, bytecode: Vec<u8>) -> bool;

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
        asm(ptr: tx_script_start_pointer()) { ptr: u64 }
    }
    fn get_tx_script_data_start_pointer() -> u64 {
        asm(r1: tx_script_data_start_pointer()) { r1: u64 }
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
        asm(r1: input_predicate_data_pointer(index).unwrap()) { r1: u64 }
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
    fn get_input_message_data(index: u64, offset: u64, expected: [u8; 3]) -> bool {
        let data = input_message_data::<[u8; 3]>(index, offset);
        let mut data_bytes = Bytes::new();
        let mut expected_data_bytes = Bytes::new();
        log(data);
        log(data[0]);
        log(expected[0]);
        log(data[1]);
        log(expected[1]);
        log(data[2]);
        log(expected[2]);

        // data_bytes.push(data[0]);
        // data_bytes.push(data[1]);
        // data_bytes.push(data[2]);
        // expected_data_bytes.push(expected[0]);
        // expected_data_bytes.push(expected[1]);
        // expected_data_bytes.push(expected[2]);
        // log(data);
        // log(data_bytes);
        // log(expected);
        // log(expected_data_bytes);
        // data_bytes == expected_data_bytes
        data[0] == expected[0] && data[1] == expected[1] && data[2] == expected[2]
    }
    fn get_input_predicate(index: u64, bytecode: Vec<u8>) -> bool {
        let code = input_predicate::<Vec<u8>>(index);
        assert(input_predicate_length(index).unwrap() == bytecode.len());
        let mut i = 0;
        while i < bytecode.len() {
            assert(bytecode.get(i).unwrap() == code.get(i).unwrap());
            i += 1;
        }
        true

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
