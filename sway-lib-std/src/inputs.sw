//! Getters for fields on transaction inputs.
//! This includes InputCoins, InputMessages and InputContracts.
library inputs;

use ::mem::read;

const GTF_INPUT_TYPE = 0x101;

// Input coins
const GTF_INPUT_COIN_TX_ID = 0x102;
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

// Input contracts
const GTF_INPUT_CONTRACT_TX_ID = 0x10E;
// const GTF_INPUT_CONTRACT_OUTPUT_INDEX = 0x10F;
// const GTF_INPUT_CONTRACT_BALANCE_ROOT = 0x110;
// const GTF_INPUT_CONTRACT_STATE_ROOT = 0x111;
// const GTF_INPUT_CONTRACT_TX_POINTER = 0x112;
// const GTF_INPUT_CONTRACT_CONTRACT_ID = 0x113;
// const GTF_INPUT_MESSAGE_MESSAGE_ID = 0x114;
// const GTF_INPUT_MESSAGE_SENDER = 0x115;
// const GTF_INPUT_MESSAGE_RECIPIENT = 0x116;
// const GTF_INPUT_MESSAGE_AMOUNT = 0x117;
// const GTF_INPUT_MESSAGE_NONCE = 0x118;

// Input messages
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
    // GTF_INPUT_TYPE = 0x101
    asm(res, i: index) {
        gtf res i i257;
        res: u64
    }
}

// Get the tx id of the input coin at `index`.
pub fn input_coin_tx_id(index: u64) -> b256 {
    // GTF_INPUT_COIN_TX_ID = 0x102
    read<b256>(asm(res, i: index) {
        gtf res i i258;
        res: u64
    })
}

// Get the owner of the input coin at `index`.
pub fn input_coin_owner(index: u64) -> Address {
    // GTF_INPUT_COIN_OWNER = 0x104
    ~Address::from(read<b256>(asm(res, i: index) {
        gtf res i i260;
        res: u64
    }))
}

/**
// GTF_INPUT_CONTRACT_TX_ID = 0x10E
            read(asm(res, i: index) {
                gtf res i i270;
                res: u64
            }))
*/

/// Get the transaction inputs count.
pub fn tx_inputs_count() -> u64 {
    // GTF_SCRIPT_INPUTS_COUNT = 0x007
    asm(res) {
        gtf res zero i7;
        res: u64
    }
}

/// Get a pointer to an input given the index of the input.
pub fn tx_input_pointer(index: u64) -> u64 {
    // GTF_SCRIPT_INPUT_AT_INDEX = 0x00D
    asm(res, i: index) {
        gtf res i i13;
        res: u64
    }
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

/// Read 256 bits from memory at a given offset from a given pointer
pub fn b256_from_pointer_offset(pointer: u64, offset: u64) -> b256 {
    asm(buffer, ptr: pointer, off: offset) {
        // Need to skip over `off` bytes
        add ptr ptr off;
        // Save old stack pointer
        move buffer sp;
        // Extend stack by 32 bytes
        cfei i32;
        // Copy 32 bytes
        mcpi buffer ptr i32;
        // `buffer` now points to the 32 bytes
        buffer: b256
    }
}

// Get the owner of the input message at `index`.
pub fn input_message_owner(index: u64) -> Address {
    // GTF_INPUT_MESSAGE_OWNER = 0x119
    ~Address::from(read<b256>(asm(res, i: index) {
        gtf res i i281;
        res: u64
    }))
}