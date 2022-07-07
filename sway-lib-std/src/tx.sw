//! Transaction field getters.
//! This will be replaced by instructions: https://github.com/FuelLabs/fuel-specs/issues/287
library tx;

use ::address::Address;
use ::context::registers::instrs_start;
use ::contract_id::ContractId;
use ::intrinsics::is_reference_type;
use ::mem::read;

////////////////////////////////////////
// GTF Immediates
////////////////////////////////////////

const GTF_TYPE = 0x001;

const GTF_SCRIPT_GAS_PRICE = 0x002;
const GTF_SCRIPT_GAS_LIMIT = 0x003;
const GTF_SCRIPT_MATURITY = 0x004;
const GTF_SCRIPT_SCRIPT_LENGTH = 0x005;
const GTF_SCRIPT_SCRIPT_DATA_LENGTH = 0x006;
const GTF_SCRIPT_INPUTS_COUNT = 0x007;
const GTF_SCRIPT_OUTPUTS_COUNT = 0x008;
const GTF_SCRIPT_WITNESSES_COUNT = 0x009;
const GTF_SCRIPT_RECEIPTS_ROOT = 0x00A;
const GTF_SCRIPT_SCRIPT = 0x00B;
const GTF_SCRIPT_SCRIPT_DATA = 0x00C;
const GTF_SCRIPT_INPUT_AT_INDEX = 0x00D;
const GTF_SCRIPT_OUTPUT_AT_INDEX = 0x00E;
const GTF_SCRIPT_WITNESS_AT_INDEX = 0x00F;

const GTF_CREATE_GAS_PRICE = 0x010;
const GTF_CREATE_GAS_LIMIT = 0x011;
const GTF_CREATE_MATURITY = 0x012;
const GTF_CREATE_BYTECODE_LENGTH = 0x013;
const GTF_CREATE_BYTECODE_WITNESS_INDEX = 0x014;
const GTF_CREATE_STORAGE_SLOTS_COUNT = 0x015;
const GTF_CREATE_INPUTS_COUNT = 0x016;
const GTF_CREATE_OUTPUTS_COUNT = 0x017;
const GTF_CREATE_WITNESSES_COUNT = 0x018;
const GTF_CREATE_SALT = 0x019;
const GTF_CREATE_STORAGE_SLOT_AT_INDEX = 0x01A;
const GTF_CREATE_INPUT_AT_INDEX = 0x01B;
const GTF_CREATE_OUTPUT_AT_INDEX = 0x01C;
const GTF_CREATE_WITNESS_AT_INDEX = 0x01D;

const GTF_INPUT_TYPE = 0x101;

const GTF_INPUT_COIN_TX_ID = 0x102;
const GTF_INPUT_COIN_OUTPUT_INDEX = 0x103;
const GTF_INPUT_COIN_OWNER = 0x104;
const GTF_INPUT_COIN_AMOUNT = 0x105;
const GTF_INPUT_COIN_ASSET_ID = 0x106;
const GTF_INPUT_COIN_TX_POINTER = 0x107;
const GTF_INPUT_COIN_WITNESS_INDEX = 0x108;
const GTF_INPUT_COIN_MATURITY = 0x109;
const GTF_INPUT_COIN_PREDICATE_LENGTH = 0x10A;
const GTF_INPUT_COIN_PREDICATE_DATA_LENGTH = 0x10B;
const GTF_INPUT_COIN_PREDICATE = 0x10C;
const GTF_INPUT_COIN_PREDICATE_DATA = 0x10D;

const GTF_INPUT_CONTRACT_TX_ID = 0x10E;
const GTF_INPUT_CONTRACT_OUTPUT_INDEX = 0x10F;
const GTF_INPUT_CONTRACT_BALANCE_ROOT = 0x110;
const GTF_INPUT_CONTRACT_STATE_ROOT = 0x111;
const GTF_INPUT_CONTRACT_TX_POINTER = 0x112;
const GTF_INPUT_CONTRACT_CONTRACT_ID = 0x113;

const GTF_INPUT_MESSAGE_MESSAGE_ID = 0x114;
const GTF_INPUT_MESSAGE_SENDER = 0x115;
const GTF_INPUT_MESSAGE_RECIPIENT = 0x116;
const GTF_INPUT_MESSAGE_AMOUNT = 0x117;
const GTF_INPUT_MESSAGE_NONCE = 0x118;
const GTF_INPUT_MESSAGE_OWNER = 0x119;
const GTF_INPUT_MESSAGE_WITNESS_INDEX = 0x11A;
const GTF_INPUT_MESSAGE_DATA_LENGTH = 0x11B;
const GTF_INPUT_MESSAGE_PREDICATE_LENGTH = 0x11C;
const GTF_INPUT_MESSAGE_PREDICATE_DATA_LENGTH = 0x11D;
const GTF_INPUT_MESSAGE_DATA = 0x11E;
const GTF_INPUT_MESSAGE_PREDICATE = 0x11F;
const GTF_INPUT_MESSAGE_PREDICATE_DATA = 0x120;

const GTF_OUTPUT_TYPE = 0x201;

const GTF_OUTPUT_COIN_TO = 0x202;
const GTF_OUTPUT_COIN_AMOUNT = 0x203;
const GTF_OUTPUT_COIN_ASSET_ID = 0x204;

const GTF_OUTPUT_CONTRACT_INPUT_INDEX = 0x205;
const GTF_OUTPUT_CONTRACT_BALANCE_ROOT = 0x206;
const GTF_OUTPUT_CONTRACT_STATE_ROOT = 0x207;

const GTF_OUTPUT_MESSAGE_RECIPIENT = 0x208;
const GTF_OUTPUT_MESSAGE_AMOUNT = 0x209;

const GTF_OUTPUT_CONTRACT_CREATED_CONTRACT_ID = 0x20A;
const GTF_OUTPUT_CONTRACT_CREATED_STATE_ROOT = 0x20B;

const GTF_WITNESS_DATA_LENGTH = 0x301;
const GTF_WITNESS_DATA = 0x302;

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
/// If the input's type is `InputCoin`, return the owner.
/// Otherwise, undefined behavior.
pub fn tx_input_coin_owner(index: u64) -> Address {
    let input_ptr = tx_input_pointer(index);
    // Need to skip over six words, so offset is 8*6=48
    ~Address::from(b256_from_pointer_offset(input_ptr, 48))
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
