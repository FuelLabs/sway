//! Getters for fields on transaction inputs.
//! This includes InputCoins, InputMessages and InputContracts.
library inputs;

use ::address::Address;
use ::mem::read;
use ::option::Option;
use ::revert::revert;
use ::tx::{
    tx_type,
    Transaction,
    GTF_SCRIPT_INPUT_AT_INDEX,
    GTF_CREATE_INPUT_AT_INDEX,
    GTF_SCRIPT_INPUTS_COUNT,
    GTF_CREATE_INPUTS_COUNT,
};

const GTF_INPUT_TYPE = 0x101;

////////////////////////////////////////
// GTF Opcode const selectors
////////////////////////////////////////

const GTF_INPUT_COIN_TX_ID = 0x102;
const GTF_INPUT_COIN_OUTPUT_INDEX = 0x103;
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

const GTF_INPUT_CONTRACT_TX_ID = 0x10E;
const GTF_INPUT_CONTRACT_OUTPUT_INDEX = 0x10F;
// const GTF_INPUT_CONTRACT_BALANCE_ROOT = 0x110;
// const GTF_INPUT_CONTRACT_STATE_ROOT = 0x111;
// const GTF_INPUT_CONTRACT_TX_POINTER = 0x112;
// const GTF_INPUT_CONTRACT_CONTRACT_ID = 0x113;

// const GTF_INPUT_MESSAGE_MESSAGE_ID = 0x114;
// const GTF_INPUT_MESSAGE_SENDER = 0x115;
// const GTF_INPUT_MESSAGE_RECIPIENT = 0x116;
const GTF_INPUT_MESSAGE_AMOUNT = 0x117;
// const GTF_INPUT_MESSAGE_NONCE = 0x118;
const GTF_INPUT_MESSAGE_OWNER = 0x119;
// const GTF_INPUT_MESSAGE_WITNESS_INDEX = 0x11A;
// const GTF_INPUT_MESSAGE_DATA_LENGTH = 0x11B;
// const GTF_INPUT_MESSAGE_PREDICATE_LENGTH = 0x11C;
// const GTF_INPUT_MESSAGE_PREDICATE_DATA_LENGTH = 0x11D;
// const GTF_INPUT_MESSAGE_DATA = 0x11E;
// const GTF_INPUT_MESSAGE_PREDICATE = 0x11F;
const GTF_INPUT_MESSAGE_PREDICATE_DATA = 0x120;

// Input types
pub const INPUT_COIN = 0u8;
pub const INPUT_CONTRACT = 1u8;
pub const INPUT_MESSAGE = 2u8;

pub enum Input {
    Coin: (),
    Contract: (),
    Message: (),
}

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

/// Get output index of coin at `index`.
/// If the input's type is `InputCoin` or `InputContract`,
/// return the amount as an Option::Some(u64).
/// Otherwise, returns Option::None.
pub fn input_output_index(index: u64) -> Option<u64> {
    let type = input_type(index);
    match type {
        Input::Coin => {
            Option::Some(__gtf::<u64>(index, GTF_INPUT_COIN_OUTPUT_INDEX))
        },
        Input::Contract => {
            Option::Some(__gtf::<u64>(index, GTF_INPUT_CONTRACT_OUTPUT_INDEX))
        },
        _ => {
            return Option::None;
        },
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
        _ => {
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

/// If the input's type is `InputCoin` or `InputMessage`,
/// return the owner as an Option::Some(owner).
/// Otherwise, returns Option::None.
pub fn input_owner(index: u64) -> Option<Address> {
    let type = input_type(index);
    match type {
        Input::Coin => {
            Option::Some(~Address::from(read::<b256>(__gtf::<u64>(index, GTF_INPUT_COIN_OWNER))))
        },
        Input::Message => {
            Option::Some(~Address::from(read::<b256>(__gtf::<u64>(index, GTF_INPUT_MESSAGE_OWNER))))
        },
        _ => {
            return Option::None;
        },
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
        _ => {
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
pub fn inputs_count() -> u64 {
    let type = tx_type();
    match type {
        Transaction::Script => {
            __gtf::<u64>(0, GTF_SCRIPT_INPUTS_COUNT)
        },
        Transaction::Create => {
            __gtf::<u64>(0, GTF_CREATE_INPUTS_COUNT)
        },
    }
}

/// Get the id of the current transaction.
/// If the input's type is `InputCoin` or `InputContract`,
/// return the data as an Option::Some(b256).
/// Otherwise, returns Option::None.
pub fn input_tx_id(index: u64) -> Option<b256> {
    let type = input_type(index);
    match type {
        Input::Coin => {
            Option::Some(read::<b256>(__gtf::<u64>(index, GTF_INPUT_COIN_TX_ID)))
        },
        Input::Contract => {
            Option::Some(read::<b256>(__gtf::<u64>(index, GTF_INPUT_CONTRACT_TX_ID)))
        },
        _ => {
            Option::None
        },
    }
}
