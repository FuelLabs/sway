//! Getters for fields on transaction inputs.
//! This includes `Input::Coins`, `Input::Messages` and `Input::Contracts`.
library;

use ::address::Address;
use ::alloc::alloc_bytes;
use ::assert::assert;
use ::asset_id::AssetId;
use ::bytes::Bytes;
use ::contract_id::ContractId;
use ::option::Option::{self, *};
use ::tx::{
    GTF_CREATE_INPUT_AT_INDEX,
    GTF_CREATE_INPUTS_COUNT,
    GTF_SCRIPT_INPUT_AT_INDEX,
    GTF_SCRIPT_INPUTS_COUNT,
    Transaction,
    tx_type,
};
use ::ops::*;
use ::revert::revert;
use ::primitive_conversions::u16::*;
use ::codec::*;
use ::raw_slice::*;

// GTF Opcode const selectors
pub const GTF_INPUT_TYPE = 0x200;
// pub const GTF_INPUT_COIN_TX_ID = 0x201;
// pub const GTF_INPUT_COIN_OUTPUT_INDEX = 0x202;
pub const GTF_INPUT_COIN_OWNER = 0x203;
pub const GTF_INPUT_COIN_AMOUNT = 0x204;
pub const GTF_INPUT_COIN_ASSET_ID = 0x205;
pub const GTF_INPUT_COIN_WITNESS_INDEX = 0x207;
pub const GTF_INPUT_COIN_PREDICATE_LENGTH = 0x209;
pub const GTF_INPUT_COIN_PREDICATE_DATA_LENGTH = 0x20A;
pub const GTF_INPUT_COIN_PREDICATE = 0x20B;
pub const GTF_INPUT_COIN_PREDICATE_DATA = 0x20C;
pub const GTF_INPUT_DATA_COIN_DATA_LENGTH = 0x20E;
pub const GTF_INPUT_DATA_COIN_DATA = 0x20F;
// pub const GTF_INPUT_COIN_PREDICATE_GAS_USED = 0x20D;
// pub const GTF_INPUT_CONTRACT_CONTRACT_ID = 0x225;
pub const GTF_INPUT_MESSAGE_SENDER = 0x240;
pub const GTF_INPUT_MESSAGE_RECIPIENT = 0x241;
pub const GTF_INPUT_MESSAGE_AMOUNT = 0x242;
pub const GTF_INPUT_MESSAGE_NONCE = 0x243;
pub const GTF_INPUT_MESSAGE_WITNESS_INDEX = 0x244;
pub const GTF_INPUT_MESSAGE_DATA_LENGTH = 0x245;
pub const GTF_INPUT_MESSAGE_PREDICATE_LENGTH = 0x246;
pub const GTF_INPUT_MESSAGE_PREDICATE_DATA_LENGTH = 0x247;
pub const GTF_INPUT_MESSAGE_DATA = 0x248;
pub const GTF_INPUT_MESSAGE_PREDICATE = 0x249;
pub const GTF_INPUT_MESSAGE_PREDICATE_DATA = 0x24A;
// pub const GTF_INPUT_MESSAGE_PREDICATE_GAS_USED = 0x24B;

/// The input type for a transaction.
pub enum Input {
    /// A coin input.
    Coin: (),
    /// A coin input.
    DataCoin: (),
    /// A contract input.
    Contract: (),
    /// A message input.
    Message: (),
    /// A read-only input.
    ReadOnly: ReadOnlyInput,
}

pub enum ReadOnlyInput {
    Coin: (),
    DataCoin: (),
    CoinPredicate: (),
    DataCoinPredicate: (),
}

impl PartialEq for Input {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (Input::Coin, Input::Coin) => true,
            (Input::DataCoin, Input::DataCoin) => true,
            (Input::Contract, Input::Contract) => true,
            (Input::Message, Input::Message) => true,
            (Input::ReadOnly(ReadOnlyInput::Coin), Input::ReadOnly(ReadOnlyInput::Coin)) => true,
            (Input::ReadOnly(ReadOnlyInput::DataCoin), Input::ReadOnly(ReadOnlyInput::DataCoin)) => true,
            (Input::ReadOnly(ReadOnlyInput::CoinPredicate), Input::ReadOnly(ReadOnlyInput::CoinPredicate)) => true,
            (Input::ReadOnly(ReadOnlyInput::DataCoinPredicate), Input::ReadOnly(ReadOnlyInput::DataCoinPredicate)) => true,
            _ => false,
        }
    }
}
impl Eq for Input {}

// General Inputs

/// Gets the type of the input at `index`.
///
/// # Additional Information
///
/// The Input can be of 3 variants, `Input::Coin`, `Input::Contract` or `Input::Message`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
///
/// # Returns
///
/// * [Option<Input>] - The type of the input at `index`.
///
/// # Examples
///
/// ```sway
/// use std::inputs::input_type;
///
/// fn foo() {
///     let input_type = input_type(0).unwrap();
///     assert(input_type == Input::Coin);
/// }
/// ```
pub fn input_type(index: u64) -> Option<Input> {
    if index >= input_count().as_u64() {
        return None
    }

    match __gtf::<u8>(index, GTF_INPUT_TYPE) {
        0u8 => Some(Input::Coin),
        1u8 => Some(Input::Contract),
        2u8 => Some(Input::Message),
        3u8 => Some(Input::DataCoin),
        4u8 => Some(Input::ReadOnly(ReadOnlyInput::Coin)),
        5u8 => Some(Input::ReadOnly(ReadOnlyInput::DataCoin)),
        6u8 => Some(Input::ReadOnly(ReadOnlyInput::CoinPredicate)),
        7u8 => Some(Input::ReadOnly(ReadOnlyInput::DataCoinPredicate)),
        _ => None,
    }
}

/// Gets the transaction inputs count.
///
/// # Returns
///
/// * [u16] - The number of inputs in the transaction.
///
/// # Reverts
///
/// * When the input type is unrecognized. This should never happen.
///
/// # Examples
///
/// ```sway
/// use std::inputs::input_count;
///
/// fn foo() {
///     let input_count = input_count();
///     assert(input_count == 1_u16);
/// }
/// ```
pub fn input_count() -> u16 {
    match tx_type() {
        Transaction::Script => __gtf::<u16>(0, GTF_SCRIPT_INPUTS_COUNT),
        Transaction::Create => __gtf::<u16>(0, GTF_CREATE_INPUTS_COUNT),
        Transaction::Upgrade => __gtf::<u16>(0, GTF_SCRIPT_INPUTS_COUNT),
        Transaction::Upload => __gtf::<u16>(0, GTF_SCRIPT_INPUTS_COUNT),
        Transaction::Blob => __gtf::<u16>(0, GTF_SCRIPT_INPUTS_COUNT),
        _ => revert(0),
    }
}

/// Gets the pointer of the input at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
///
/// # Returns
///
/// * [Option<raw_ptr>] - The pointer of the input at `index`.
///
/// # Examples
///
/// ```sway
/// use std::inputs::input_pointer;
///
/// fn foo() {
///     let input_pointer = input_pointer(0).unwrap();
/// }
/// ```
#[allow(dead_code)]
fn input_pointer(index: u64) -> Option<raw_ptr> {
    if index >= input_count().as_u64() {
        return None
    }

    match tx_type() {
        Transaction::Script => Some(__gtf::<raw_ptr>(index, GTF_SCRIPT_INPUT_AT_INDEX)),
        Transaction::Create => Some(__gtf::<raw_ptr>(index, GTF_CREATE_INPUT_AT_INDEX)),
        Transaction::Upgrade => Some(__gtf::<raw_ptr>(index, GTF_SCRIPT_INPUT_AT_INDEX)),
        Transaction::Upload => Some(__gtf::<raw_ptr>(index, GTF_SCRIPT_INPUT_AT_INDEX)),
        Transaction::Blob => Some(__gtf::<raw_ptr>(index, GTF_SCRIPT_INPUT_AT_INDEX)),
        _ => None,
    }
}

/// Gets amount field from input at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
///
/// # Returns
///
/// * [Option<u64>] - The amount of the input at `index`, if the input's type is `Input::Coin` or `Input::Message`, else `None`.
///
/// # Examples
///
/// ```sway
/// use std::inputs::input_amount;
///
/// fn foo() {
///     let input_amount = input_amount(0);
///     assert(input_amount.unwrap() == 100);
/// }
/// ```
pub fn input_amount(index: u64) -> Option<u64> {
    match input_type(index) {
        Some(Input::Coin) | Some(Input::DataCoin) | Some(Input::ReadOnly(_)) => Some(__gtf::<u64>(index, GTF_INPUT_COIN_AMOUNT)),
        Some(Input::Message) => Some(__gtf::<u64>(index, GTF_INPUT_MESSAGE_AMOUNT)),
        _ => None,
    }
}

/// Gets owner field from input at `index` if it's a coin.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
///
/// # Returns
///
/// * [Option<Address>] - The owner of the input at `index`, if the input's type is `Input::Coin`, else `None`.
///
/// # Examples
///
/// ```sway
/// use std::inputs::input_coin_owner;
///
/// fn foo() {
///     let input_coin_owner = input_coin_owner(0);
///     assert(input_coin_owner.is_some()); // Ensure the input is a coin input.
/// }
/// ```
pub fn input_coin_owner(index: u64) -> Option<Address> {
    match input_type(index) {
        Some(Input::Coin) | Some(Input::DataCoin) => Some(Address::from(__gtf::<b256>(index, GTF_INPUT_COIN_OWNER))),
        _ => None,
    }
}

/// Gets the predicate data pointer from the input at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
///
/// # Returns
///
/// * [Option<raw_ptr>] - The predicate data pointer of the input at `index`, if the input's type is `Input::Coin` or `Input::Message`, else `None`.
///
/// # Examples
///
/// ```sway
/// use std::inputs::input_predicate_data_pointer;
///
/// fn foo() {
///     let input_predicate_data_pointer = input_predicate_data_pointer(0);
///     assert(input_predicate_data_pointer.is_some()); // Ensure the input is a coin or message input.
/// }
#[allow(dead_code)]
fn input_predicate_data_pointer(index: u64) -> Option<raw_ptr> {
    match input_type(index) {
        Some(Input::Coin) | Some(Input::DataCoin) => Some(__gtf::<raw_ptr>(index, GTF_INPUT_COIN_PREDICATE_DATA)),
        Some(Input::Message) => Some(__gtf::<raw_ptr>(index, GTF_INPUT_MESSAGE_PREDICATE_DATA)),
        _ => None,
    }
}

/// Gets the predicate data from the input at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
///
/// # Returns
///
/// * [Option<T>] - The predicate data of the input at `index`.
///
/// # Examples
///
/// ```sway
/// use std::inputs::input_predicate_data;
///
/// fn foo() {
///     let input_predicate_data: u64 = input_predicate_data::<u64>(0).unwrap();
///     assert(input_predicate_data == 100);
/// }
/// ```
pub fn input_predicate_data<T>(index: u64) -> Option<T>
where
    T: AbiDecode,
{
    match input_type(index) {
        Some(Input::Coin) | Some(Input::DataCoin) => Some(decode_predicate_data_by_index::<T>(index)),
        Some(Input::Message) => Some(decode_predicate_data_by_index::<T>(index)),
        _ => None,
    }
}

pub fn input_data_coin_data<T>(index: u64) -> Option<T>
where
    T: AbiDecode,
{
    match input_type(index) {
        Some(Input::DataCoin) | Some(Input::ReadOnly(ReadOnlyInput::DataCoin)) | Some(Input::ReadOnly(ReadOnlyInput::DataCoinPredicate)) => Some(decode_data_coin_data_by_index::<T>(index)),
        _ => None,
    }
}

/// Gets the AssetId of the input at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
///
/// # Returns
///
/// * [Option<AssetId>] - The asset_id of the input at `index`, if the input's type is `Input::Coin` or `Input::Message`, else `None`.
///
/// # Examples
///
/// ```sway
/// use std::inputs::input_asset_id;
///
/// fn foo() {
///     let input_asset_id = input_asset_id(0);
///     assert(input_asset_id.unwrap() == AssetId::base());
/// }
/// ```
pub fn input_asset_id(index: u64) -> Option<AssetId> {
    match input_type(index) {
        Some(Input::Coin) | Some(Input::DataCoin) | Some(Input::ReadOnly(_)) => Some(AssetId::from(__gtf::<b256>(index, GTF_INPUT_COIN_ASSET_ID))),
        Some(Input::Message) => Some(AssetId::base()),
        _ => None,
    }
}

/// Gets the witness index from the input at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
///
/// # Returns
///
/// * [Option<u16>] - The witness index of the input at `index`, if the input's type is `Input::Coin` or `Input::Message`, else `None`.
///
/// # Examples
///
/// ```sway
/// use std::inputs::input_witness_index;
///
/// fn foo() {
///     let input_witness_index = input_witness_index(0);
///     assert(input_witness_index.is_some()); // Ensure the input has a witness index.
/// }
/// ```
pub fn input_witness_index(index: u64) -> Option<u16> {
    match input_type(index) {
        Some(Input::Coin) | Some(Input::DataCoin) | Some(Input::ReadOnly(ReadOnlyInput::CoinPredicate)) | Some(Input::ReadOnly(ReadOnlyInput::DataCoinPredicate)) => Some(__gtf::<u16>(index, GTF_INPUT_COIN_WITNESS_INDEX)),
        Some(Input::Message) => Some(__gtf::<u16>(index, GTF_INPUT_MESSAGE_WITNESS_INDEX)),
        _ => None,
    }
}

/// Gets the predicate length from the input at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
///
/// # Returns
///
/// * [Option<u64>] - The predicate length of the input at `index`, if the input's type is `Input::Coin` or `Input::Message`, else `None`.
///
/// # Examples
///
/// ```sway
/// use std::inputs::input_predicate_length;
///
/// fn foo() {
///     let input_predicate_length = input_predicate_length(0);
///     assert(input_predicate_length.unwrap() != 0u64);
/// }
/// ```
pub fn input_predicate_length(index: u64) -> Option<u64> {
    match input_type(index) {
        Some(Input::Coin) | Some(Input::DataCoin) | Some(Input::ReadOnly(ReadOnlyInput::CoinPredicate)) | Some(Input::ReadOnly(ReadOnlyInput::DataCoinPredicate)) => Some(__gtf::<u64>(index, GTF_INPUT_COIN_PREDICATE_LENGTH)),
        Some(Input::Message) => Some(__gtf::<u64>(index, GTF_INPUT_MESSAGE_PREDICATE_LENGTH)),
        _ => None,
    }
}

/// Gets the predicate pointer from the input at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
///
/// # Returns
///
/// * [Option<raw_ptr>] - The predicate pointer of the input at `index`, if the input's type is `Input::Coin` or `Input::Message`, else `None`.
///
/// # Examples
///
/// ```sway
/// use std::inputs::input_predicate_pointer;
///
/// fn foo() {
///     let input_predicate_pointer = input_predicate_pointer(0);
///     assert(input_predicate_pointer.is_some());
/// }
/// ```
fn input_predicate_pointer(index: u64) -> Option<raw_ptr> {
    match input_type(index) {
        Some(Input::Coin) | Some(Input::DataCoin) | Some(Input::ReadOnly(ReadOnlyInput::CoinPredicate)) | Some(Input::ReadOnly(ReadOnlyInput::DataCoinPredicate)) => Some(__gtf::<raw_ptr>(index, GTF_INPUT_COIN_PREDICATE)),
        Some(Input::Message) => Some(__gtf::<raw_ptr>(index, GTF_INPUT_MESSAGE_PREDICATE)),
        _ => None,
    }
}

/// Gets the predicate from the input at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
///
/// # Returns
///
/// * [Option<Bytes>] - The predicate bytecode of the input at `index`, if the input's type is `Input::Coin` or `Input::Message`.
///
/// # Examples
///
/// ```sway
/// use std::inputs::input_predicate;
///
/// fn foo() {
///     let input_predicate = input_predicate(0).unwrap();
///     assert(input_predicate.len() != 0);
/// }
/// ```
pub fn input_predicate(index: u64) -> Option<Bytes> {
    let wrapped = input_predicate_length(index);
    if wrapped.is_none() {
        return None
    }

    let length = wrapped.unwrap();
    match input_predicate_pointer(index) {
        Some(d) => {
            let new_ptr = alloc_bytes(length);
            d.copy_bytes_to(new_ptr, length);
            Some(Bytes::from(raw_slice::from_parts::<u8>(new_ptr, length)))
        },
        None => None,
    }
}

/// Gets the predicate data length from the input at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
///
/// # Returns
///
/// * [Option<u64>] - The predicate data length of the input at `index`, if the input's type is `Input::Coin` or `Input::Message`, else `None`.
///
/// # Examples
///
/// ```sway
/// use std::inputs::input_predicate_data_length;
///
/// fn foo() {
///     let input_predicate_data_length = input_predicate_data_length(0);
///     assert(input_predicate_data_length.unwrap() != 0_u64);
/// }
/// ```
pub fn input_predicate_data_length(index: u64) -> Option<u64> {
    match input_type(index) {
        Some(Input::Coin) | Some(Input::DataCoin) | Some(Input::ReadOnly(ReadOnlyInput::CoinPredicate)) | Some(Input::ReadOnly(ReadOnlyInput::DataCoinPredicate)) => Some(__gtf::<u64>(index, GTF_INPUT_COIN_PREDICATE_DATA_LENGTH)),
        Some(Input::Message) => Some(__gtf::<u64>(index, GTF_INPUT_MESSAGE_PREDICATE_DATA_LENGTH)),
        _ => None,
    }
}

pub fn input_data_coin_data_length(index: u64) -> Option<u64> {
    match input_type(index) {
        Some(Input::DataCoin) | Some(Input::ReadOnly(ReadOnlyInput::DataCoin)) | Some(Input::ReadOnly(ReadOnlyInput::DataCoinPredicate)) => Some(__gtf::<u64>(index, GTF_INPUT_DATA_COIN_DATA_LENGTH)),
        _ => None,
    }
}

/// Gets the sender of the input message at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
///
/// # Returns
///
/// * [Option<Address>] - The sender of the input message at `index`, if the input's type is `Input::Message`.
///
/// # Examples
///
/// ```sway
/// use std::inputs::input_message_sender;
///
/// fn foo() {
///     let input_message_sender = input_message_sender(0).unwrap();
///     assert(input_message_sender != Address::zero());
/// }
/// ```
pub fn input_message_sender(index: u64) -> Option<Address> {
    match input_type(index) {
        Some(Input::Message) => Some(Address::from(__gtf::<b256>(index, GTF_INPUT_MESSAGE_SENDER))),
        _ => None,
    }
}

/// Gets the recipient of the input message at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
///
/// # Returns
///
/// * [Option<Address>] - The recipient of the input message at `index`, if the input's type is `Input::Message`.
///
/// # Examples
///
/// ```sway
/// use std::inputs::input_message_recipient;
///
/// fn foo() {
///     let input_message_recipient = input_message_recipient(0).unwrap();
///     assert(input_message_recipient != Address::zero());
/// }
/// ```
pub fn input_message_recipient(index: u64) -> Option<Address> {
    match input_type(index) {
        Some(Input::Message) => Some(Address::from(__gtf::<b256>(index, GTF_INPUT_MESSAGE_RECIPIENT))),
        _ => None,
    }
}

/// Gets the nonce of input message at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
///
/// # Returns
///
/// * [b256] - The nonce of the input message at `index`, if the input's type is `Input::Message`.
///
/// # Examples
///
/// ```sway
/// use std::inputs::input_message_nonce;
///
/// fn foo() {
///     let input_message_nonce = input_message_nonce(0);
///     assert(input_message_nonce != b256::zero());
/// }
/// ```
pub fn input_message_nonce(index: u64) -> Option<b256> {
    match input_type(index) {
        Some(Input::Message) => Some(__gtf::<b256>(index, GTF_INPUT_MESSAGE_NONCE)),
        _ => None,
    }
}

/// Gets the length of the input message at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
///
/// # Returns
///
/// * [Option<u64>] - The length of the input message at `index`, if the input's type is `Input::Message`.
///
/// # Examples
///
/// ```sway
/// use std::inputs::input_message_length;
///
/// fn foo() {
///     let input_message_length = input_message_length(0).unwrap();
///     assert(input_message_length != 0_u64);
/// }
/// ```
pub fn input_message_data_length(index: u64) -> Option<u64> {
    match input_type(index) {
        Some(Input::Message) => Some(__gtf::<u64>(index, GTF_INPUT_MESSAGE_DATA_LENGTH)),
        _ => None,
    }
}

/// Gets the data of the input message at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
/// * `offset`: [u64] - The offset to start reading the data from.
///
/// # Returns
///
/// * [Option<Bytes>] - The data of the input message at `index`, if the input's type is `Input::Message`.
///
/// # Examples
///
/// ```sway
/// use std::inputs::input_message_data;
///
/// fn foo() {
///     let input_message_data = input_message_data(0, 0).unwrap();
///     assert(input_message_data.len() != 0);
/// }
/// ```
pub fn input_message_data(index: u64, offset: u64) -> Option<Bytes> {
    match input_type(index) {
        Some(Input::Message) => {
            let data = __gtf::<raw_ptr>(index, GTF_INPUT_MESSAGE_DATA);
            let data_with_offset = data.add_uint_offset(offset);
            let total_length = input_message_data_length(index).unwrap();
            if offset > total_length {
                return None
            }
            let offset_length = total_length - offset;

            let new_ptr = alloc_bytes(offset_length);

            data_with_offset.copy_bytes_to(new_ptr, offset_length);
            Some(Bytes::from(raw_slice::from_parts::<u8>(new_ptr, offset_length)))
        },
        _ => None,
    }
}
