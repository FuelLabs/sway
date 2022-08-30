//! Getters for fields on transaction outputs.
//! This includes OutputCoins, InputMessages and InputContracts.
library outputs;

use ::contract_id::ContractId;
use ::mem::read;
use ::revert::revert;
use ::logging::log;
use ::tx::{
    GTF_CREATE_OUTPUTS_COUNT,
    GTF_CREATE_OUTPUT_AT_INDEX,
    GTF_SCRIPT_OUTPUTS_COUNT,
    GTF_SCRIPT_OUTPUT_AT_INDEX,
    Transaction,
    tx_type,
};

////////////////////////////////////////
// GTF Opcode const selectors
////////////////////////////////////////

const GTF_OUTPUT_TYPE = 0x201;
// const GTF_OUTPUT_COIN_TO = 0x202;
const GTF_OUTPUT_COIN_AMOUNT = 0x203;
// const GTF_OUTPUT_COIN_ASSET_ID = 0x204;
// const GTF_OUTPUT_CONTRACT_INPUT_INDEX = 0x205;
// const GTF_OUTPUT_CONTRACT_BALANCE_ROOT = 0x206;
// const GTF_OUTPUT_CONTRACT_STATE_ROOT = 0x207;
// const GTF_OUTPUT_MESSAGE_RECIPIENT = 0x208;
const GTF_OUTPUT_MESSAGE_AMOUNT = 0x209;
// const GTF_OUTPUT_CONTRACT_CREATED_CONTRACT_ID = 0x20A;
// const GTF_OUTPUT_CONTRACT_CREATED_STATE_ROOT = 0x20B;

pub enum Output {
    Coin: (),
    Contract: (),
    Message: (),
    Change: (),
    Variable: (),
}

/// Get the type of an output at `index`.
pub fn output_type(index: u64) -> Output {
    let type = __gtf::<u64>(index, GTF_OUTPUT_TYPE);
    log(type);
    match type {
        0u8 => {
            Output::Coin
        },
        1u8 => {
            Output::Contract
        },
        2u8 => {
            Output::Message
        },
        3u8 => {
            Output::Change
        },
        4u8 => {
            Output::Variable
        },
        _ => {
            revert(0);
        },
    }
}

/// Get a pointer to the Ouput at `index`
/// for either tx type (transaction-script or transaction-create).
pub fn output_pointer(index: u64) -> u64 {
    let type = tx_type();
    match type {
        Transaction::Script => {
            __gtf::<u64>(index, GTF_SCRIPT_OUTPUT_AT_INDEX)
        },
        Transaction::Create => {
            __gtf::<u64>(index, GTF_CREATE_OUTPUT_AT_INDEX)
        },
    }
}

/// Get the transaction outputs count for either tx type
/// (transaction-script or transaction-create).
pub fn output_count() -> u64 {
    let type = tx_type();
    match type {
        Transaction::Script => {
            __gtf::<u64>(0, GTF_SCRIPT_OUTPUTS_COUNT)
        },
        Transaction::Create => {
            __gtf::<u64>(0, GTF_CREATE_OUTPUTS_COUNT)
        },
    }
}

/// Get the amount of coins to send for the output at `index`.
/// This method is only meaningful if the output type has the `amount` field.
/// Specifically: OutputCoin, OutputMessage, OutputChange, OutputVariable.
pub fn output_amount(index: u64) -> u64 {
    let type = output_type(index);
    match type {
        Output::Coin => {
            __gtf::<u64>(index, GTF_OUTPUT_COIN_AMOUNT)
        },
        Output::Contract => {
            revert(0);
        },
        Output::Message => {
            __gtf::<u64>(index, GTF_OUTPUT_MESSAGE_AMOUNT)
        },
        // Output changes are always guaranteed to have an amount of zero since
        // they're only set after execution terminates
        Output::Change => {
            0
        },
        // use `__gtf` when GTF_OUTPUT_VARIABLE_AMOUNT is available
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
