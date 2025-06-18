//! Getters for fields on transaction outputs.
//! This includes `Output::Coin`, `Output::Contract`, `Output::Change`, `Output::Variable`, and `Output::ContractCreated`.
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
    GTF_TYPE,
    Transaction,
    tx_type,
    TX_TYPE_CREATE,
    TX_TYPE_MINT,
};
use ::option::Option::{self, *};
use ::hash::{Hash, Hasher};
use ::ops::*;
use ::primitive_conversions::u16::*;
use ::raw_ptr::*;
use ::codec::*;
use ::debug::*;

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

const OUTPUT_VARIABLE_ASSET_ID_OFFSET = 48;
const OUTPUT_VARIABLE_TO_OFFSET = 8;

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

impl PartialEq for Output {
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
impl Eq for Output {}

impl Hash for Output {
    fn hash(self, ref mut state: Hasher) {
        match self {
            Self::Coin => {
                0_u8.hash(state);
            },
            Self::Contract => {
                1_u8.hash(state);
            },
            Self::Change => {
                2_u8.hash(state);
            },
            Self::Variable => {
                3_u8.hash(state);
            },
            Self::ContractCreated => {
                4_u8.hash(state);
            },
        }
    }
}

const OUTPUT_TYPE_COIN: u8 = 0;
const OUTPUT_TYPE_CONTRACT: u8 = 1;
const OUTPUT_TYPE_CHANGE: u8 = 2;
const OUTPUT_TYPE_VARIABLE: u8 = 3;
const OUTPUT_TYPE_CONTRACT_CREATED: u8 = 4;

/// Returns the `u8` type id of the output at `index` if such output exists,
/// or a non-existing type id if the `index` is out of output bounds.
///
/// This private function is used to avoid the overhead of creating and
/// inspecting `Option`s for the output type.
fn output_type_id(index: u64) -> u8 {
    if index < output_count().as_u64() {
        __gtf::<u8>(index, GTF_OUTPUT_TYPE)
    } else {
        u8::max()
    }
}

/// Get the type of the output at `index`.
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
    match output_type_id(index) {
        OUTPUT_TYPE_COIN => Some(Output::Coin),
        OUTPUT_TYPE_CONTRACT => Some(Output::Contract),
        OUTPUT_TYPE_CHANGE => Some(Output::Change),
        OUTPUT_TYPE_VARIABLE => Some(Output::Variable),
        OUTPUT_TYPE_CONTRACT_CREATED => Some(Output::ContractCreated),
        _ => None,
    }
}

/// Returns the pointer to the output at `index`.
///
/// This private function **does not check if the `index` is out of bounds**.
/// It assumes that the caller has already checked the output count.
fn output_pointer(index: u64) -> raw_ptr {
    match __gtf::<u8>(0, GTF_TYPE) {
        TX_TYPE_CREATE => __gtf::<raw_ptr>(index, GTF_CREATE_OUTPUT_AT_INDEX),
        TX_TYPE_MINT => revert(0),
        _ => __gtf::<raw_ptr>(index, GTF_SCRIPT_OUTPUT_AT_INDEX),
    }
}

/// Gets the transaction outputs count.
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
    match __gtf::<u8>(0, GTF_TYPE) {
        TX_TYPE_CREATE => __gtf::<u16>(0, GTF_CREATE_OUTPUTS_COUNT),
        TX_TYPE_MINT => revert(0),
        _ => __gtf::<u16>(0, GTF_SCRIPT_OUTPUTS_COUNT),
    }
}

/// The amount of coins to send to the output at `index`.
///
/// # Additional Information
///
/// This method is only meaningful if the `Output` type has the `amount` field,
/// specifically: `Output::Coin`, `Output::Change`, and `Output::Variable`.
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
    match output_type_id(index) {
        OUTPUT_TYPE_COIN => Some(__gtf::<u64>(index, GTF_OUTPUT_COIN_AMOUNT)),
        OUTPUT_TYPE_CHANGE => Some(0),
        OUTPUT_TYPE_VARIABLE => {
            let ptr = output_pointer(index);
            Some(
                asm(r1, r2, r3: ptr) {
                    addi r2 r3 i40;
                    lw r1 r2 i0;
                    r1: u64
                },
            )
        },
        _ => None,
    }
}

/// Gets the asset id of the output at `index`.
///
/// If you want to get the asset id and the receiver of the output,
/// use `output_asset_id_and_to` instead.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the output to get the asset id of.
///
/// # Returns
///
/// * [Option<AssetId>] - The asset id of the output.
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
    match output_type_id(index) {
        OUTPUT_TYPE_COIN | OUTPUT_TYPE_CHANGE => Some(AssetId::from(__gtf::<b256>(index, GTF_OUTPUT_COIN_ASSET_ID))),
        OUTPUT_TYPE_VARIABLE => {
            let ptr = output_pointer(index);
            Some(AssetId::from(ptr.add_uint_offset(OUTPUT_VARIABLE_ASSET_ID_OFFSET).read::<b256>()))
        },
        _ => None,
    }
}

/// Returns the receiver of the output at `index`.
///
/// If you want to get the asset id and the receiver of the output,
/// use `output_asset_id_and_to` instead.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the output to get the receiver of.
///
/// # Returns
///
/// * [Option<Address>] - The receiver of the output.
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
    match output_type_id(index) {
        OUTPUT_TYPE_COIN | OUTPUT_TYPE_CHANGE => Some(__gtf::<Address>(index, GTF_OUTPUT_COIN_TO)),
        OUTPUT_TYPE_VARIABLE => {
            let ptr = output_pointer(index);
            Some(Address::from(ptr.add_uint_offset(OUTPUT_VARIABLE_TO_OFFSET).read::<b256>()))
        },
        _ => None,
    }
}

/// Gets the asset id and the receiver of the output at `index`.
///
/// # Arguments
///
/// * `index`: [u64] - The index of the output to get the asset id and the receiver of.
///
/// # Returns
///
/// * [Option<AssetId, Address>] - The asset id and the receiver of the output.
///
/// # Examples
///
/// ```sway
/// use std::outputs::output_asset_id_and_to;
///
/// fn foo() {
///     let (output_asset_id, output_receiver) = output_asset_id_and_to(0);
///     log(output_asset_id);
///     log(output_receiver);
/// }
/// ```
pub fn output_asset_id_and_to(index: u64) -> Option<(AssetId, Address)> {
    match output_type_id(index) {
        OUTPUT_TYPE_COIN | OUTPUT_TYPE_CHANGE => Some((
            AssetId::from(__gtf::<b256>(index, GTF_OUTPUT_COIN_ASSET_ID)),
            __gtf::<Address>(index, GTF_OUTPUT_COIN_TO),
        )),
        OUTPUT_TYPE_VARIABLE => {
            let ptr = output_pointer(index);
            Some((
                AssetId::from(ptr.add_uint_offset(OUTPUT_VARIABLE_ASSET_ID_OFFSET).read::<b256>()),
                Address::from(ptr.add_uint_offset(OUTPUT_VARIABLE_TO_OFFSET).read::<b256>()),
            ))
        },
        _ => None,
    }
}
