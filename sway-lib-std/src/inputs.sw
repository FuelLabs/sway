//! Getters for fields on transaction inputs.
//! This includes InputCoins, InputMessages and InputContracts.
library inputs;

use ::mem::read;

const GTF_INPUT_TYPE = 0x101;

// Input coins
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

// Input contracts
const GTF_INPUT_CONTRACT_TX_ID = 0x10E;
// const GTF_INPUT_CONTRACT_OUTPUT_INDEX = 0x10F;
// const GTF_INPUT_CONTRACT_BALANCE_ROOT = 0x110;
// const GTF_INPUT_CONTRACT_STATE_ROOT = 0x111;
// const GTF_INPUT_CONTRACT_TX_POINTER = 0x112;
// const GTF_INPUT_CONTRACT_CONTRACT_ID = 0x113;

// Input messages
// const GTF_INPUT_MESSAGE_MESSAGE_ID = 0x114;
// const GTF_INPUT_MESSAGE_SENDER = 0x115;
// const GTF_INPUT_MESSAGE_RECIPIENT = 0x116;
// const GTF_INPUT_MESSAGE_AMOUNT = 0x117;
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

/// Get the type of an input given a pointer to the input.
pub fn tx_input_type(index: u64) -> u8 {
    __gtf::<u8>(index, GTF_INPUT_TYPE)
}

/// Get the tx id of the input coin at `index`.
pub fn input_coin_tx_id(index: u64) -> b256 {
    __gtf::<b256>(index, GTF_INPUT_COIN_TX_ID)
}

/// Get output index of coin at `index`.
pub fn input_coin_output_index(index: u64) -> u64 {
    __gtf::<u64>(index, GTF_INPUT_COIN_OUTPUT_INDEX)
}

/// Get amount field from coin at `index`.
pub fn input_coin_amount(index: u64) -> u64 {
    __gtf::<u64>(index, GTF_INPUT_COIN_AMOUNT)
}


/// Get the owner of the input coin at `index`.
pub fn input_coin_owner(index: u64) -> Address {
    ~Address::from(__gtf::<b256>(index, GTF_INPUT_COIN_OWNER))
}

/// Get predicate data from InputCoin at `index`.
pub fn input_coin_predicate_data(index: u64) -> T {
    read::<T>(__gtf::<u64>(index, GTF_INPUT_COIN_PREDICATE_DATA))
}

/// Get predicate data for input at `index`
/// If the input's type is `InputCoin` or `InputMessage`,
/// return the data as an Option::Some(T).
/// Otherwise, returns Option::None.
pub fn predicate_data<T>(index: u64) -> Option<T> {
    let type = tx_input_type(index);
    match type {
        // 0 is the `Coin` Input type
        0u8 => {
            Option::Some(input_coin_predicate_data(index))
        },
        // 2 is the `Message` Input type
        2u8 => {
            Option::Some(input_message_predicate_data(index))
        },
        _ => {
            return Option::None;
        },
    };
}

/// Get the transaction inputs count.
pub fn tx_inputs_count() -> u64 {
    __gtf::<u64>(0, GTF_SCRIPT_INPUTS_COUNT)
}

/// Get a pointer to the input at `index`.
pub fn tx_input_pointer(index: u64) -> u64 {
    __gtf::<u64>(index, GTF_SCRIPT_INPUT_AT_INDEX)
}

/// If the input's type is `InputCoin` or `InputMessage`,
/// return the owner as an Option::Some(owner).
/// Otherwise, returns Option::None.
pub fn tx_input_owner(index: u64) -> Option<Address> {
    let type = tx_input_type(index);
    match type {
        // 0 is the `Coin` Input type
        0u8 => {
            return Option::Some(input_coin_owner(index))
        },
        // 2 is the `Message` Input type
        2u8 => {
            return Option::Some(input_message_owner(index))
        },
        _ => {
            return Option::None;
        },
    }
}

/// Get the owner address of the input message at `index`.
pub fn input_message_owner(index: u64) -> Address {
    ~Address::from(__gtf::<b256>(index, GTF_INPUT_MESSAGE_OWNER))
}

/// Get predicate data from message at `index`.
pub fn input_message_predicate_data(index: u64) -> T {
    read<T>(__gtf::<u64>(index, GTF_INPUT_MESSAGE_PREDICATE_DATA))
}