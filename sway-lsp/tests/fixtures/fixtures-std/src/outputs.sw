//! Getters for fields on transaction outputs.
//! This includes `Output::Coins`, `Input::Messages` and `Input::Contracts`.
library;

use ::contract_id::{AssetId, ContractId};
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
<<<<<<< Updated upstream
pub const GTF_OUTPUT_TYPE = 0x201;
pub const GTF_OUTPUT_COIN_TO = 0x202;
pub const GTF_OUTPUT_COIN_AMOUNT = 0x203;
pub const GTF_OUTPUT_COIN_ASSET_ID = 0x204;
// pub const GTF_OUTPUT_CONTRACT_INPUT_INDEX = 0x205;
// pub const GTF_OUTPUT_CONTRACT_BALANCE_ROOT = 0x206;
// pub const GTF_OUTPUT_CONTRACT_STATE_ROOT = 0x207;
// pub const GTF_OUTPUT_CONTRACT_CREATED_CONTRACT_ID = 0x208;
// pub const GTF_OUTPUT_CONTRACT_CREATED_STATE_ROOT = 0x209;
=======
pub const GTF_OUTPUT_TYPE = 0x300;
pub const GTF_OUTPUT_COIN_TO = 0x301;
pub const GTF_OUTPUT_COIN_AMOUNT = 0x302;
pub const GTF_OUTPUT_COIN_ASSET_ID = 0x303;
// pub const GTF_OUTPUT_CONTRACT_INPUT_INDEX = 0x304;
// pub const GTF_OUTPUT_CONTRACT_BALANCE_ROOT = 0x305;
// pub const GTF_OUTPUT_CONTRACT_STATE_ROOT = 0x306;
// pub const GTF_OUTPUT_CONTRACT_CREATED_CONTRACT_ID = 0x307;
// pub const GTF_OUTPUT_CONTRACT_CREATED_STATE_ROOT = 0x308;
>>>>>>> Stashed changes

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
}

/// Get the type of an output at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the output to get the type of.
///
/// # Returns
///
/// * [Output] - The type of the output at `index`.
///
/// # Reverts
///
/// * When the output type is unrecognized. This should never happen.
<<<<<<< Updated upstream
/// 
=======
///
>>>>>>> Stashed changes
/// # Examples
///
/// ```sway
/// use std::outputs::output_type;
///
/// fn foo() {
///     let output_type = output_type(0);
///     match output_type {
///         Output::Coin => { log("The output is a coin") },
///         Output::Contract => { log("The output is a contract") },
///         Output::Change => { log("The output is change") },
///         Output::Variable => { log("The output is a variable") },
///     };
/// }
/// ```
pub fn output_type(index: u64) -> Output {
    match __gtf::<u8>(index, GTF_OUTPUT_TYPE) {
        0u8 => Output::Coin,
        1u8 => Output::Contract,
        2u8 => Output::Change,
        3u8 => Output::Variable,
        _ => revert(0),
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
/// * [u64] - A pointer to the output at `index`.
///
/// # Reverts
///
/// * When the output type is unrecognized. This should never happen.
///
/// # Examples
///
/// ```sway
/// use std::outputs::output_pointer;
///
/// fn foo() {
///     let output_pointer = output_pointer(0);
///     log(output_pointer);
/// }
/// ```
pub fn output_pointer(index: u64) -> u64 {
    match tx_type() {
        Transaction::Script => __gtf::<u64>(index, GTF_SCRIPT_OUTPUT_AT_INDEX),
        Transaction::Create => __gtf::<u64>(index, GTF_CREATE_OUTPUT_AT_INDEX),
    }
}

/// Get the transaction outputs count for either `tx_type`
/// (transaction-script or transaction-create).
///
/// # Returns
///
/// * [u64] - The transaction outputs count.
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
pub fn output_count() -> u64 {
    match tx_type() {
        Transaction::Script => __gtf::<u64>(0, GTF_SCRIPT_OUTPUTS_COUNT),
        Transaction::Create => __gtf::<u64>(0, GTF_CREATE_OUTPUTS_COUNT),
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
/// * [u64] - The amount of coins to send to the output at `index`.
///
/// # Reverts
///
/// * When the output type is `Output::Contract`.
/// * When the output type is unrecognized. This should never happen.
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
pub fn output_amount(index: u64) -> u64 {
    match output_type(index) {
        Output::Coin => __gtf::<u64>(index, GTF_OUTPUT_COIN_AMOUNT),
        Output::Contract => revert(0),
        // For now, output changes are always guaranteed to have an amount of
        // zero since they're only set after execution terminates.
        // use `__gtf` when GTF_OUTPUT_CHANGE_AMOUNT is available.
        // See https://github.com/FuelLabs/fuel-specs/issues/402
        // and https://github.com/FuelLabs/sway/issues/2671.
        Output::Change => 0,
        // use `__gtf` when GTF_OUTPUT_VARIABLE_AMOUNT is available.
        // See https://github.com/FuelLabs/fuel-specs/issues/402
        // and https://github.com/FuelLabs/sway/issues/2671.
        Output::Variable => {
            let ptr = output_pointer(index);
            asm(r1, r2, r3: ptr) {
                addi r2 r3 i40;
                lw r1 r2 i0;
                r1: u64
            }
        },
    }
}

/// Gets the AssetId of the output if it is a `Output::Coin`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the output to get the AssetId of.
///
/// # Returns
///
/// * [Option<AssetId>] - The AssetId of the output if it is a `Output::Coin`. None otherwise.
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
        Output::Coin => Some(AssetId::from(__gtf::<b256>(index, GTF_OUTPUT_COIN_ASSET_ID))),
        _ => None,
    }
}

// TODO: Update to `Identity` when https://github.com/FuelLabs/sway/issues/4569 is resolved
/// Returns the reciever of the output if it is a `Output::Coin`. Represents the reciever as a `b256`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the output to get the reciever of.
///
/// # Returns
///
/// * [Option<b256>] - The reciever of the output if it is a `Output::Coin`. None otherwise.
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
///     let output_reciever = output_asset_to(0);
///     log(output_reciever);
/// }
/// ```
pub fn output_asset_to(index: u64) -> Option<b256> {
    match output_type(index) {
        Output::Coin => Some(__gtf::<b256>(index, GTF_OUTPUT_COIN_TO)),
        _ => None,
    }
}
