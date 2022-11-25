//! Getters for fields on transaction inputs.
//! This includes InputCoins, InputMessages and InputContracts.
library inputs;

use ::address::Address;
use ::option::Option;
use ::revert::revert;
use ::tx::{
    GTF_CREATE_INPUT_AT_INDEX,
    GTF_CREATE_INPUTS_COUNT,
    GTF_SCRIPT_INPUT_AT_INDEX,
    GTF_SCRIPT_INPUTS_COUNT,
    Transaction,
    tx_type,
};

const GTF_INPUT_TYPE = 0x101;

////////////////////////////////////////
// GTF Opcode const selectors
////////////////////////////////////////
// const GTF_INPUT_COIN_TX_ID = 0x102;
// const GTF_INPUT_COIN_OUTPUT_INDEX = 0x103;
const GTF_INPUT_COIN_OWNER = 0x104;
// const GTF_INPUT_COIN_AMOUNT = 0x105;
// const GTF_INPUT_COIN_ASSET_ID = 0x106;
// const GTF_INPUT_COIN_TX_POINTER = 0x107;
// const GTF_INPUT_COIN_WITNESS_INDEX = 0x108;
// const GTF_INPUT_COIN_MATURITY = 0x109;
// const GTF_INPUT_COIN_PREDICATE_LENGTH = 0x10A;
// const GTF_INPUT_COIN_PREDICATE_DATA_LENGTH = 0x10B;
// const GTF_INPUT_COIN_PREDICATE = 0x10C;
const GTF_INPUT_COIN_PREDICATE_DATA = 0x10D;

// const GTF_INPUT_CONTRACT_TX_ID = 0x10E;
// const GTF_INPUT_CONTRACT_OUTPUT_INDEX = 0x10F;
// const GTF_INPUT_CONTRACT_BALANCE_ROOT = 0x110;
// const GTF_INPUT_CONTRACT_STATE_ROOT = 0x111;
// const GTF_INPUT_CONTRACT_TX_POINTER = 0x112;
// const GTF_INPUT_CONTRACT_CONTRACT_ID = 0x113;
// const GTF_INPUT_MESSAGE_MESSAGE_ID = 0x114;
// const GTF_INPUT_MESSAGE_SENDER = 0x115;
const GTF_INPUT_MESSAGE_RECIPIENT = 0x116;

// const GTF_INPUT_MESSAGE_AMOUNT = 0x117;
// const GTF_INPUT_MESSAGE_NONCE = 0x118;
// These are based on the old spec (before
// https://github.com/FuelLabs/fuel-specs/pull/400) because that's what's
// currently implemented in `fuel-core`, `fuel-asm`, and `fuel-tx. They should
// eventually be updated.
// const GTF_INPUT_MESSAGE_WITNESS_INDEX = 0x11A;
// const GTF_INPUT_MESSAGE_DATA_LENGTH = 0x11B;
// const GTF_INPUT_MESSAGE_PREDICATE_LENGTH = 0x11C;
// const GTF_INPUT_MESSAGE_PREDICATE_DATA_LENGTH = 0x11D;
// const GTF_INPUT_MESSAGE_DATA = 0x11E;
// const GTF_INPUT_MESSAGE_PREDICATE = 0x11F;
const GTF_INPUT_MESSAGE_PREDICATE_DATA = 0x120;

pub enum Input {
    Coin: (),
    Contract: (),
    Message: (),
}

/// Get the type of the input at `index`.
pub fn input_type(index: u64) -> Input {
    let type = __gtf::<u8>(index, GTF_INPUT_TYPE);
    match type {
        0u8 => Input::Coin,
        1u8 => Input::Contract,
        2u8 => Input::Message,
        _ => revert(0),
    }
}

/// for either tx type (transaction-script or transaction-create).
pub fn input_pointer(index: u64) -> u64 {
    let type = tx_type();
    match type {
        Transaction::Script => __gtf::<u64>(index, GTF_SCRIPT_INPUT_AT_INDEX),
        Transaction::Create => __gtf::<u64>(index, GTF_CREATE_INPUT_AT_INDEX),
    }
}

/// If the input's type is `InputCoin` the owner as an Option::Some(owner).
/// Otherwise, returns Option::None.
pub fn input_owner(index: u64) -> Option<Address> {
    let type = input_type(index);
    match type {
        Input::Coin => Option::Some(Address::from(__gtf::<b256>(index, GTF_INPUT_COIN_OWNER))),
        _ => Option::None,
    }
}

/// Get the predicate data pointer from the input at `index`.
/// If the input's type is `InputCoin` or `InputMessage`,
/// return the data as an Option::Some(ptr).
/// Otherwise, returns Option::None.
pub fn input_predicate_data_pointer(index: u64) -> Option<raw_ptr> {
    let type = input_type(index);
    match type {
        Input::Coin => Option::Some(__gtf::<raw_ptr>(index, GTF_INPUT_COIN_PREDICATE_DATA)),
        Input::Message => Option::Some(__gtf::<raw_ptr>(index, GTF_INPUT_MESSAGE_PREDICATE_DATA)),
        _ => Option::None,
    }
}

pub fn input_predicate_data<T>(index: u64) -> T {
    let data = input_predicate_data_pointer(index);
    match data {
        Option::Some(d) => d.read::<T>(),
        Option::None => revert(0),
    }
}

/// Get the transaction inputs count for either tx type
/// (transaction-script or transaction-create).
pub fn input_count() -> u8 {
    let type = tx_type();
    match type {
        Transaction::Script => __gtf::<u8>(0, GTF_SCRIPT_INPUTS_COUNT),
        Transaction::Create => __gtf::<u8>(0, GTF_CREATE_INPUTS_COUNT),
    }
}
