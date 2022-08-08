//! Transaction field getters.
library tx;

use ::address::Address;
use ::inputs::{tx_input_type, input_coin_tx_id, input_contract_tx_id};
use ::mem::read;
use ::option::Option;
use ::revert::revert;

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
// const GTF_CREATE_BYTECODE_LENGTH = 0x013;
// const GTF_CREATE_BYTECODE_WITNESS_INDEX = 0x014;
// const GTF_CREATE_STORAGE_SLOTS_COUNT = 0x015;
const GTF_CREATE_INPUTS_COUNT = 0x016;
const GTF_CREATE_OUTPUTS_COUNT = 0x017;
const GTF_CREATE_WITNESSES_COUNT = 0x018;
// const GTF_CREATE_SALT = 0x019;
// const GTF_CREATE_STORAGE_SLOT_AT_INDEX = 0x01A;
const GTF_CREATE_INPUT_AT_INDEX = 0x01B;
const GTF_CREATE_OUTPUT_AT_INDEX = 0x01C;
const GTF_CREATE_WITNESS_AT_INDEX = 0x01D;

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

const GTF_WITNESS_DATA_LENGTH = 0x301;
const GTF_WITNESS_DATA = 0x302;

// Output types
pub const OUTPUT_COIN = 0u8;
pub const OUTPUT_CONTRACT = 1u8;
pub const OUTPUT_MESSAGE = 2u8;
pub const OUTPUT_CHANGE = 3u8;
pub const OUTPUT_VARIABLE = 4u8;
pub const OUTPUT_CONTRACT_CREATED = 5u8;

pub enum Transaction {
    Script: (),
    Create: (),
}

pub enum Input {
    Coin: (),
    Contract: (),
    Message: (),
}

pub enum Output {
    Coin: (),
    Contract: (),
    Message: (),
    Change: (),
    Variable: (),
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
            revert(1)
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
            revert(1)
        }
    }
}

/// Get the transaction outputs count for either tx type
/// (transaction-script or transaction-create).
pub fn tx_outputs_count() -> u64 {
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

// Get witness at index `index`
pub fn script_witness(index: u64) -> b256 {
    // GTF_SCRIPT_WITNESS_AT_INDEX = 0x00F
    read(asm(res, i: index) {
        gtf res i i15;
        res: u64
    })
}

// Get the length of the witness data at `index`
pub fn witness_data_length(index: u64) -> u64 {
    asm(res, i: index) {
        gtf res i i769;
        res: u64
    }
}

// @todo figure out return type
// Get the witness data at `index`.
pub fn witness_data<T>(index: u64) -> T {
    // GTF_WITNESS_DATA = 0x302
    read(asm(res, i: index) {
        gtf res i i770;
        res: u64
    })
}

// const GTF_WITNESS_DATA_LENGTH = 0x301;
// const GTF_WITNESS_DATA = 0x302;

/// Get the transaction receipts root.
/// Reverts if not a transaction-script.
pub fn tx_receipts_root() -> b256 {
    let type = tx_type();
    match type {
        Transaction::Script => {
            let val: b256 = read(__gtf::<u64>(0, GTF_SCRIPT_RECEIPTS_ROOT));
            val
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
    read(tx_script_data_start_pointer())
}

/// Get the script bytecode
/// Must be cast to a u64 array, with sufficient length to contain the bytecode.
/// Bytecode will be padded to next whole word.
pub fn tx_script_bytecode<T>() -> T {
    read(tx_script_start_pointer())
}

/// Get a pointer to an input given the index of the input
/// for either tx type (transaction-script or transaction-create).
pub fn tx_input_pointer(index: u64) -> u64 {
    let type = tx_type();
    match type {
        Transaction::Script => {
            __gtf::<u64>(index, GTF_SCRIPT_INPUT_AT_INDEX)
        },
        Transaction::Create => {
            __gtf::<u64>(index, GTF_CREATE_INPUT_AT_INDEX)
        }
    }
}

/// If the input's type is `InputCoin` or `InputMessage`,
/// return the owner as an Option::Some(owner).
/// Otherwise, returns Option::None.
pub fn tx_input_owner(index: u64) -> Option<Address> {
    let type = tx_input_type(index);
    let owner_ptr = match type {
        Input::Coin => {
            __gtf::<u64>(index, GTF_INPUT_COIN_OWNER)
        },
        Input::Message => {
            __gtf::<u64>(index, GTF_INPUT_MESSAGE_OWNER)
        },
        _ => {
            return Option::None;
        },
    };
    let val: b256 = read(owner_ptr);
    Option::Some(~Address::from(val))
}

////////////////////////////////////////
// Inputs > Predicate
////////////////////////////////////////

/// Get the predicate data pointer from the input at `index`.
/// If the input's type is `InputCoin` or `InputMessage`,
/// return the data as an Option::Some(ptr).
/// Otherwise, returns Option::None.
pub fn predicate_data_pointer(index: u64) -> Option<u64> {
    let type = tx_input_type(index);
    match type {
        Input::Coin => {
            Option::Some(__gtf::<u64>(index, GTF_INPUT_COIN_PREDICATE_DATA))
        },
        Input::Message => {
            Option::Some(__gtf::<u64>(index, GTF_INPUT_MESSAGE_PREDICATE_DATA))
        },
        _ => {
            Option::None
        }
    }
}

pub fn get_predicate_data<T>(index: u64) -> T {
    let data = predicate_data_pointer(index);
    match data {
        Option::Some(d) => {
            read(d)
        },
        Option::None => {
            revert(0)
        },
    }
}

////////////////////////////////////////
// Outputs
////////////////////////////////////////

/// Get a pointer to the Ouput at `index`
/// for either tx type (transaction-script or transaction-create).
pub fn tx_output_pointer(index: u64) -> u64 {
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

/// Get a pointer to the witnex at `index`.
pub fn tx_witness_pointer(index: u64) -> u64 {
    let type = tx_type();
    match type {
        Transaction::Script => {
            read(__gtf::<u64>(index, GTF_SCRIPT_WITNESS_AT_INDEX))
        },
        Transaction::Create => {
            read(__gtf::<u64>(index, GTF_CREATE_WITNESS_AT_INDEX))
        },
    }

}

/// Get the type of an output at `index`.
pub fn tx_output_type(index: u64) -> Output {
    let type = __gtf::<u8>(index, GTF_OUTPUT_TYPE);
    match type {
        0u8 => {
            Output::Coin
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

/// Get the amount of coins to send for the output at `index`.
/// This method is only meaningful if the output type has the `amount` field.
/// Specifically: OutputCoin, OutputMessage, OutputChange, OutputVariable.
pub fn tx_output_amount(index: u64) -> u64 {
    let type = tx_output_type(index);
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
        // ues GTF_OUTPUT_MESSAGE_AMOUNT as there's no simlar const for OutputChange
        Output::Change => {
            __gtf::<u64>(index, GTF_OUTPUT_MESSAGE_AMOUNT)
        },
        // use GTF_OUTPUT_MESSAGE_AMOUNT as there's no simlar const for OutputVariable
        Output::Variable => {
            __gtf::<u64>(index, GTF_OUTPUT_MESSAGE_AMOUNT)
        },
    }
}

/// Get the id of the current transaction.
/// If the input's type is `InputCoin` or `InputContract`,
/// return the data as an Option::Some(b256).
/// Otherwise, returns Option::None.
pub fn tx_id(index: u64) -> Option<b256> {
    let type = tx_input_type(index);
    match type {
        Input::Coin => {
            let val: b256 = read(__gtf::<u64>(index, GTF_INPUT_COIN_TX_ID));
            Option::Some(val)
        },
        Input::Contract => {
            let val: b256 = read(__gtf::<u64>(index, GTF_INPUT_CONTRACT_TX_ID));
            Option::Some(val)
        },
        _ => {
           Option::None
        },
    }
}
