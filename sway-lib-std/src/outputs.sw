//! Getters for fields on transaction outputs.
//! This includes `Output::Coins`, `Input::Messages` and `Input::Contracts`.
library;

use ::address::Address;
use ::asset_id::AssetId;
use ::contract_id::ContractId;
use ::revert::revert;
use ::tx::{
    GTF_CREATE_OUTPUT_AT_INDEX,
    GTF_CREATE_OUTPUTS_COUNT,
    GTF_SCRIPT_OUTPUT_AT_INDEX,
    GTF_SCRIPT_OUTPUTS_COUNT,
    Transaction,
    tx_type,
};
use ::option::Option::{self, *};

// GTF Opcode const selectors
//
pub const GTF_OUTPUT_TYPE = 0x300;
pub const GTF_OUTPUT_COIN_TO = 0x301;
pub const GTF_OUTPUT_COIN_AMOUNT = 0x302;
pub const GTF_OUTPUT_COIN_ASSET_ID = 0x303;
// pub const GTF_OUTPUT_CONTRACT_INPUT_INDEX = 0x304;
// pub const GTF_OUTPUT_CONTRACT_BALANCE_ROOT = 0x305;
// pub const GTF_OUTPUT_CONTRACT_STATE_ROOT = 0x306;
// pub const GTF_OUTPUT_CONTRACT_CREATED_CONTRACT_ID = 0x307;
// pub const GTF_OUTPUT_CONTRACT_CREATED_STATE_ROOT = 0x308;

/// The output type for a transaction.
pub enum Output {
    /// A coin output.
    Coin: (),
    /// A contract output.
    Contract: (),
    /// Remaining "change" from spending of a coin.
    Change: (),
    /// A variable output.
    Variable: (),
    /// A contract deployment.
    ContractCreated: (),
}

/// Get the type of an output at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the output to get the type of.
///
/// # Returns
///
/// * [Option<Output>] - The type of the output at `index`.
///
/// # Examples
///
/// ```sway
/// use std::outputs::output_type;
///
/// fn foo() {
///     let output_type = output_type(0).unwrap();
///     match output_type {
///         Output::Coin => { log("The output is a coin") },
///         Output::Contract => { log("The output is a contract") },
///         Output::Change => { log("The output is change") },
///         Output::Variable => { log("The output is a variable") },
///         Output::ContractCreated => { log("The output is a contract creation") },
///     };
/// }
/// ```
pub fn output_type(index: u64) -> Option<Output> {
    if index >= output_count().as_u64() {
        return None
    }

    match __gtf::<u8>(index, GTF_OUTPUT_TYPE) {
        0u8 => Some(Output::Coin),
        1u8 => Some(Output::Contract),
        2u8 => Some(Output::Change),
        3u8 => Some(Output::Variable),
        4u8 => Some(Output::ContractCreated),
        _ => None,
    }
}

/// Get a pointer to the output at `index`
/// for either `tx_type` (transaction-script or transaction-create).
///
/// # Arguments
///
/// * `index`: [u64] - The index of the output to get the pointer to.
///
/// # Returns
///
/// * [Option<raw_ptr>] - A pointer to the output at `index`.
///
/// # Examples
///
/// ```sway
/// use std::outputs::output_pointer;
///
/// fn foo() {
///     let output_pointer = output_pointer(0).unwrap();
/// }
/// ```
fn output_pointer(index: u64) -> Option<raw_ptr> {
    if output_type(index).is_none() {
        return None
    }

    match tx_type() {
        Transaction::Script => Some(__gtf::<raw_ptr>(index, GTF_SCRIPT_OUTPUT_AT_INDEX)),
        Transaction::Create => Some(__gtf::<raw_ptr>(index, GTF_CREATE_OUTPUT_AT_INDEX)),
        Transaction::Upgrade => Some(__gtf::<raw_ptr>(index, GTF_SCRIPT_OUTPUT_AT_INDEX)),
        Transaction::Upload => Some(__gtf::<raw_ptr>(index, GTF_SCRIPT_OUTPUT_AT_INDEX)),
        Transaction::Blob => Some(__gtf::<raw_ptr>(index, GTF_SCRIPT_OUTPUT_AT_INDEX)),
        _ => None,
    }
}

/// Get the transaction outputs count for either `tx_type`
/// (transaction-script or transaction-create).
///
/// # Returns
///
/// * [u16] - The transaction outputs count.
///
/// # Reverts
///
/// * When the output type is unrecognized. This should never happen.
///
/// # Examples
///
/// ```sway
/// use std::outputs::output_count;
///
/// fn foo() {
///     let output_count = output_count();
///     log(output_count);
/// }
/// ```
pub fn output_count() -> u16 {
    match tx_type() {
        Transaction::Script => __gtf::<u16>(0, GTF_SCRIPT_OUTPUTS_COUNT),
        Transaction::Create => __gtf::<u16>(0, GTF_CREATE_OUTPUTS_COUNT),
        Transaction::Upgrade => __gtf::<u16>(0, GTF_SCRIPT_OUTPUTS_COUNT),
        Transaction::Upload => __gtf::<u16>(0, GTF_SCRIPT_OUTPUTS_COUNT),
        Transaction::Blob => __gtf::<u16>(0, GTF_SCRIPT_OUTPUTS_COUNT),
        _ => revert(0),
    }
}

/// The amount of coins to send to the output at `index`.
///
/// # Additional Information
///
/// This method is only meaningful if the `Output` type has the `amount` field,
/// specifically: `Output::Coin`, `Output::Change` & `Output::Variable`.
///
/// For now, output changes are always guaranteed to have an amount of
/// zero since they're only set after execution terminates.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the output to get the amount of.
///
/// # Returns
///
/// * [Option<u64>] - The amount of coins to send to the output at `index`.
///
/// # Examples
///
/// ```sway
/// use std::outputs::output_amount;
///
/// fn foo() {
///     let output_amount = output_amount(0);
///     log(output_amount);
/// }
/// ```
pub fn output_amount(index: u64) -> Option<u64> {
    match output_type(index) {
        Some(Output::Coin) => Some(__gtf::<u64>(index, GTF_OUTPUT_COIN_AMOUNT)),
        Some(Output::Contract) => None,
        // For now, output changes are always guaranteed to have an amount of
        // zero since they're only set after execution terminates.
        Some(Output::Change) => Some(0),
        Some(Output::Variable) => {
            let ptr = output_pointer(index).unwrap();
            Some(
                asm(r1, r2, r3: ptr) {
                    addi r2 r3 i40;
                    lw r1 r2 i0;
                    r1: u64
                },
            )
        },
        Some(Output::ContractCreated) => None,
        None => None,
    }
}

/// Gets the AssetId of the output.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the output to get the AssetId of.
///
/// # Returns
///
/// * [Option<AssetId>] - The AssetId of the output. None otherwise.
///
/// # Reverts
///
/// * When the output type is unrecognized. This should never happen.
///
/// # Examples
///
/// ```sway
/// use std::outputs::output_asset_id;
///
/// fn foo() {
///     let output_asset_id = output_asset_id(0);
///     log(output_asset_id);
/// }
/// ```
pub fn output_asset_id(index: u64) -> Option<AssetId> {
    match output_type(index) {
        Some(Output::Coin) => Some(AssetId::from(__gtf::<b256>(index, GTF_OUTPUT_COIN_ASSET_ID))),
        Some(Output::Change) => Some(AssetId::from(__gtf::<b256>(index, GTF_OUTPUT_COIN_ASSET_ID))),
        _ => None,
    }
}

/// Returns the receiver of the output.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the output to get the receiver of.
///
/// # Returns
///
/// * [Option<Address>] - The receiver of the output. None otherwise.
///
/// # Reverts
///
/// * When the output type is unrecognized. This should never happen.
///
/// # Examples
///
/// ```sway
/// use std::outputs::output_asset_to;
///
/// fn foo() {
///     let output_receiver = output_asset_to(0);
///     log(output_receiver);
/// }
/// ```
pub fn output_asset_to(index: u64) -> Option<Address> {
    match output_type(index) {
        Some(Output::Coin) => Some(__gtf::<Address>(index, GTF_OUTPUT_COIN_TO)),
        Some(Output::Change) => Some(__gtf::<Address>(index, GTF_OUTPUT_COIN_TO)),
        _ => None,
    }
}

impl core::ops::Eq for Output {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (Output::Coin, Output::Coin) => true,
            (Output::Contract, Output::Contract) => true,
            (Output::Change, Output::Change) => true,
            (Output::Variable, Output::Variable) => true,
            (Output::ContractCreated, Output::ContractCreated) => true,
            _ => false,
        }
    }
}
