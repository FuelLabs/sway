//! Transaction field getters.
//! This will be replaced by instructions: https://github.com/FuelLabs/fuel-specs/issues/287
library tx;

use ::address::Address;
use ::contract_id::ContractId;
use ::intrinsics::is_reference_type;

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

/// Get the transaction script start offset.
pub fn tx_script_start_offset() -> u32 {
    asm(r1, r2: TX_SCRIPT_START_OFFSET) {
        move r1 r2;
        r1: u32
    }
}

////////////////////////////////////////
// Script
////////////////////////////////////////

/// Get the transaction script data start offset.
pub fn tx_script_data_start_offset() -> u32 {
    asm(r1, r2: TX_SCRIPT_START_OFFSET, r3: TX_SCRIPT_LENGTH_OFFSET) {
        lw r3 r3 i0;
        add r1 r2 r3;
        r1: u32
    }
}

/// Get the script data, typed. Unsafe.
pub fn tx_script_data<T>() -> T {
    // TODO some safety checks on the input data? We are going to assume it is the right type for now.
    let ptr = tx_script_data_start_offset();
    if is_reference_type::<T>() {
        asm(r1: ptr) {
            r1: T
        }
    } else {
        asm(r1: ptr) {
            lw r1 r1 i0;
            r1: T
        }
    }
}

/// Get the script bytecode
/// Must be cast to a u64 array, with sufficient length to contain the bytecode.
/// Bytecode will be padded to next whole word.
pub fn tx_script_bytecode<T>() -> T {
    let script_ptr = tx_script_start_offset();
    let script_bytecode = asm(r1: script_ptr) {
        r1: T
    };
    script_bytecode
}

////////////////////////////////////////
// Inputs
////////////////////////////////////////

/// Get a pointer to an input given the index of the input.
pub fn tx_input_pointer(index: u64) -> u32 {
    asm(r1, r2: index) {
        xis r1 r2;
        r1: u32
    }
}

/// Get the type of an input given a pointer to the input.
pub fn tx_input_type_from_pointer(ptr: u32) -> u8 {
    asm(r1, r2: ptr) {
        lw r1 r2 i0;
        r1: u8
    }
}

/// Get the type of an input at a given index
pub fn tx_input_type(index: u64) -> u8 {
    let ptr = tx_input_pointer(index);
    let input_type = tx_input_type_from_pointer(ptr);
    input_type
}

/// Read 256 bits from memory at a given offset from a given pointer
pub fn b256_from_pointer_offset(pointer: u32, offset: u32) -> b256 {
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
/// If the input's type is `InputCoin`, return the owner.
/// Otherwise, undefined behavior.
pub fn tx_input_coin_owner(index: u64) -> Address {
    let input_ptr = tx_input_pointer(index);
    // Need to skip over six words, so offset is 8*6=48
    let owner_addr = ~Address::from(b256_from_pointer_offset(input_ptr, 48));
    owner_addr
}

////////////////////////////////////////
// Outputs
////////////////////////////////////////

/// Get a pointer to an output given the index of the output.
pub fn tx_output_pointer(index: u64) -> u32 {
    asm(r1, r2: index) {
        xos r1 r2;
        r1: u32
    }
}

/// Get the type of an output given a pointer to the output.
pub fn tx_output_type_from_pointer(ptr: u32) -> u8 {
    asm(r1, r2: ptr) {
        lw r1 r2 i0;
        r1: u8
    }
}

/// Get the type of an output at a given index
pub fn tx_output_type(index: u64) -> u8 {
    let ptr = tx_output_pointer(index);
    let output_type = tx_output_type_from_pointer(ptr);
    output_type
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
