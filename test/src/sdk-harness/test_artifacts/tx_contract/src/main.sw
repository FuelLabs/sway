contract;

use std::{
    address::Address,
    inputs::{
        Input,
        input_amount,
        input_asset,
        input_count,
        input_maturity,
        input_owner,
        input_predicate,
        input_predicate_length,
        input_predicate_data,
        input_predicate_data_length,
        input_type,
        input_witness_index,
    },
    option::Option,
    outputs::{
        Output,
        output_count,
        output_type,
    },
    tx::{
        Transaction,
        tx_gas_price,
        tx_gas_limit,
        tx_id,
        tx_maturity,
        tx_receipts_root,
        tx_script_length,
        tx_script_data_length,
        tx_script_start_pointer,
        tx_script_bytecode_hash,
        tx_type,
        tx_witnesses_count,
    },
};

abi TxContractTest {
    fn get_tx_id() -> b256;
    fn get_tx_type() -> Transaction;
    fn get_tx_gas_price() -> u64;
    fn get_tx_gas_limit() -> u64;
    fn get_tx_maturity() -> u32;
    fn get_tx_script_length() -> u64;
    fn get_tx_script_data_length() -> u64;
    fn get_tx_witnesses_count() -> u64;
    fn get_tx_receipts_root() -> b256;
    fn get_tx_script_start_pointer() -> u64;
    fn get_tx_script_bytecode_hash() -> b256;

    fn get_input_count() -> u64;
    fn get_input_type(index: u64) -> Input;
    fn get_input_owner(index: u64) -> Address;
    fn get_input_amount(index: u64) -> u64;
    fn get_input_asset(index: u64) -> ContractId;
    fn get_input_maturity(index: u64) -> u32;
    fn get_input_witness_index(index: u64) -> u8;
    fn get_input_predicate_length(index: u64) -> u16;
    fn get_input_predicate(index: u64) -> u32;
    fn get_input_predicate_data_length(index: u64) -> u16;
    fn get_input_predicate_data(index: u64) -> u32;

    fn get_output_count() -> u64;
    fn get_output_type(index: u64) -> Output;
}

impl TxContractTest for Contract {
    fn get_tx_id() -> b256 {
        tx_id()
    }
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
    fn get_tx_witnesses_count() -> u64 {
        tx_witnesses_count()
    }
    fn get_tx_receipts_root() -> b256 {
        tx_receipts_root()
    }
    fn get_tx_script_start_pointer() -> u64 {
        tx_script_start_pointer()
    }
    fn get_tx_script_bytecode_hash() -> b256 {
        tx_script_bytecode_hash()
    }

    fn get_input_count() -> u64 {
        input_count()
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
    fn get_input_asset(index: u64) -> ContractId {
        input_asset(index).unwrap()
    }
    fn get_input_maturity(index: u64) -> u32 {
        input_maturity(index).unwrap()
    }
    fn get_input_witness_index(index: u64) -> u8 {
        input_witness_index(index).unwrap()
    }
    fn get_input_predicate_length(index: u64) -> u16 {
        input_predicate_length(index).unwrap()
    }
    fn get_input_predicate(index: u64) -> u32 {
        input_predicate(index)
    }
    fn get_input_predicate_data_length(index: u64) -> u16 {
        input_predicate_data_length(index).unwrap()
    }
    fn get_input_predicate_data(index: u64) -> u32 {
        input_predicate_data(index)
    }

    fn get_output_count() -> u64 {
        output_count()
    }
    fn get_output_type(ptr: u64) -> Output {
        output_type(ptr)
    }
}
