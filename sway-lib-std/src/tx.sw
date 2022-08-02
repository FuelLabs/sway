//! Transaction field getters.
//! This will be replaced by instructions: https://github.com/FuelLabs/fuel-specs/issues/287
library tx;

use ::address::Address;
use ::context::registers::instrs_start;
use ::contract_id::ContractId;
use ::intrinsics::is_reference_type;
use ::mem::read;
use ::option::Option;

////////////////////////////////////////
// Transaction fields
////////////////////////////////////////

// The transaction starts at
// TX_START = 32 + MAX_INPUTS*(32+8) + 8 = 32 + 255 * (40) + 8 = 10240
//
// Note that everything when serialized is padded to word length.
//
// type              = TX_START +  0*WORD_SIZE = 10240 +  0*8 = 10240
// gasPrice          = TX_START +  1*WORD_SIZE = 10240 +  1*8 = 10248
// gasLimit          = TX_START +  2*WORD_SIZE = 10240 +  2*8 = 10256
// bytePrice         = TX_START +  3*WORD_SIZE = 10240 +  3*8 = 10264
// maturity          = TX_START +  4*WORD_SIZE = 10240 +  4*8 = 10272
// scriptLength      = TX_START +  5*WORD_SIZE = 10240 +  5*8 = 10280
// scriptDataLength  = TX_START +  6*WORD_SIZE = 10240 +  6*8 = 10288
// inputsCount       = TX_START +  7*WORD_SIZE = 10240 +  7*8 = 10296
// outputsCount      = TX_START +  8*WORD_SIZE = 10240 +  8*8 = 10304
// witnessesCount    = TX_START +  9*WORD_SIZE = 10240 +  9*8 = 10312
// receiptsRoot      = TX_START + 10*WORD_SIZE = 10240 + 10*8 = 10320
// SCRIPT_START      = TX_START + 11*WORD_SIZE = 10240 + 14*8 = 10352
// SCRIPT_DATA_START = SCRIPT_START + SCRIPT_LENGTH

const TX_TYPE_OFFSET = 10240;
const TX_GAS_PRICE_OFFSET = 10248;
const TX_GAS_LIMIT_OFFSET = 10256;
const TX_BYTE_PRICE_OFFSET = 10264;
const TX_MATURITY_OFFSET = 10272;
const TX_SCRIPT_LENGTH_OFFSET = 10280;
const TX_SCRIPT_DATA_LENGTH_OFFSET = 10288;
const TX_INPUTS_COUNT_OFFSET = 10296;
const TX_OUTPUTS_COUNT_OFFSET = 10304;
const TX_WITNESSES_COUNT_OFFSET = 10312;
const TX_RECEIPTS_ROOT_OFFSET = 10320;
const TX_SCRIPT_START_OFFSET = 10352;
const TX_ID_OFFSET = 0;

// Input types
pub const INPUT_COIN = 0u8;
pub const INPUT_CONTRACT = 1u8;
pub const INPUT_MESSAGE = 2u8;

// Output types
pub const OUTPUT_COIN = 0u8;
pub const OUTPUT_CONTRACT = 1u8;
pub const OUTPUT_MESSAGE = 2u8;
pub const OUTPUT_CHANGE = 3u8;
pub const OUTPUT_VARIABLE = 4u8;
pub const OUTPUT_CONTRACT_CREATED = 5u8;

/// Get the transaction type.
pub fn tx_type() -> u8 {
    asm(r1, r2: TX_TYPE_OFFSET) {
        lw r1 r2 i0;
        r1: u8
    }
}

/// Get the transaction gas price.
pub fn tx_gas_price() -> u64 {
    asm(r1, r2: TX_GAS_PRICE_OFFSET) {
        lw r1 r2 i0;
        r1: u64
    }
}

/// Get the transaction gas limit.
pub fn tx_gas_limit() -> u64 {
    asm(r1, r2: TX_GAS_LIMIT_OFFSET) {
        lw r1 r2 i0;
        r1: u64
    }
}

/// Get the transaction byte price.
pub fn tx_byte_price() -> u64 {
    asm(r1, r2: TX_BYTE_PRICE_OFFSET) {
        lw r1 r2 i0;
        r1: u64
    }
}

/// Get the transaction maturity.
pub fn tx_maturity() -> u32 {
    asm(r1, r2: TX_MATURITY_OFFSET) {
        lw r1 r2 i0;
        r1: u32
    }
}

/// Get the transaction script length.
pub fn tx_script_length() -> u64 {
    asm(r1, r2: TX_SCRIPT_LENGTH_OFFSET) {
        lw r1 r2 i0;
        r1: u64
    }
}

/// Get the transaction script data length.
pub fn tx_script_data_length() -> u64 {
    asm(r1, r2: TX_SCRIPT_DATA_LENGTH_OFFSET) {
        lw r1 r2 i0;
        r1: u64
    }
}

/// Get the transaction inputs count.
pub fn tx_inputs_count() -> u64 {
    asm(r1, r2: TX_INPUTS_COUNT_OFFSET) {
        lw r1 r2 i0;
        r1: u64
    }
}

/// Get the transaction outputs count.
pub fn tx_outputs_count() -> u64 {
    asm(r1, r2: TX_OUTPUTS_COUNT_OFFSET) {
        lw r1 r2 i0;
        r1: u64
    }
}

/// Get the transaction witnesses count.
pub fn tx_witnesses_count() -> u64 {
    asm(r1, r2: TX_WITNESSES_COUNT_OFFSET) {
        lw r1 r2 i0;
        r1: u64
    }
}

/// Get the transaction receipts root.
pub fn tx_receipts_root() -> b256 {
    asm(r1: TX_RECEIPTS_ROOT_OFFSET) {
        r1: b256
    }
}

/// Get the transaction script start pointer.
pub fn tx_script_start_pointer() -> u64 {
    asm(r1, r2: TX_SCRIPT_START_OFFSET) {
        move r1 r2;
        r1: u64
    }
}

////////////////////////////////////////
// Script
////////////////////////////////////////

/// Get the transaction script data start pointer.
pub fn tx_script_data_start_pointer() -> u64 {
    tx_script_start_pointer() + tx_script_length()
}

/// Get the script data, typed. Unsafe.
pub fn tx_script_data<T>() -> T {
    // TODO some safety checks on the input data? We are going to assume it is the right type for now.
    read(tx_script_data_start_pointer())
}

/// Get the script bytecode
/// Must be cast to a u64 array, with sufficient length to contain the bytecode.
/// Bytecode will be padded to next whole word.
pub fn tx_script_bytecode<T>() -> T {
    read(tx_script_start_pointer())
}

////////////////////////////////////////
// Inputs
////////////////////////////////////////

/// Get a pointer to an input given the index of the input.
pub fn tx_input_pointer(index: u64) -> u64 {
    asm(r1, r2: index) {
        xis r1 r2;
        r1: u64
    }
}

/// Get the type of an input given a pointer to the input.
pub fn tx_input_type_from_pointer(ptr: u64) -> u8 {
    asm(r1, r2: ptr) {
        lw r1 r2 i0;
        r1: u8
    }
}

/// If the input's type is `InputCoin` or `InputMessage`,
/// return the owner as an Option::Some(owner).
/// Otherwise, returns Option::None.
pub fn tx_input_owner(index: u64) -> Option<Address> {
    let type = tx_input_type(index);
    let owner_offset = match type {
        // 0 is the `Coin` Input type
        0u8 => {
            // Need to skip over six words, so add 8*6=48
            48
        },
        // 2 is the `Message` Input type
        2u8 => {
            // Need to skip over eighteen words, so add 8*18=144
            144
        },
        _ => {
            return Option::None;
        },
    };

    let ptr = tx_input_pointer(index);
    Option::Some(~Address::from(b256_from_pointer_offset(ptr, owner_offset)))
}

/// Get the type of an input at a given index
pub fn tx_input_type(index: u64) -> u8 {
    let ptr = tx_input_pointer(index);
    tx_input_type_from_pointer(ptr)
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

////////////////////////////////////////
// Inputs > Predicate
////////////////////////////////////////

pub fn tx_predicate_data_start_pointer() -> u64 {
    // $is is word-aligned
    let is = instrs_start();
    let predicate_length_ptr = is - 16;
    let predicate_code_length = asm(r1, r2: predicate_length_ptr) {
        lw r1 r2 i0;
        r1: u64
    };

    let predicate_data_ptr = is + predicate_code_length;
    // predicate_data_ptr % 8 is guaranteed to be either
    //  0: if there are an even number of instructions (predicate_data_ptr is word-aligned already)
    //  4: if there are an odd number of instructions
    predicate_data_ptr + predicate_data_ptr % 8
}

pub fn get_predicate_data<T>() -> T {
    read(tx_predicate_data_start_pointer())
}

////////////////////////////////////////
// Outputs
////////////////////////////////////////

/// Get a pointer to an output given the index of the output.
pub fn tx_output_pointer(index: u64) -> u64 {
    asm(r1, r2: index) {
        xos r1 r2;
        r1: u64
    }
}

/// Get the type of an output given a pointer to the output.
pub fn tx_output_type_from_pointer(ptr: u64) -> u8 {
    asm(r1, r2: ptr) {
        lw r1 r2 i0;
        r1: u8
    }
}

/// Get the type of an output at a given index
pub fn tx_output_type(index: u64) -> u8 {
    let ptr = tx_output_pointer(index);
    tx_output_type_from_pointer(ptr)
}

/// Get the amount of coins to send for an output given a pointer to the output.
/// This method is only meaningful if the output type has the `amount` field.
/// Specifically: OutputCoin, OutputWithdrawal, OutputChange, OutputVariable.
pub fn tx_output_amount(index: u64) -> u64 {
    let ptr = tx_output_pointer(index);
    asm(r1, r2, r3: ptr) {
        addi r2 r3 i40;
        lw r1 r2 i0;
        r1: u64
    }
}

/// Get the id of the current transaction.
pub fn tx_id() -> b256 {
    read(TX_ID_OFFSET)
}
