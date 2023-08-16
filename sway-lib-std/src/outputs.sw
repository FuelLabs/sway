//! Getters for fields on transaction outputs.
//! This includes `Output::Coins`, `Input::Messages` and `Input::Contracts`.
library;

use ::alias::AssetId;
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
use ::option::*;

// GTF Opcode const selectors
//
pub const GTF_OUTPUT_TYPE = 0x201;
pub const GTF_OUTPUT_COIN_TO = 0x202;
pub const GTF_OUTPUT_COIN_AMOUNT = 0x203;
pub const GTF_OUTPUT_COIN_ASSET_ID = 0x204;
// pub const GTF_OUTPUT_CONTRACT_INPUT_INDEX = 0x205;
// pub const GTF_OUTPUT_CONTRACT_BALANCE_ROOT = 0x206;
// pub const GTF_OUTPUT_CONTRACT_STATE_ROOT = 0x207;
// pub const GTF_OUTPUT_CONTRACT_CREATED_CONTRACT_ID = 0x208;
// pub const GTF_OUTPUT_CONTRACT_CREATED_STATE_ROOT = 0x209;

/// The output type for a transaction.
pub enum Output {
    Coin: (),
    Contract: (),
    Change: (),
    Variable: (),
}

/// Get the type of an output at `index`.
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
pub fn output_pointer(index: u64) -> u64 {
    match tx_type() {
        Transaction::Script => __gtf::<u64>(index, GTF_SCRIPT_OUTPUT_AT_INDEX),
        Transaction::Create => __gtf::<u64>(index, GTF_CREATE_OUTPUT_AT_INDEX),
    }
}

/// Get the transaction outputs count for either `tx_type`
/// (transaction-script or transaction-create).
pub fn output_count() -> u64 {
    match tx_type() {
        Transaction::Script => __gtf::<u64>(0, GTF_SCRIPT_OUTPUTS_COUNT),
        Transaction::Create => __gtf::<u64>(0, GTF_CREATE_OUTPUTS_COUNT),
    }
}

/// Get the amount of coins to send for the output at `index`.
/// This method is only meaningful if the `Output` type has the `amount` field,
/// specifically: `Output::Coin`, `Output::Change` & `Output::Variable`.
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

/// If the output's type is `Output::Coin` return the asset ID as an `Some(id)`.
/// Otherwise, returns `None`.
pub fn output_asset_id(index: u64) -> Option<AssetId> {
    match output_type(index) {
        Output::Coin => Option::Some(__gtf::<b256>(index, GTF_OUTPUT_COIN_ASSET_ID)),
        _ => Option::None,
    }
}

/// If the output's type is `Output::Coin` return the b256 as `Some(to)`.
/// Otherwise, returns `None`.
/// TODO: Update to `Identity` when https://github.com/FuelLabs/sway/issues/4569 is resolved
pub fn output_asset_to(index: u64) -> Option<b256> {
    match output_type(index) {
        Output::Coin => Option::Some(__gtf::<b256>(index, GTF_OUTPUT_COIN_TO)),
        _ => Option::None,
    }
}
