contract;

use std::{
    b512::B512,
    bytes::Bytes,
    inputs::*,
    outputs::{
        Output,
        output_amount,
        output_count,
        output_type,
    },
    tx::*,
};

abi TxContractTest {
    fn get_tx_type() -> Transaction;
    fn get_tx_tip() -> Option<u64>;
    fn get_script_gas_limit() -> u64;
    fn get_tx_maturity() -> Option<u32>;
    fn get_tx_witness_limit() -> Option<u64>;
    fn get_tx_max_fee() -> Option<u64>;
    fn get_tx_expiration() -> Option<u32>;
    fn get_tx_script_length() -> Option<u64>;
    fn get_tx_script_data_length() -> Option<u64>;
    fn get_tx_inputs_count() -> u64;
    fn get_tx_outputs_count() -> u16;
    fn get_tx_witnesses_count() -> u64;
    fn get_tx_witness_data_length(index: u64) -> Option<u64>;
    fn get_tx_witness_data(index: u64) -> Option<B512>;
    fn get_tx_id() -> b256;
    fn get_tx_script_bytecode_hash() -> Option<b256>;

    fn get_input_type(index: u64) -> Option<Input>;
    fn get_input_coin_owner(index: u64) -> Option<Address>;
    fn get_input_amount(index: u64) -> Option<u64>;
    fn get_input_message_sender(index: u64) -> Option<Address>;
    fn get_input_message_recipient(index: u64) -> Option<Address>;
    fn get_input_message_nonce(index: u64) -> Option<b256>;
    fn get_input_witness_index(index: u64) -> Option<u16>;
    fn get_input_message_data_length(index: u64) -> Option<u64>;
    fn get_input_predicate_length(index: u64) -> Option<u64>;
    fn get_input_predicate_data_length(index: u64) -> Option<u64>;
    fn get_input_message_data(index: u64, offset: u64, expected_data_bytes: Bytes) -> bool;
    fn get_input_predicate(index: u64, bytecode: Vec<u8>) -> bool;

    fn get_output_type(ptr: u64) -> Option<Output>;
    fn get_tx_output_amount(index: u64) -> Option<u64>;
}

impl TxContractTest for Contract {
    fn get_tx_type() -> Transaction {
        tx_type()
    }
    fn get_tx_tip() -> Option<u64> {
        tx_tip()
    }
    fn get_script_gas_limit() -> u64 {
        script_gas_limit()
    }
    fn get_tx_maturity() -> Option<u32> {
        tx_maturity()
    }
    fn get_tx_witness_limit() -> Option<u64> {
        tx_witness_limit()
    }
    fn get_tx_max_fee() -> Option<u64> {
        tx_max_fee()
    }
    fn get_tx_expiration() -> Option<u32> {
        tx_expiration()
    }
    fn get_tx_script_length() -> Option<u64> {
        tx_script_length()
    }
    fn get_tx_script_data_length() -> Option<u64> {
        tx_script_data_length()
    }
    fn get_tx_inputs_count() -> u64 {
        input_count().as_u64()
    }
    fn get_tx_outputs_count() -> u16 {
        output_count()
    }
    fn get_tx_witnesses_count() -> u64 {
        tx_witnesses_count()
    }
    fn get_tx_witness_data_length(index: u64) -> Option<u64> {
        tx_witness_data_length(index)
    }
    fn get_tx_witness_data(index: u64) -> Option<B512> {
        tx_witness_data(index)
    }
    fn get_tx_id() -> b256 {
        tx_id()
    }
    fn get_tx_script_bytecode_hash() -> Option<b256> {
        tx_script_bytecode_hash()
    }
    fn get_input_type(index: u64) -> Option<Input> {
        input_type(index)
    }
    fn get_input_coin_owner(index: u64) -> Option<Address> {
        input_coin_owner(index)
    }
    fn get_input_amount(index: u64) -> Option<u64> {
        input_amount(index)
    }
    fn get_input_message_sender(index: u64) -> Option<Address> {
        input_message_sender(index)
    }
    fn get_input_message_recipient(index: u64) -> Option<Address> {
        input_message_recipient(index)
    }
    fn get_input_message_nonce(index: u64) -> Option<b256> {
        input_message_nonce(index)
    }
    fn get_input_witness_index(index: u64) -> Option<u16> {
        input_witness_index(index)
    }
    fn get_input_message_data_length(index: u64) -> Option<u64> {
        input_message_data_length(index)
    }
    fn get_input_predicate_length(index: u64) -> Option<u64> {
        input_predicate_length(index)
    }
    fn get_input_predicate_data_length(index: u64) -> Option<u64> {
        input_predicate_data_length(index)
    }
    fn get_input_message_data(index: u64, offset: u64, expected_data_bytes: Bytes) -> bool {
        let data = match input_message_data(index, offset) {
            Some(b) => b,
            None => return false,
        };

        if expected_data_bytes.len() != data.len() {
            return false
        }

        let mut iter = 0;
        while iter < expected_data_bytes.len() {
            if data.get(iter).unwrap() != expected_data_bytes.get(iter).unwrap()
            {
                return false
            }
            iter += 1;
        }

        return true
    }

    fn get_input_predicate(index: u64, bytecode: Vec<u8>) -> bool {
        let code = match input_predicate(index) {
            Some(code) => code,
            None => return false,
        };

        assert(input_predicate_length(index).unwrap() == bytecode.len());
        let mut i = 0;
        while i < bytecode.len() {
            assert(bytecode.get(i).unwrap() == code.get(i).unwrap());
            i += 1;
        }
        true
    }
    fn get_output_type(ptr: u64) -> Option<Output> {
        output_type(ptr)
    }
    fn get_tx_output_amount(index: u64) -> Option<u64> {
        output_amount(index)
    }
}
