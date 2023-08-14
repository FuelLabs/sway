//! Getters for fields on transaction inputs.
//! This includes `Input::Coins`, `Input::Messages` and `Input::Contracts`.
library;

use ::address::Address;
use ::alias::AssetId;
use ::assert::assert;
use ::bytes::Bytes;
use ::constants::BASE_ASSET_ID;
use ::contract_id::ContractId;
use ::option::Option::{self, *};
use ::revert::revert;
use ::tx::{
    GTF_CREATE_INPUT_AT_INDEX,
    GTF_CREATE_INPUTS_COUNT,
    GTF_SCRIPT_INPUT_AT_INDEX,
    GTF_SCRIPT_INPUTS_COUNT,
    Transaction,
    tx_type,
};
use core::ops::Eq;
use core::primitive_conversions::*;

const GTF_INPUT_TYPE = 0x101;

// GTF Opcode const selectors
//
// pub const GTF_INPUT_COIN_TX_ID = 0x102;
// pub const GTF_INPUT_COIN_OUTPUT_INDEX = 0x103;
pub const GTF_INPUT_COIN_OWNER = 0x104;
pub const GTF_INPUT_COIN_AMOUNT = 0x105;
pub const GTF_INPUT_COIN_ASSET_ID = 0x106;
// pub const GTF_INPUT_COIN_TX_POINTER = 0x107;
pub const GTF_INPUT_COIN_WITNESS_INDEX = 0x108;
pub const GTF_INPUT_COIN_MATURITY = 0x109;
pub const GTF_INPUT_COIN_PREDICATE_LENGTH = 0x10A;
pub const GTF_INPUT_COIN_PREDICATE_DATA_LENGTH = 0x10B;
pub const GTF_INPUT_COIN_PREDICATE = 0x10C;
pub const GTF_INPUT_COIN_PREDICATE_DATA = 0x10D;

// pub const GTF_INPUT_CONTRACT_TX_ID = 0x10E;
// pub const GTF_INPUT_CONTRACT_OUTPUT_INDEX = 0x10F;
// pub const GTF_INPUT_CONTRACT_BALANCE_ROOT = 0x110;
// pub const GTF_INPUT_CONTRACT_STATE_ROOT = 0x111;
// pub const GTF_INPUT_CONTRACT_TX_POINTER = 0x112;
// pub const GTF_INPUT_CONTRACT_CONTRACT_ID = 0x113;
pub const GTF_INPUT_MESSAGE_SENDER = 0x115;
pub const GTF_INPUT_MESSAGE_RECIPIENT = 0x116;
pub const GTF_INPUT_MESSAGE_AMOUNT = 0x117;
pub const GTF_INPUT_MESSAGE_NONCE = 0x118;
// These are based on the old spec (before
// https://github.com/FuelLabs/fuel-specs/pull/400) because that's what's
// currently implemented in `fuel-core`, `fuel-asm`, and `fuel-tx. They should
// eventually be updated.
pub const GTF_INPUT_MESSAGE_WITNESS_INDEX = 0x119;
pub const GTF_INPUT_MESSAGE_DATA_LENGTH = 0x11A;
pub const GTF_INPUT_MESSAGE_PREDICATE_LENGTH = 0x11B;
pub const GTF_INPUT_MESSAGE_PREDICATE_DATA_LENGTH = 0x11C;
pub const GTF_INPUT_MESSAGE_DATA = 0x11D;
pub const GTF_INPUT_MESSAGE_PREDICATE = 0x11E;
pub const GTF_INPUT_MESSAGE_PREDICATE_DATA = 0x11F;

/// The input type for a transaction.
pub enum Input {
    Coin: (),
    Contract: (),
    Message: (),
}

impl Eq for Input {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (Input::Coin, Input::Coin) => true,
            (Input::Contract, Input::Contract) => true,
            (Input::Message, Input::Message) => true,
            _ => false,
        }
    }
}

// General Inputs
//
/// Get the type of the input at `index`.
pub fn input_type(index: u64) -> Input {
    match __gtf::<u8>(index, GTF_INPUT_TYPE) {
        0u8 => Input::Coin,
        1u8 => Input::Contract,
        2u8 => Input::Message,
        _ => revert(0),
    }
}

/// Get the transaction inputs count.
pub fn input_count() -> u8 {
    match tx_type() {
        Transaction::Script => __gtf::<u8>(0, GTF_SCRIPT_INPUTS_COUNT),
        Transaction::Create => __gtf::<u8>(0, GTF_CREATE_INPUTS_COUNT),
    }
}

/// Get the pointer of the input at `index`.
pub fn input_pointer(index: u64) -> u64 {
    match tx_type() {
        Transaction::Script => __gtf::<u64>(index, GTF_SCRIPT_INPUT_AT_INDEX),
        Transaction::Create => __gtf::<u64>(index, GTF_CREATE_INPUT_AT_INDEX),
    }
}

/// Get amount field from input at `index`.
/// If the input's type is `Input::Coin` or `Input::Message`,
/// return the amount as an `Some(u64)`.
/// Otherwise, returns `None`.
pub fn input_amount(index: u64) -> Option<u64> {
    match input_type(index) {
        Input::Coin => Some(__gtf::<u64>(index, GTF_INPUT_COIN_AMOUNT)),
        Input::Message => Some(__gtf::<u64>(index, GTF_INPUT_MESSAGE_AMOUNT)),
        Input::Contract => None,
    }
}

/// If the input's type is `Input::Coin` return the owner as an `Some(owner)`.
/// If the input's type is `Input::Message` return the owner as an `Some(recipient)`.
/// Otherwise, returns None.
pub fn input_owner(index: u64) -> Option<Address> {
    match input_type(index) {
        Input::Coin => Some(Address::from(__gtf::<b256>(index, GTF_INPUT_COIN_OWNER))),
        _ => None,
    }
}

/// Get the predicate data pointer from the input at `index`.
/// If the input's type is `Input::Coin` or `Input::Message`,
/// return the data as an `Some(ptr)`.
/// Otherwise, returns `None`.
pub fn input_predicate_data_pointer(index: u64) -> Option<raw_ptr> {
    match input_type(index) {
        Input::Coin => Some(__gtf::<raw_ptr>(index, GTF_INPUT_COIN_PREDICATE_DATA)),
        Input::Message => Some(__gtf::<raw_ptr>(index, GTF_INPUT_MESSAGE_PREDICATE_DATA)),
        Input::Contract => None,
    }
}

/// Get the predicate data from the input at `index`.
/// If the input's type is `Input::Coin` or `Input::Message`,
/// return the data, otherwise reverts.
pub fn input_predicate_data<T>(index: u64) -> T {
    match input_predicate_data_pointer(index) {
        Some(d) => d.read::<T>(),
        None => revert(0),
    }
}

/// If the input's type is `Input::Coin` return the asset ID as an `Some(id)`.
/// If the input's type is `Input::Message` return the `BASE_ASSET_ID` as an `Some(id)`.
/// Otherwise, returns `None`.
pub fn input_asset_id(index: u64) -> Option<AssetId> {
    match input_type(index) {
        Input::Coin => Some(__gtf::<b256>(index, GTF_INPUT_COIN_ASSET_ID)),
        Input::Message => Some(BASE_ASSET_ID),
        Input::Contract => None,
    }
}

/// Get the witness index from the input at `index`.
/// If the input's type is `Input::Coin` or `Input::Message`,
/// return the index as an `Some(u8)`.
/// Otherwise, returns `None`.
pub fn input_witness_index(index: u64) -> Option<u8> {
    match input_type(index) {
        Input::Coin => Some(__gtf::<u8>(index, GTF_INPUT_COIN_WITNESS_INDEX)),
        Input::Message => Some(__gtf::<u8>(index, GTF_INPUT_MESSAGE_WITNESS_INDEX)),
        Input::Contract => None,
    }
}

/// Get the predicate length from the input at `index`.
/// If the input's type is `Input::Coin` or `Input::Message`,
/// return the length as an `Some(u16)`.
/// Otherwise, returns `None`.
pub fn input_predicate_length(index: u64) -> Option<u16> {
    match input_type(index) {
        Input::Coin => Some(__gtf::<u16>(index, GTF_INPUT_COIN_PREDICATE_LENGTH)),
        Input::Message => Some(__gtf::<u16>(index, GTF_INPUT_MESSAGE_PREDICATE_LENGTH)),
        Input::Contract => None,
    }
}

/// Get the predicate pointer from the input at `index`.
/// If the input's type is `Input::Coin` or `Input::Message`,
/// return the pointer as an `Some(ptr)`.
/// Otherwise, returns `None`.
pub fn input_predicate_pointer(index: u64) -> Option<raw_ptr> {
    match input_type(index) {
        Input::Coin => Some(__gtf::<raw_ptr>(index, GTF_INPUT_COIN_PREDICATE)),
        Input::Message => Some(__gtf::<raw_ptr>(index, GTF_INPUT_MESSAGE_PREDICATE)),
        Input::Contract => None,
    }
}

/// Get the predicate from the input at `index`.
/// If the input's type is `Input::Coin` or `Input::Message`,
/// return the data, otherwise reverts.
pub fn input_predicate(index: u64) -> Bytes {
    let wrapped = input_predicate_length(index);
    if wrapped.is_none() {
        revert(0);
    };
    let length = wrapped.unwrap().as_u64();
    let mut data_bytes = Bytes::with_capacity(length);
    match input_predicate_pointer(index) {
        Some(d) => {
            data_bytes.len = length;
            d.copy_bytes_to(data_bytes.buf.ptr, length);
            data_bytes
        },
        None => revert(0),
    }
}

/// Get the predicate data length from the input at `index`.
/// If the input's type is `Input::Coin` or `Input::Message`,
/// return the data length as an `Some(u16)`.
/// Otherwise, returns `None`.
pub fn input_predicate_data_length(index: u64) -> Option<u16> {
    match input_type(index) {
        Input::Coin => Some(__gtf::<u16>(index, GTF_INPUT_COIN_PREDICATE_DATA_LENGTH)),
        Input::Message => Some(__gtf::<u16>(index, GTF_INPUT_MESSAGE_PREDICATE_DATA_LENGTH)),
        Input::Contract => None,
    }
}

// Coin Inputs
//
/// Get the maturity from the input at `index`.
/// If the input's type is `Input::Coin`,
/// return the index as an `Some(u32)`.
/// Otherwise, returns `None`.
pub fn input_maturity(index: u64) -> Option<u32> {
    match input_type(index) {
        Input::Coin => Some(__gtf::<u32>(index, GTF_INPUT_COIN_MATURITY)),
        _ => None,
    }
}

/// Get the sender of the input message at `index`.
pub fn input_message_sender(index: u64) -> Address {
    Address::from(__gtf::<b256>(index, GTF_INPUT_MESSAGE_SENDER))
}

/// Get the recipient of the input message at `index`.
pub fn input_message_recipient(index: u64) -> Address {
    Address::from(__gtf::<b256>(index, GTF_INPUT_MESSAGE_RECIPIENT))
}

/// Get the nonce of input message at `index`.
pub fn input_message_nonce(index: u64) -> b256 {
    __gtf::<b256>(index, GTF_INPUT_MESSAGE_NONCE)
}

/// Get the length of the input message at `index`.
pub fn input_message_data_length(index: u64) -> u16 {
    __gtf::<u16>(index, GTF_INPUT_MESSAGE_DATA_LENGTH)
}

/// Get the data of the input message at `index`.
pub fn input_message_data(index: u64, offset: u64) -> Bytes {
    assert(valid_input_type(index, Input::Message));
    let data = __gtf::<raw_ptr>(index, GTF_INPUT_MESSAGE_DATA);
    let data_with_offset = data.add_uint_offset(offset);
    let length = input_message_data_length(index).as_u64();
    let mut data_bytes = Bytes::with_capacity(length);
    data_bytes.len = length;
    data_with_offset.copy_bytes_to(data_bytes.buf.ptr, length);
    data_bytes
}

fn valid_input_type(index: u64, expected_type: Input) -> bool {
    input_type(index) == expected_type
}
