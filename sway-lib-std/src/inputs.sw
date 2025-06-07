//! Getters for fields on transaction inputs.
//! This includes `Input::Coin`, `Input::Message`, and `Input::Contract`.
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
    GTF_TYPE,
    Transaction,
    tx_type,
    TX_TYPE_CREATE,
    TX_TYPE_MINT,
};
use ::ops::*;
use ::revert::revert;
use ::primitive_conversions::u16::*;
use ::codec::*;
use ::debug::*;
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
    /// A contract input.
    Contract: (),
    /// A message input.
    Message: (),
}

impl PartialEq for Input {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (Input::Coin, Input::Coin) => true,
            (Input::Contract, Input::Contract) => true,
            (Input::Message, Input::Message) => true,
            _ => false,
        }
    }
}
impl Eq for Input {}

const INPUT_TYPE_COIN: u8 = 0;
const INPUT_TYPE_CONTRACT: u8 = 1;
const INPUT_TYPE_MESSAGE: u8 = 2;

// Returns the `u8` type id of the input at `index` if such input exists,
// or a non-existing type id if the `index` is out of input bounds.
//
// This private function is used to avoid the overhead of creating and
// inspecting `Option`s for the input type.
fn input_type_id(index: u64) -> u8 {
    if index < input_count().as_u64() {
        __gtf::<u8>(index, GTF_INPUT_TYPE)
    } else {
        u8::max()
    }
}

/// Gets the type of the input at `index`.
///
/// # Additional Information
///
/// The `Input` can be of three variants, `Input::Coin`, `Input::Contract`, or `Input::Message`.
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
    match input_type_id(index) {
        INPUT_TYPE_COIN => Some(Input::Coin),
        INPUT_TYPE_CONTRACT => Some(Input::Contract),
        INPUT_TYPE_MESSAGE => Some(Input::Message),
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
    match __gtf::<u8>(0, GTF_TYPE) {
        TX_TYPE_CREATE => __gtf::<u16>(0, GTF_CREATE_INPUTS_COUNT),
        TX_TYPE_MINT => revert(0),
        _ => __gtf::<u16>(0, GTF_SCRIPT_INPUTS_COUNT),
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

    match __gtf::<u8>(0, GTF_TYPE) {
        TX_TYPE_CREATE => Some(__gtf::<raw_ptr>(index, GTF_CREATE_INPUT_AT_INDEX)),
        TX_TYPE_MINT => None,
        _ => Some(__gtf::<raw_ptr>(index, GTF_SCRIPT_INPUT_AT_INDEX)),
    }
}

/// Gets the amount field from the input at `index`.
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
    match input_type_id(index) {
        INPUT_TYPE_COIN => Some(__gtf::<u64>(index, GTF_INPUT_COIN_AMOUNT)),
        INPUT_TYPE_MESSAGE => Some(__gtf::<u64>(index, GTF_INPUT_MESSAGE_AMOUNT)),
        _ => None,
    }
}

/// Gets the owner field from the input at `index` if it's a coin.
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
    match input_type_id(index) {
        INPUT_TYPE_COIN => Some(Address::from(__gtf::<b256>(index, GTF_INPUT_COIN_OWNER))),
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
    match input_type_id(index) {
        INPUT_TYPE_COIN => Some(__gtf::<raw_ptr>(index, GTF_INPUT_COIN_PREDICATE_DATA)),
        INPUT_TYPE_MESSAGE => Some(__gtf::<raw_ptr>(index, GTF_INPUT_MESSAGE_PREDICATE_DATA)),
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
    match input_type_id(index) {
        INPUT_TYPE_COIN | INPUT_TYPE_MESSAGE => Some(decode_predicate_data_by_index::<T>(index)),
        _ => None,
    }
}

/// Gets the asset id of the input at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
///
/// # Returns
///
/// * [Option<AssetId>] - The asset id of the input at `index`, if the input's type is `Input::Coin` or `Input::Message`, else `None`.
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
    match input_type_id(index) {
        INPUT_TYPE_COIN => Some(AssetId::from(__gtf::<b256>(index, GTF_INPUT_COIN_ASSET_ID))),
        INPUT_TYPE_MESSAGE => Some(AssetId::base()),
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
    match input_type_id(index) {
        INPUT_TYPE_COIN => Some(__gtf::<u16>(index, GTF_INPUT_COIN_WITNESS_INDEX)),
        INPUT_TYPE_MESSAGE => Some(__gtf::<u16>(index, GTF_INPUT_MESSAGE_WITNESS_INDEX)),
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
    match input_type_id(index) {
        INPUT_TYPE_COIN => Some(__gtf::<u64>(index, GTF_INPUT_COIN_PREDICATE_LENGTH)),
        INPUT_TYPE_MESSAGE => Some(__gtf::<u64>(index, GTF_INPUT_MESSAGE_PREDICATE_LENGTH)),
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
pub fn input_predicate_pointer(index: u64) -> Option<raw_ptr> {
    match input_type_id(index) {
        INPUT_TYPE_COIN => Some(__gtf::<raw_ptr>(index, GTF_INPUT_COIN_PREDICATE)),
        INPUT_TYPE_MESSAGE => Some(__gtf::<raw_ptr>(index, GTF_INPUT_MESSAGE_PREDICATE)),
        _ => None,
    }
}

/// Gets the predicate bytecode from the input at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
///
/// # Returns
///
/// * [Option<Bytes>] - The predicate bytecode of the input at `index`, if the input's type is `Input::Coin` or `Input::Message`, else `None`.
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
    let (length, ptr) = match input_type_id(index) {
        INPUT_TYPE_COIN => (
            __gtf::<u64>(index, GTF_INPUT_COIN_PREDICATE_LENGTH),
            __gtf::<raw_ptr>(index, GTF_INPUT_COIN_PREDICATE),
        ),
        INPUT_TYPE_MESSAGE => (
            __gtf::<u64>(index, GTF_INPUT_MESSAGE_PREDICATE_LENGTH),
            __gtf::<raw_ptr>(index, GTF_INPUT_MESSAGE_PREDICATE),
        ),
        _ => return None,
    };

    let new_ptr = alloc_bytes(length);
    ptr.copy_bytes_to(new_ptr, length);
    Some(Bytes::from(raw_slice::from_parts::<u8>(new_ptr, length)))
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
    match input_type_id(index) {
        INPUT_TYPE_COIN => Some(__gtf::<u64>(index, GTF_INPUT_COIN_PREDICATE_DATA_LENGTH)),
        INPUT_TYPE_MESSAGE => Some(__gtf::<u64>(index, GTF_INPUT_MESSAGE_PREDICATE_DATA_LENGTH)),
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
/// * [Option<Address>] - The sender of the input message at `index`, if the input's type is `Input::Message`, else `None`.
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
    match input_type_id(index) {
        INPUT_TYPE_MESSAGE => Some(Address::from(__gtf::<b256>(index, GTF_INPUT_MESSAGE_SENDER))),
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
/// * [Option<Address>] - The recipient of the input message at `index`, if the input's type is `Input::Message`, else `None`.
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
    match input_type_id(index) {
        INPUT_TYPE_MESSAGE => Some(Address::from(__gtf::<b256>(index, GTF_INPUT_MESSAGE_RECIPIENT))),
        _ => None,
    }
}

/// Gets the nonce of the input message at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
///
/// # Returns
///
/// * [b256] - The nonce of the input message at `index`, if the input's type is `Input::Message`, else `None`.
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
    match input_type_id(index) {
        INPUT_TYPE_MESSAGE => Some(__gtf::<b256>(index, GTF_INPUT_MESSAGE_NONCE)),
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
/// * [Option<u64>] - The length of the input message at `index`, if the input's type is `Input::Message`, else `None`.
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
    match input_type_id(index) {
        INPUT_TYPE_MESSAGE => Some(__gtf::<u64>(index, GTF_INPUT_MESSAGE_DATA_LENGTH)),
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
/// * [Option<Bytes>] - The data of the input message at `index`, if the input's type is `Input::Message`, else `None`.
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
    match input_type_id(index) {
        INPUT_TYPE_MESSAGE => {
            let total_length = __gtf::<u64>(index, GTF_INPUT_MESSAGE_DATA_LENGTH);

            if offset > total_length {
                None
            } else {
                let offset_length = total_length - offset;
                let new_ptr = alloc_bytes(offset_length);

                let data = __gtf::<raw_ptr>(index, GTF_INPUT_MESSAGE_DATA);
                let data_with_offset = data.add_uint_offset(offset);

                data_with_offset.copy_bytes_to(new_ptr, offset_length);
                Some(Bytes::from(raw_slice::from_parts::<u8>(new_ptr, offset_length)))
            }
        },
        _ => None,
    }
}

/// Gets the owner field from the input at `index`, if it's a coin,
/// or the recipient of the input message, if it is a message.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the input to check.
///
/// # Returns
///
/// * [Option<Address>] - The owner of the input at `index`, if the input's type is `Input::Coin`, or the recipient of the input message, if the input's type is `Input::Message`, else `None`.
///
/// # Examples
///
/// ```sway
/// use std::inputs::input_address;
///
/// fn foo() {
///     let input_address = input_address(0).unwrap();
///     assert(input_address != Address::zero());
/// }
/// ```
pub fn input_address(index: u64) -> Option<Address> {
    match input_type_id(index) {
        INPUT_TYPE_COIN => Some(Address::from(__gtf::<b256>(index, GTF_INPUT_COIN_OWNER))),
        INPUT_TYPE_MESSAGE => Some(Address::from(__gtf::<b256>(index, GTF_INPUT_MESSAGE_RECIPIENT))),
        _ => None,
    }
}
