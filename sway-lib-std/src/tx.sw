//! Transaction field getters.
library;

use ::revert::revert;
use ::option::Option::{self, *};
use ::alloc::alloc_bytes;

// GTF Opcode const selectors
//
pub const GTF_TYPE = 0x001;
pub const GTF_SCRIPT_GAS_LIMIT = 0x002;
pub const GTF_SCRIPT_SCRIPT_LENGTH = 0x003;
pub const GTF_SCRIPT_SCRIPT_DATA_LENGTH = 0x004;
pub const GTF_SCRIPT_INPUTS_COUNT = 0x005;
pub const GTF_SCRIPT_OUTPUTS_COUNT = 0x006;
pub const GTF_SCRIPT_WITNESSES_COUNT = 0x007;
pub const GTF_SCRIPT_SCRIPT = 0x009;
pub const GTF_SCRIPT_SCRIPT_DATA = 0x00A;
pub const GTF_SCRIPT_INPUT_AT_INDEX = 0x00B;
pub const GTF_SCRIPT_OUTPUT_AT_INDEX = 0x00C;
pub const GTF_SCRIPT_WITNESS_AT_INDEX = 0x00D;

pub const GTF_TX_LENGTH = 0x00E;

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
pub const GTF_POLICY_TIP = 0x501;
pub const GTF_POLICY_WITNESS_LIMIT = 0x502;
pub const GTF_POLICY_MATURITY = 0x503;
pub const GTF_POLICY_MAX_FEE = 0x504;

/// A transaction type.
pub enum Transaction {
    /// A standard transaction, where execution is defined by a script.
    Script: (),
    /// A contract deployment transaction.
    Create: (),
    /// The transaction is created by the block producer and is not signed.
    ///
    /// # Additional Information
    ///
    /// NOTE: This should never be valid in execution but it provided for congruency to the FuelVM specs.
    Mint: (),
    /// The Upgrade transaction allows upgrading either consensus parameters or state transition function used by the network to produce future blocks.
    Upgrade: (),
    ///The Upload transaction allows the huge bytecode to be divided into subsections and uploaded slowly to the chain.
    Upload: (),
    /// The Blob inserts a simple binary blob in the chain. It's raw immutable data that can be cheaply loaded by the VM and used as instructions or just data.
    Blob: (),
}

impl core::ops::Eq for Transaction {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (Transaction::Script, Transaction::Script) => true,
            (Transaction::Create, Transaction::Create) => true,
            (Transaction::Mint, Transaction::Mint) => true,
            (Transaction::Upgrade, Transaction::Upgrade) => true,
            (Transaction::Upload, Transaction::Upload) => true,
            (Transaction::Blob, Transaction::Blob) => true,
            _ => false,
        }
    }
}

/// Get the type of the current transaction.
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
///         Transaction::Mint => {
///             log("This should never happen");
///         },
///         Transaction::Upgrade => {
///             log("Upgrade transaction");
///         },
///         Transaction::Upload => {
///             log("Upload transaction");
///         },
///         Transaction::Blob => {
///             log("Blob transaction");
///         },
///     }
/// }
/// ```
pub fn tx_type() -> Transaction {
    match __gtf::<u8>(0, GTF_TYPE) {
        0u8 => Transaction::Script,
        1u8 => Transaction::Create,
        3u8 => Transaction::Upgrade,
        4u8 => Transaction::Upload,
        5u8 => Transaction::Blob,
        _ => revert(0),
    }
}

const TIP_POLICY: u32 = 1u32 << 0;
const WITNESS_LIMIT_POLICY: u32 = 1u32 << 1;
const MATURITY_POLICY: u32 = 1u32 << 2;
const MAX_FEE_POLICY: u32 = 1u32 << 3;

/// Returns policies bits. It can be used to identify which policies are set.
fn policies() -> u32 {
    __gtf::<u32>(0, GTF_POLICY_TYPES)
}

/// Get the tip for the transaction, if it is set.
///
/// # Returns
///
/// * [Option<u64>] - The tip for the transaction.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_tip;
///
/// fn foo() {
///     let tip = tx_tip();
///     log(tip);
/// }
/// ```
pub fn tx_tip() -> Option<u64> {
    let bits = policies();
    if bits & TIP_POLICY > 0 {
        Some(__gtf::<u64>(0, GTF_POLICY_TIP))
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
///     let maturity = tx_maturity().unwrap();
///     log(maturity);
/// }
/// ```
pub fn tx_maturity() -> Option<u32> {
    let bits = policies();
    if bits & MATURITY_POLICY > 0 {
        Some(__gtf::<u32>(0, GTF_POLICY_MATURITY))
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
/// * [Option<u64>] - The script length for the transaction.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_script_length;
///
/// fn foo() {
///     let script_length = tx_script_length().unwrap();
///     assert(script_length > 0);
/// }
/// ```
pub fn tx_script_length() -> Option<u64> {
    match tx_type() {
        Transaction::Script => Some(__gtf::<u64>(0, GTF_SCRIPT_SCRIPT_LENGTH)),
        _ => None,
    }
}

/// Get the script data length for the transaction.
///
/// # Returns
///
/// * [u64] - The script data length for the transaction.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_script_data_length;
///
/// fn foo() {
///     let script_data_length = tx_script_data_length().unwrap();
///     assert(script_data_length > 0);
/// }
/// ```
pub fn tx_script_data_length() -> Option<u64> {
    match tx_type() {
        Transaction::Script => Some(__gtf::<u64>(0, GTF_SCRIPT_SCRIPT_DATA_LENGTH)),
        _ => None,
    }
}

/// Get the transaction witnesses count for the transaction.
///
/// # Returns
///
/// * [u64] - The witnesses count for the transaction.
///
/// # Reverts
///
/// * When the transaction type is unrecognized. This should never happen.
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
        Transaction::Upgrade => __gtf::<u64>(0, GTF_SCRIPT_WITNESSES_COUNT),
        Transaction::Upload => __gtf::<u64>(0, GTF_SCRIPT_WITNESSES_COUNT),
        Transaction::Blob => __gtf::<u64>(0, GTF_SCRIPT_WITNESSES_COUNT),
        _ => revert(0),
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
/// * [Option<raw_ptr>] - The pointer to the witness at index `index`.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_witness_pointer;
///
/// fn foo() {
///     let witness_pointer = tx_witness_pointer(0).unwrap();
/// }
/// ```
#[allow(dead_code)]
fn tx_witness_pointer(index: u64) -> Option<raw_ptr> {
    if index >= tx_witnesses_count() {
        return None
    }

    match tx_type() {
        Transaction::Script => Some(__gtf::<raw_ptr>(index, GTF_SCRIPT_WITNESS_AT_INDEX)),
        Transaction::Create => Some(__gtf::<raw_ptr>(index, GTF_CREATE_WITNESS_AT_INDEX)),
        Transaction::Upgrade => Some(__gtf::<raw_ptr>(index, GTF_SCRIPT_WITNESS_AT_INDEX)),
        Transaction::Upload => Some(__gtf::<raw_ptr>(index, GTF_SCRIPT_WITNESS_AT_INDEX)),
        Transaction::Blob => Some(__gtf::<raw_ptr>(index, GTF_SCRIPT_WITNESS_AT_INDEX)),
        _ => None,
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
/// * [Option<64>] - The length of the witness data at `index`.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_witness_data_length;
///
/// fn foo() {
///     let witness_data_length = tx_witness_data_length(0).unwrap();
///     log(witness_data_length);
/// }
/// ```
pub fn tx_witness_data_length(index: u64) -> Option<u64> {
    if index >= tx_witnesses_count() {
        return None
    }

    Some(__gtf::<u64>(index, GTF_WITNESS_DATA_LENGTH))
}

/// Get the witness data at `index`.
///
/// # Additional Information
///
/// **Unsafe. Assumes the type is correct.**
/// This function does not support ownership types(Vec, Bytes, String, etc).
///
/// # Arguments
///
/// * `index` - The index of the witness to get the data for.
///
/// # Returns
///
/// * [Option<T>] - The witness data at `index`.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_witness_data;
///
/// fn foo() {
///     let witness_data: u64 = tx_witness_data(0).unwrap();
///     log(witness_data);
/// }
/// ```
pub fn tx_witness_data<T>(index: u64) -> Option<T> {
    if index >= tx_witnesses_count() {
        return None
    }

    let length = match tx_witness_data_length(index) {
        Some(len) => len,
        None => return None,
    };

    if __is_reference_type::<T>() {
        let witness_data_ptr = __gtf::<raw_ptr>(index, GTF_WITNESS_DATA);
        let new_ptr = alloc_bytes(length);
        witness_data_ptr.copy_bytes_to(new_ptr, length);

        Some(asm(ptr: new_ptr) {
            ptr: T
        })
    } else {
        // u8 is the only value type that is less than 8 bytes and should be handled separately
        if __size_of::<T>() == 1 {
            Some(__gtf::<raw_ptr>(index, GTF_WITNESS_DATA).add::<u8>(7).read::<T>())
        } else {
            Some(__gtf::<raw_ptr>(index, GTF_WITNESS_DATA).read::<T>())
        }
    }
}

/// Get the transaction script start pointer.
///
/// # Returns
///
/// * [Option<raw_ptr>] - The transaction script start pointer.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_script_start_pointer;
///
/// fn foo() {
///     let script_start_pointer = tx_script_start_pointer().unwrap();
///     log(script_start_pointer);
/// }
/// ```
fn tx_script_start_pointer() -> Option<raw_ptr> {
    match tx_type() {
        Transaction::Script => Some(__gtf::<raw_ptr>(0, GTF_SCRIPT_SCRIPT)),
        _ => None,
    }
}

/// Get the transaction script data start pointer.
///
/// # Returns
///
/// * [Option<raw_ptr>] - The transaction script data start pointer.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_script_data_start_pointer;
///
/// fn foo() {
///     let script_data_start_pointer = tx_script_data_start_pointer().unwrap();
///     log(script_data_start_pointer);
/// }
/// ```
fn tx_script_data_start_pointer() -> Option<raw_ptr> {
    match tx_type() {
        Transaction::Script => Some(__gtf::<raw_ptr>(0, GTF_SCRIPT_SCRIPT_DATA)),
        _ => None,
    }
}

/// Get the script data, typed.
///
/// # Additional Information
///
/// **Unsafe. Assumes the type is correct.**
/// This function does not support ownership types(Vec, Bytes, String, etc).
///
/// # Returns
///
/// * [Option<T>] - The script data, typed.
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
pub fn tx_script_data<T>() -> Option<T> {
    let ptr = tx_script_data_start_pointer();
    if ptr.is_none() {
        return None
    }

    // TODO some safety checks on the input data? We are going to assume it is the right type for now.
    Some(ptr.unwrap().read::<T>())
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
/// * [Option<T>] - The script bytecode.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_script_bytecode;
///
/// fn foo() {
///     let script_bytecode: [u64; 64] = tx_script_bytecode().unwrap();
///     log(script_bytecode);
/// }
/// ```
pub fn tx_script_bytecode<T>() -> Option<T> {
    match tx_type() {
        Transaction::Script => Some(tx_script_start_pointer().unwrap().read::<T>()),
        _ => None,
    }
}

/// Get the hash of the script bytecode.
///
/// # Returns
///
/// * [Option<b256>] - The hash of the script bytecode.
///
/// # Examples
///
/// ```sway
/// use std::tx::tx_script_bytecode_hash;
///
/// fn foo() {
///     let script_bytecode_hash: b256 = tx_script_bytecode_hash().unwrap();
///     assert(script_bytecode_hash == 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef);
/// }
/// ```
pub fn tx_script_bytecode_hash() -> Option<b256> {
    match tx_type() {
        Transaction::Script => {
            // Get the script memory details
            let mut result_buffer = b256::zero();
            let script_length = tx_script_length().unwrap();
            let script_ptr = tx_script_start_pointer().unwrap();

            // Run the hash opcode for the script in memory
            Some(
                asm(hash: result_buffer, ptr: script_ptr, len: script_length) {
                    s256 hash ptr len;
                    hash: b256
                },
            )
        },
        _ => None,
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
    asm(ptr: TX_ID_OFFSET) {
        ptr: raw_ptr
    }.read()
}
