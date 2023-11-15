//! Transaction field getters.
library;

use ::constants::ZERO_B256;
use ::revert::revert;
use ::option::Option::{self, *};

// GTF Opcode const selectors
//
pub const GTF_TYPE = 0x001;
pub const GTF_SCRIPT_GAS_LIMIT = 0x002;
pub const GTF_SCRIPT_SCRIPT_LENGTH = 0x003;
pub const GTF_SCRIPT_SCRIPT_DATA_LENGTH = 0x004;
pub const GTF_SCRIPT_INPUTS_COUNT = 0x005;
pub const GTF_SCRIPT_OUTPUTS_COUNT = 0x006;
pub const GTF_SCRIPT_WITNESSES_COUNT = 0x007;
pub const GTF_SCRIPT_RECEIPTS_ROOT = 0x008;
pub const GTF_SCRIPT_SCRIPT = 0x009;
pub const GTF_SCRIPT_SCRIPT_DATA = 0x00A;
pub const GTF_SCRIPT_INPUT_AT_INDEX = 0x00B;
pub const GTF_SCRIPT_OUTPUT_AT_INDEX = 0x00C;
pub const GTF_SCRIPT_WITNESS_AT_INDEX = 0x00D;

// pub const GTF_CREATE_BYTECODE_LENGTH = 0x100;
// pub const GTF_CREATE_BYTECODE_WITNESS_INDEX = 0x101;
// pub const GTF_CREATE_STORAGE_SLOTS_COUNT = 0x102;
pub const GTF_CREATE_INPUTS_COUNT = 0x103;
pub const GTF_CREATE_OUTPUTS_COUNT = 0x104;
pub const GTF_CREATE_WITNESSES_COUNT = 0x105;
// pub const GTF_CREATE_SALT = 0x106;
// pub const GTF_CREATE_STORAGE_SLOT_AT_INDEX = 0x107;
pub const GTF_CREATE_INPUT_AT_INDEX = 0x108;
pub const GTF_CREATE_OUTPUT_AT_INDEX = 0x109;
pub const GTF_CREATE_WITNESS_AT_INDEX = 0x10A;

pub const GTF_WITNESS_DATA_LENGTH = 0x400;
pub const GTF_WITNESS_DATA = 0x401;

pub const GTF_POLICY_TYPES = 0x500;
pub const GTF_POLICY_GAS_PRICE = 0x501;
pub const GTF_POLICY_WITNESS_LIMIT = 0x502;
pub const GTF_POLICY_MATURITY = 0x503;
pub const GTF_POLICY_MAX_FEE = 0x504;

/// A transaction type.
pub enum Transaction {
    /// A standard transaction, where execution is defined by a script.
    Script: (),
    /// A contract deployment transaction.
    Create: (),
}

/// Get the type of the current transaction.
/// Either `Transaction::Script` or `Transaction::Create`.
///
/// # Returns
///
/// * [Transaction] - The type of the current transaction.
///
/// # Reverts
///
/// * When the transaction type is unrecognized. This should never happen.
///
/// # Example
///
/// ```sway
/// use std::tx::tx_type;
///
/// fn foo() {
///     let tx_type = tx_type();
///     match tx_type {
///         Transaction::Script => {
///             log("Regular script transaction");
///         },
///         Transaction::Create => {
///             log("Contract deployment transaction");
///         },
///     }
/// }
/// ```
pub fn tx_type() -> Transaction {
    match __gtf::<u8>(0, GTF_TYPE) {
        0u8 => Transaction::Script,
        1u8 => Transaction::Create,
        _ => revert(0),
    }
}

const GAS_PRICE_POLICY: u32 = 1u32 << 0;
const MATURITY_POLICY: u32 = 1u32 << 1;
const WITNESS_LIMIT_POLICY: u32 = 1u32 << 2;
const MAX_FEE_POLICY: u32 = 1u32 << 3;

/// Returns policies bits. It can be used to identify which policies are set.
fn policies() -> u32 {
    __gtf::<u32>(0, GTF_POLICY_TYPES)
}

/// Get the gas price for the transaction, if it is set.
///
/// # Returns
///
/// * [Option<u64>] - The gas price for the transaction.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_gas_price;
///
/// fn foo() {
///     let gas_price = tx_gas_price();
///     log(gas_price);
/// }
/// ```
pub fn tx_gas_price() -> Option<u64> {
    let bits = policies();
    if bits & GAS_PRICE_POLICY > 0 {
        Some(__gtf::<u64>(0, GTF_POLICY_GAS_PRICE))
    } else {
        None
    }
}

/// Get the script gas limit for the transaction.
///
/// # Returns
///
/// * [u64] - The script gas limit for the transaction.
///
/// # Examples
///
/// ```sway
/// use std::tx::script_gas_limit;
///
/// fn foo() {
///     let gas_limit = script_gas_limit();
///     log(gas_limit);
/// }
/// ```
pub fn script_gas_limit() -> u64 {
    __gtf::<u64>(0, GTF_SCRIPT_GAS_LIMIT)
}

/// Get the maturity for the transaction, if it is set.
///
/// # Returns
///
/// * [Option<u32>] - The maturity for the transaction.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_maturity;
///
/// fn foo() {
///     let maturity = tx_maturity();
///     log(maturity);
/// }
/// ```
pub fn tx_maturity() -> Option<u32> {
    let bits = policies();
    if bits & MATURITY_POLICY > 0 {
        Some(__gtf::<u32>(0, GTF_POLICY_GAS_PRICE))
    } else {
        None
    }
}

/// Get the witness limit for the transaction, if it is set.
///
/// # Returns
///
/// * [Option<u64>] - The witness limit for the transaction.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_witness_limit;
///
/// fn foo() {
///     let witness_limit = tx_witness_limit();
///     log(witness_limit);
/// }
/// ```
pub fn tx_witness_limit() -> Option<u64> {
    let bits = policies();
    if bits & WITNESS_LIMIT_POLICY > 0 {
        Some(__gtf::<u64>(0, GTF_POLICY_WITNESS_LIMIT))
    } else {
        None
    }
}

/// Get the max fee for the transaction, if it is set.
///
/// # Returns
///
/// * [Option<u64>] - The max fee for the transaction.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_max_fee;
///
/// fn foo() {
///     let max_fee = tx_max_fee();
///     log(max_fee);
/// }
/// ```
pub fn tx_max_fee() -> Option<u64> {
    let bits = policies();
    if bits & MAX_FEE_POLICY > 0 {
        Some(__gtf::<u64>(0, GTF_POLICY_MAX_FEE))
    } else {
        None
    }
}

/// Get the length of the script for the transaction.
///
/// # Returns
///
/// * [u64] - The script length for the transaction.
///
/// # Reverts
///
/// * When the transaction type is of type `Transaction::Create`.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_script_length;
///
/// fn foo() {
///     let script_length = tx_script_length();
///     assert(script_length > 0);
/// }
/// ```
pub fn tx_script_length() -> u64 {
    match tx_type() {
        Transaction::Script => __gtf::<u64>(0, GTF_SCRIPT_SCRIPT_LENGTH),
        Transaction::Create => revert(0),
    }
}

/// Get the script data length for the transaction.
///
/// # Returns
///
/// * [u64] - The script data length for the transaction.
///
/// # Reverts
///
/// * When the transaction type is of type `Transaction::Create`.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_script_data_length;
///
/// fn foo() {
///     let script_data_length = tx_script_data_length();
///     assert(script_data_length > 0);
/// }
/// ```
pub fn tx_script_data_length() -> u64 {
    match tx_type() {
        Transaction::Script => __gtf::<u64>(0, GTF_SCRIPT_SCRIPT_DATA_LENGTH),
        Transaction::Create => revert(0),
    }
}

/// Get the transaction witnesses count for the transaction.
///
/// # Returns
///
/// * [u64] - The witnesses count for the transaction.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_witnesses_count;
///
/// fn foo() {
///     let witnesses_count = tx_witnesses_count();
///     log(witnesses_count);
/// }
/// ```
pub fn tx_witnesses_count() -> u64 {
    match tx_type() {
        Transaction::Script => __gtf::<u64>(0, GTF_SCRIPT_WITNESSES_COUNT),
        Transaction::Create => __gtf::<u64>(0, GTF_CREATE_WITNESSES_COUNT),
    }
}

/// Get a pointer to the witness at index `index` for the transaction.
///
/// # Arguments
///
/// * `index` - The index of the witness to get the pointer for.
///
/// # Returns
///
/// * [u64] - The pointer to the witness at index `index`.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_witness_pointer;
///
/// fn foo() {
///     let witness_pointer = tx_witness_pointer(0);
///     log(witness_pointer);
/// }
/// ```
pub fn tx_witness_pointer(index: u64) -> u64 {
    match tx_type() {
        Transaction::Script => __gtf::<u64>(index, GTF_SCRIPT_WITNESS_AT_INDEX),
        Transaction::Create => __gtf::<u64>(index, GTF_CREATE_WITNESS_AT_INDEX),
    }
}

/// Get the length of the witness data at `index`.
///
/// # Arguments
///
/// * `index` - The index of the witness to get the data length for.
///
/// # Returns
///
/// * [u64] - The length of the witness data at `index`.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_witness_data_length;
///
/// fn foo() {
///     let witness_data_length = tx_witness_data_length(0);
///     log(witness_data_length);
/// }
/// ```
pub fn tx_witness_data_length(index: u64) -> u64 {
    __gtf::<u64>(index, GTF_WITNESS_DATA_LENGTH)
}

/// Get the witness data at `index`.
///
/// # Arguments
///
/// * `index` - The index of the witness to get the data for.
///
/// # Returns
///
/// * [T] - The witness data at `index`.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_witness_data;
///
/// fn foo() {
///     let witness_data: u64 = tx_witness_data(0);
///     log(witness_data);
/// }
/// ```
pub fn tx_witness_data<T>(index: u64) -> T {
    __gtf::<raw_ptr>(index, GTF_WITNESS_DATA).read::<T>()
}

/// Get the transaction receipts root.
///
/// # Returns
///
/// * [b256] - The transaction receipts root.
///
/// # Reverts
///
/// * When the transaction type is of type `Transaction::Create`.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_receipts_root;
///
/// fn foo() {
///     let receipts_root = tx_receipts_root();
///     log(receipts_root);
/// }
/// ```
pub fn tx_receipts_root() -> b256 {
    match tx_type() {
        Transaction::Script => __gtf::<raw_ptr>(0, GTF_SCRIPT_RECEIPTS_ROOT).read::<b256>(),
        _ => revert(0),
    }
}

/// Get the transaction script start pointer.
///
/// # Returns
///
/// * [raw_ptr] - The transaction script start pointer.
///
/// # Reverts
///
/// * When the transaction type is of type `Transaction::Create`.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_script_start_pointer;
///
/// fn foo() {
///     let script_start_pointer = tx_script_start_pointer();
///     log(script_start_pointer);
/// }
/// ```
pub fn tx_script_start_pointer() -> raw_ptr {
    match tx_type() {
        Transaction::Script => __gtf::<raw_ptr>(0, GTF_SCRIPT_SCRIPT),
        _ => revert(0),
    }
}

/// Get the transaction script data start pointer.
///
/// # Returns
///
/// * [raw_ptr] - The transaction script data start pointer.
///
/// # Reverts
///
/// * When the transaction type is of type `Transaction::Create`.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_script_data_start_pointer;
///
/// fn foo() {
///     let script_data_start_pointer = tx_script_data_start_pointer();
///     log(script_data_start_pointer);
/// }
/// ```
pub fn tx_script_data_start_pointer() -> raw_ptr {
    match tx_type() {
        Transaction::Script => __gtf::<raw_ptr>(0, GTF_SCRIPT_SCRIPT_DATA),
        _ => {
            // transaction-create has no script data length
            revert(0);
        }
    }
}

/// Get the script data, typed.
///
/// # Additional Information
///
/// **Unsafe.**
/// **Assumes the type is correct.**
///
/// # Returns
///
/// * [T] - The script data, typed.
///
/// # Reverts
///
/// * When the transaction type is of type `Transaction::Create`.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_script_data;
///
/// fn foo() {
///     let script_data: u64 = tx_script_data();
///     log(script_data);
/// }
/// ```
pub fn tx_script_data<T>() -> T {
    let ptr = tx_script_data_start_pointer();
    // TODO some safety checks on the input data? We are going to assume it is the right type for now.
    ptr.read::<T>()
}

/// Get the script bytecode.
///
/// # Additional Information
///
/// Must be cast to a `u64` array, with sufficient length to contain the bytecode.
/// Bytecode will be padded to next whole word.
///
/// # Returns
///
/// * [T] - The script bytecode.
///
/// # Reverts
///
/// * When the transaction type is of type `Transaction::Create`.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_script_bytecode;
///
/// fn foo() {
///     let script_bytecode: [u64; 64] = tx_script_bytecode();
///     log(script_bytecode);
/// }
/// ```
pub fn tx_script_bytecode<T>() -> T {
    tx_script_start_pointer().read::<T>()
}

/// Get the hash of the script bytecode.
/// Reverts if not a transaction-script.
///
/// # Returns
///
/// * [b256] - The hash of the script bytecode.
///
/// # Reverts
///
/// * When the transaction type is of type `Transaction::Create`.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_script_bytecode_hash;
///
/// fn foo() {
///     let script_bytecode_hash: b256 = tx_script_bytecode_hash();
///     assert(script_bytecode_hash == 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef);
/// }
/// ```
pub fn tx_script_bytecode_hash() -> b256 {
    match tx_type() {
        Transaction::Script => {
            // Get the script memory details
            let mut result_buffer = ZERO_B256;
            let script_length = tx_script_length();
            let script_ptr = tx_script_start_pointer();

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

/// Get the Transaction ID of the current transaction.
///
/// # Returns
///
/// * [b256] - The Transaction ID of the current transaction.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_id;
///
/// fn foo() {
///     let tx_id: b256 = tx_id();
///     log(tx_id);
/// }
/// ```
pub fn tx_id() -> b256 {
    asm(ptr: TX_ID_OFFSET) { ptr: raw_ptr }.read()
}
