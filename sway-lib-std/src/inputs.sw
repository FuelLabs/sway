//! Getters for fields on transaction inputs.
//! This includes InputCoins, InputMessages and InputContracts.
library inputs;

use ::address::Address;
use ::mem::read;
use ::option::Option;
use ::revert::revert;
use ::tx::{
    GTF_CREATE_INPUTS_COUNT,
    GTF_CREATE_INPUT_AT_INDEX,
    GTF_SCRIPT_INPUTS_COUNT,
    GTF_SCRIPT_INPUT_AT_INDEX,
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
const GTF_INPUT_COIN_AMOUNT = 0x105;
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

const GTF_INPUT_MESSAGE_MESSAGE_ID = 0x114;
const GTF_INPUT_MESSAGE_SENDER = 0x115;
const GTF_INPUT_MESSAGE_RECIPIENT = 0x116;
const GTF_INPUT_MESSAGE_AMOUNT = 0x117;
const GTF_INPUT_MESSAGE_NONCE = 0x118;
// These are based on the old spec (before
// https://github.com/FuelLabs/fuel-specs/pull/400) because that's what's
// currently implemented in `fuel-core`, `fuel-asm`, and `fuel-tx. They should
// eventually be updated.
const GTF_INPUT_MESSAGE_WITNESS_INDEX = 0x11A;
const GTF_INPUT_MESSAGE_DATA_LENGTH = 0x11B;
const GTF_INPUT_MESSAGE_PREDICATE_LENGTH = 0x11C;
const GTF_INPUT_MESSAGE_PREDICATE_DATA_LENGTH = 0x11D;
const GTF_INPUT_MESSAGE_DATA = 0x11E;
const GTF_INPUT_MESSAGE_PREDICATE = 0x11F;
const GTF_INPUT_MESSAGE_PREDICATE_DATA = 0x120;

pub enum Input {
    Coin: (),
    Contract: (),
    Message: (),
}

////////////////////////////////////////
// General Inputs
////////////////////////////////////////

/// Get the type of the input at `index`.
pub fn input_type(index: u64) -> Input {
    let type = __gtf::<u8>(index, GTF_INPUT_TYPE);
    match type {
        0u8 => {
            Input::Coin
        },
        1u8 => {
            Input::Contract
        },
        2u8 => {
            Input::Message
        },
        _ => {
            revert(0);
        }
    }
}

/// Get amount field from input at `index`.
/// If the input's type is `InputCoin` or `InputMessage`,
/// return the amount as an Option::Some(u64).
/// Otherwise, returns Option::None.
pub fn input_amount(index: u64) -> Option<u64> {
    let type = input_type(index);
    match type {
        Input::Coin => {
            Option::Some(__gtf::<u64>(index, GTF_INPUT_COIN_AMOUNT))
        },
        Input::Message => {
            Option::Some(__gtf::<u64>(index, GTF_INPUT_MESSAGE_AMOUNT))
        },
        Input::Contract => {
            return Option::None;
        },
    }
}

/// Get a pointer to an input given the index of the input
/// for either tx type (transaction-script or transaction-create).
pub fn input_pointer(index: u64) -> u64 {
    let type = tx_type();
    match type {
        Transaction::Script => {
            __gtf::<u64>(index, GTF_SCRIPT_INPUT_AT_INDEX)
        },
        Transaction::Create => {
            __gtf::<u64>(index, GTF_CREATE_INPUT_AT_INDEX)
        }
    }
}

/// If the input's type is `InputCoin` return the owner as an Option::Some(owner).
/// Otherwise, returns Option::None.
pub fn input_coin_owner(index: u64) -> Option<Address> {
    let type = input_type(index);
    if let Input::Coin = type {
        Option::Some(~Address::from(__gtf::<b256>(index, GTF_INPUT_COIN_OWNER)))
    } else {
        return Option::None;
    }
}

/// Get the predicate data pointer from the input at `index`.
/// If the input's type is `InputCoin` or `InputMessage`,
/// return the data as an Option::Some(ptr).
/// Otherwise, returns Option::None.
pub fn input_predicate_data_pointer(index: u64) -> Option<u64> {
    let type = input_type(index);
    match type {
        Input::Coin => {
            Option::Some(__gtf::<u64>(index, GTF_INPUT_COIN_PREDICATE_DATA))
        },
        Input::Message => {
            Option::Some(__gtf::<u64>(index, GTF_INPUT_MESSAGE_PREDICATE_DATA))
        },
        Input::Contract => {
            Option::None
        }
    }
}

pub fn input_predicate_data<T>(index: u64) -> T {
    let data = input_predicate_data_pointer(index);
    match data {
        Option::Some(d) => {
            read::<T>(d)
        },
        Option::None => {
            revert(0)
        },
    }
}

/// Get the transaction inputs count for either tx type
/// (transaction-script or transaction-create).
pub fn input_count() -> u8 {
    let type = tx_type();
    match type {
        Transaction::Script => {
            __gtf::<u8>(0, GTF_SCRIPT_INPUTS_COUNT)
        },
        Transaction::Create => {
            __gtf::<u8>(0, GTF_CREATE_INPUTS_COUNT)
        },
    }
}

////////////////////////////////////////
// Message Inputs
////////////////////////////////////////

/// Get the message id of the input message at `index`.
pub fn input_message_msg_id(index: u64) -> b256 {
    __gtf::<b256>(index, GTF_INPUT_MESSAGE_MESSAGE_ID)
}

/// Get the sender of the input message at `index`.
pub fn input_message_sender(index: u64) -> Address {
    ~Address::from(__gtf::<b256>(index, GTF_INPUT_MESSAGE_SENDER))
}

/// Get the recipient of the input message at `index`.
pub fn input_message_recipient(index: u64) -> Address {
    ~Address::from(__gtf::<b256>(index, GTF_INPUT_MESSAGE_RECIPIENT))
}

/// Get the nonce of input message at `index`.
pub fn input_message_nonce(index: u64) -> u64 {
    __gtf::<u64>(index, GTF_INPUT_MESSAGE_NONCE)
}

/// Get the witness index of the input messasge at `index`.
pub fn input_message_witness_index(index: u64) -> u8 {
    __gtf::<u8>(index, GTF_INPUT_MESSAGE_WITNESS_INDEX)
}

/// Get the length of the input message at `index`.
pub fn input_message_data_length(index: u64) -> u16 {
    __gtf::<u16>(index, GTF_INPUT_MESSAGE_DATA_LENGTH)
}

/// Get the predicate length of the input message at `index`.
pub fn input_message_predicate_length(index: u64) -> u16 {
    __gtf::<u16>(index, GTF_INPUT_MESSAGE_PREDICATE_LENGTH)
}

/// Get the predicate data length of the input message at `index`.
pub fn input_message_predicate_data_length(index: u64) -> u16 {
    __gtf::<u16>(index, GTF_INPUT_MESSAGE_PREDICATE_DATA_LENGTH)
}

/// Get the data of the input message at `index`.
pub fn input_message_data<T>(index: u64, offset: u64) -> T {
    read::<T>(__gtf::<u64>(index, GTF_INPUT_MESSAGE_DATA) + offset)
}

/// Get the predicate of the input message at `index`.
pub fn input_message_predicate<T>(index: u64) -> T {
    read::<T>(__gtf::<u64>(index, GTF_INPUT_MESSAGE_PREDICATE))
}
