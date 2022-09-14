//! Transaction field getters.
library tx;

use ::address::Address;
use ::mem::read;
use ::option::Option;
use ::revert::revert;
use ::constants::ZERO_B256;

////////////////////////////////////////
// GTF Opcode const selectors
////////////////////////////////////////

const GTF_TYPE = 0x001;
const GTF_SCRIPT_GAS_PRICE = 0x002;
const GTF_SCRIPT_GAS_LIMIT = 0x003;
const GTF_SCRIPT_MATURITY = 0x004;
const GTF_SCRIPT_SCRIPT_LENGTH = 0x005;
const GTF_SCRIPT_SCRIPT_DATA_LENGTH = 0x006;
pub const GTF_SCRIPT_INPUTS_COUNT = 0x007;
pub const GTF_SCRIPT_OUTPUTS_COUNT = 0x008;
const GTF_SCRIPT_WITNESSES_COUNT = 0x009;
const GTF_SCRIPT_RECEIPTS_ROOT = 0x00A;
const GTF_SCRIPT_SCRIPT = 0x00B;
const GTF_SCRIPT_SCRIPT_DATA = 0x00C;
pub const GTF_SCRIPT_INPUT_AT_INDEX = 0x00D;
pub const GTF_SCRIPT_OUTPUT_AT_INDEX = 0x00E;
const GTF_SCRIPT_WITNESS_AT_INDEX = 0x00F;

const GTF_CREATE_GAS_PRICE = 0x010;
const GTF_CREATE_GAS_LIMIT = 0x011;
const GTF_CREATE_MATURITY = 0x012;
// const GTF_CREATE_BYTECODE_LENGTH = 0x013;
// const GTF_CREATE_BYTECODE_WITNESS_INDEX = 0x014;
// const GTF_CREATE_STORAGE_SLOTS_COUNT = 0x015;
pub const GTF_CREATE_INPUTS_COUNT = 0x016;
pub const GTF_CREATE_OUTPUTS_COUNT = 0x017;
const GTF_CREATE_WITNESSES_COUNT = 0x018;
// const GTF_CREATE_SALT = 0x019;
// const GTF_CREATE_STORAGE_SLOT_AT_INDEX = 0x01A;
pub const GTF_CREATE_INPUT_AT_INDEX = 0x01B;
pub const GTF_CREATE_OUTPUT_AT_INDEX = 0x01C;
const GTF_CREATE_WITNESS_AT_INDEX = 0x01D;

const GTF_WITNESS_DATA_LENGTH = 0x301;
const GTF_WITNESS_DATA = 0x302;

pub enum Transaction {
    Script: (),
    Create: (),
}

/// Get the type of the current transaction.
/// Either 0 (transaction-script) or 1 (transaction-create)
pub fn tx_type() -> Transaction {
    let type = __gtf::<u8>(0, GTF_TYPE);
    match type {
        0u8 => {
            Transaction::Script
        },
        1u8 => {
            Transaction::Create
        },
        _ => {
            revert(0);
        },
    }
}

/// Get the transaction gas price for either tx type
/// (transaction-script or transaction-create).
pub fn tx_gas_price() -> u64 {
    let type = tx_type();
    match type {
        Transaction::Script => {
            __gtf::<u64>(0, GTF_SCRIPT_GAS_PRICE)
        },
        Transaction::Create => {
            __gtf::<u64>(0, GTF_CREATE_GAS_PRICE)
        },
    }
}

/// Get the transaction-script gas limit for either tx type
/// (transaction-script or transaction-create).
pub fn tx_gas_limit() -> u64 {
    let type = tx_type();
    match type {
        Transaction::Script => {
            __gtf::<u64>(0, GTF_SCRIPT_GAS_LIMIT)
        },
        Transaction::Create => {
            __gtf::<u64>(0, GTF_CREATE_GAS_LIMIT)
        },
    }
}

/// Get the transaction maturity for either tx type
/// (transaction-script or transaction-create).
pub fn tx_maturity() -> u32 {
    let type = tx_type();
    match type {
        Transaction::Script => {
            __gtf::<u32>(0, GTF_SCRIPT_MATURITY)
        },
        Transaction::Create => {
            __gtf::<u32>(0, GTF_CREATE_MATURITY)
        },
    }
}

/// Get the transaction-script script length.
/// Reverts if not a transaction-script.
pub fn tx_script_length() -> u64 {
    let type = tx_type();
    match type {
        Transaction::Script => {
            __gtf::<u64>(0, GTF_SCRIPT_SCRIPT_LENGTH)
        },
        Transaction::Create => {
            revert(0)
        },
    }
}

/// Get the transaction script data length.
/// Reverts if not a transaction-script.
pub fn tx_script_data_length() -> u64 {
    let type = tx_type();
    match type {
        Transaction::Script => {
            __gtf::<u64>(0, GTF_SCRIPT_SCRIPT_DATA_LENGTH)
        },
        Transaction::Create => {
            revert(0)
        }
    }
}

/// Get the transaction witnesses count for either tx type
/// (transaction-script or transaction-create).
pub fn tx_witnesses_count() -> u64 {
    let type = tx_type();
    match type {
        Transaction::Script => {
            __gtf::<u64>(0, GTF_SCRIPT_WITNESSES_COUNT)
        },
        Transaction::Create => {
            __gtf::<u64>(0, GTF_CREATE_WITNESSES_COUNT)
        },
    }
}

// Get a pointer to the witness at index `index` for either tx type
/// (transaction-script or transaction-create).
pub fn tx_witness_pointer(index: u64) -> u64 {
    let type = tx_type();
    match type {
        Transaction::Script => {
            __gtf::<u64>(0, GTF_SCRIPT_WITNESS_AT_INDEX)
        },
        Transaction::Create => {
            __gtf::<u64>(0, GTF_CREATE_WITNESS_AT_INDEX)
        },
    }
}

// Get the length of the witness data at `index`
pub fn tx_witness_data_length(index: u64) -> u64 {
    __gtf::<u64>(index, GTF_WITNESS_DATA_LENGTH)
}

// Get the witness data at `index`.
pub fn tx_witness_data<T>(index: u64) -> T {
    read::<T>(__gtf::<u64>(index, GTF_WITNESS_DATA))
}

/// Get the transaction receipts root.
/// Reverts if not a transaction-script.
pub fn tx_receipts_root() -> b256 {
    let type = tx_type();
    match type {
        Transaction::Script => {
            read::<b256>(__gtf::<u64>(0, GTF_SCRIPT_RECEIPTS_ROOT))
        },
        _ => {
            revert(0);
        }
    }
}

/// Get the transaction script start pointer.
/// Reverts if not a transaction-script.
pub fn tx_script_start_pointer() -> u64 {
    let type = tx_type();
    match type {
        Transaction::Script => {
            __gtf::<u64>(0, GTF_SCRIPT_SCRIPT)
        },
        _ => {
            revert(0);
        }
    }
}

/// Get the transaction script data start pointer.
/// Reverts if not a transaction-script
/// (transaction-create has no script data length),
pub fn tx_script_data_start_pointer() -> u64 {
    let type = tx_type();
    match type {
        Transaction::Script => {
            __gtf::<u64>(0, GTF_SCRIPT_SCRIPT_DATA)
        },
        _ => {
            // transaction-create has no script data length
            revert(0);
        }
    }
}

/// Get the script data, typed. Unsafe.
pub fn tx_script_data<T>() -> T {
    let ptr = tx_script_data_start_pointer();
    // TODO some safety checks on the input data? We are going to assume it is the right type for now.
    read::<T>(tx_script_data_start_pointer())
}

/// Get the script bytecode
/// Must be cast to a u64 array, with sufficient length to contain the bytecode.
/// Bytecode will be padded to next whole word.
pub fn tx_script_bytecode<T>() -> T {
    read::<T>(tx_script_start_pointer())
}

/// Get the hash of the script bytecode.
/// Reverts if not a transaction-script
pub fn tx_script_bytecode_hash() -> b256 {
    let type = tx_type();
    match type {
        Transaction::Script => {
            // Get the script memory details
            let mut result_buffer: b256 = ZERO_B256;
            let script_length = __gtf::<u64>(0, GTF_SCRIPT_SCRIPT_LENGTH);
            let script_ptr = __gtf::<u64>(0, GTF_SCRIPT_SCRIPT);
            
            // Run the hash opcode for the script in memory
            asm(hash: result_buffer, ptr: script_ptr, len: script_length) {
                s256 hash ptr len;
                hash: b256
            }
        },
        _ => revert(0),
    }
}

const TX_ID_OFFSET = 0;

/// Get the id of the current transaction.
pub fn tx_id() -> b256 {
    read(TX_ID_OFFSET)
}
