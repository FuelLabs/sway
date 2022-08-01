//! Transaction field getters.
//! This will be replaced by instructions: https://github.com/FuelLabs/fuel-specs/issues/287
library tx;

use ::address::Address;
use ::context::registers::instrs_start;
use ::contract_id::ContractId;
use ::intrinsics::is_reference_type;
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

const GTF_INPUT_TYPE = 0x101;
const GTF_INPUT_COIN_TX_ID = 0x102;
// const GTF_INPUT_COIN_OUTPUT_INDEX = 0x103;
const GTF_INPUT_COIN_OWNER = 0x104;
// const GTF_INPUT_COIN_AMOUNT = 0x105;
// const GTF_INPUT_COIN_ASSET_ID = 0x106;
// const GTF_INPUT_COIN_TX_POINTER = 0x107;
// const GTF_INPUT_COIN_WITNESS_INDEX = 0x108;
// const GTF_INPUT_COIN_MATURITY = 0x109;
// const GTF_INPUT_COIN_PREDICATE_LENGTH = 0x10A;
// const GTF_INPUT_COIN_PREDICATE_DATA_LENGTH = 0x10B;
// const GTF_INPUT_COIN_PREDICATE = 0x10C;
const GTF_INPUT_COIN_PREDICATE_DATA = 0x10D;

const GTF_INPUT_CONTRACT_TX_ID = 0x10E;
// const GTF_INPUT_CONTRACT_OUTPUT_INDEX = 0x10F;
// const GTF_INPUT_CONTRACT_BALANCE_ROOT = 0x110;
// const GTF_INPUT_CONTRACT_STATE_ROOT = 0x111;
// const GTF_INPUT_CONTRACT_TX_POINTER = 0x112;
// const GTF_INPUT_CONTRACT_CONTRACT_ID = 0x113;
// const GTF_INPUT_MESSAGE_MESSAGE_ID = 0x114;
// const GTF_INPUT_MESSAGE_SENDER = 0x115;
// const GTF_INPUT_MESSAGE_RECIPIENT = 0x116;
// const GTF_INPUT_MESSAGE_AMOUNT = 0x117;
// const GTF_INPUT_MESSAGE_NONCE = 0x118;

const GTF_INPUT_MESSAGE_OWNER = 0x119;
// const GTF_INPUT_MESSAGE_WITNESS_INDEX = 0x11A;
// const GTF_INPUT_MESSAGE_DATA_LENGTH = 0x11B;
// const GTF_INPUT_MESSAGE_PREDICATE_LENGTH = 0x11C;
// const GTF_INPUT_MESSAGE_PREDICATE_DATA_LENGTH = 0x11D;
// const GTF_INPUT_MESSAGE_DATA = 0x11E;
// const GTF_INPUT_MESSAGE_PREDICATE = 0x11F;
const GTF_INPUT_MESSAGE_PREDICATE_DATA = 0x120;

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

// const GTF_WITNESS_DATA_LENGTH = 0x301;
// const GTF_WITNESS_DATA = 0x302;

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

// @todo make generic version of tx_witness_at_index()

enum Transaction {
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
            Option::Some(__gtf::<u64>(0, GTF_SCRIPT_SCRIPT_DATA_LENGTH))
        },
        Transaction::Create => {
            revert(1)
        }
    }
}

/// Get the transaction inputs count for either tx type
/// (transaction-script or transaction-create).
pub fn tx_inputs_count() -> u64 {
    let type = tx_type();
    match type {
        Transaction::Script => {
            __gtf::<u64>(0, GTF_SCRIPT_INPUTS_COUNT)
        },
        Transaction::Create => {
            __gtf::<u64>(0, GTF_CREATE_INPUTS_COUNT)
        },
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

/// Get the type of the input at `index`.
pub fn tx_input_type(index: u64) -> u8 {
    __gtf::<u8>(index, GTF_INPUT_TYPE)
}

/// If the input's type is `InputCoin` or `InputMessage`,
/// return the owner as an Option::Some(owner).
/// Otherwise, returns Option::None.
pub fn tx_input_owner(index: u64) -> Option<Address> {
    let type = tx_input_type(index);
    let owner_ptr = match type {
        // 0 is the `Coin` Input type
        0u8 => {
            __gtf::<u64>(index, GTF_INPUT_COIN_OWNER)
        },
        // 2 is the `Message` Input type
        2u8 => {
            __gtf::<u64>(index, GTF_INPUT_MESSAGE_OWNER)
        },
        _ => {
            return Option::None;
        },
    };

    Option::Some(~Address::from(read::<b256>(
        owner_ptr,
        0
    )))
}

////////////////////////////////////////
// Inputs > Predicate
////////////////////////////////////////

/// Get the predicate dats from the input at `index`.
/// If the input's type is `InputCoin` or `InputMessage`,
/// return the data as an Option::Some(T).
/// Otherwise, returns Option::None.
pub fn predicate_data<T>(index: u64) -> T {
    let type = tx_input_type(index);
    let ptr = match type {
        // 0 is the `Coin` Input type
        0u8 => {
            Option::Some(read::<T>(__gtf::<u64>(index, GTF_INPUT_COIN_PREDICATE_DATA)))
        },
        // 2 is the `Message` Input type
        2u8 => {
            Option::Some(read::<T>(__gtf::<u64>(index, GTF_INPUT_MESSAGE_PREDICATE_DATA)))
        },
        _ => {
            return Option::None;
        },
    };
}

////////////////////////////////////////
// Outputs
////////////////////////////////////////

/// Get a pointer to the input at `index`.
/// If the input's type is `InputCoin` or `InputMessage`,
/// return the data as an Option::Some(u64).
/// Otherwise, returns Option::None.
pub fn tx_output_pointer(index: u64) -> Option<u64> {
    let type = tx_type();
    match type {
        Transaction::Script => {
            read::<T>(__gtf::<u64>(index, GTF_SCRIPT_OUTPUT_AT_INDEX))
        },
        Transaction::Create => {
            read::<T>(__gtf::<u64>(index, GTF_CREATE_OUTPUT_AT_INDEX))
        },
    }
}

// @review and add docs
// https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/tx_format.md#witness
pub fn tx_witness(index: u64) -> T {
    let type = tx_type();
    match type {
        Transaction::Script => {
            read::<T>(__gtf::<u64>(index, GTF_SCRIPT_WITNESS_AT_INDEX))
        },
        Transaction::Create => {
            read::<T>(__gtf::<u64>(index, GTF_CREATE_WITNESS_AT_INDEX))
        },
    }

}

/// Get the type of an output at `index`.
pub fn tx_output_type(index: u64) -> u8 {
    __gtf::<u8>(index, GTF_OUTPUT_TYPE)
}

/// Get the amount of coins to send for the output at `index`.
/// This method is only meaningful if the output type has the `amount` field.
/// Specifically: OutputCoin, OutputMessage, OutputChange, OutputVariable.
pub fn tx_output_amount(index: u64) -> Option<u64> {
    let type = tx_output_type(index);
    match type {
        // 0 is the `Coin` Output type
        0u8 => {
            Option::Some(__gtf::<u64>(index, GTF_OUTPUT_COIN_AMOUNT))
        },
        // 2 is the `Message` Output type
        2u8 => {
            Option::Some(__gtf::<u64>(index, GTF_OUTPUT_MESSAGE_AMOUNT))
        },
        // 3 is the `Change` Output type
        // reusing GTF_OUTPUT_MESSAGE_AMOUNT as there's no simlar const for OutputChange
        3u8 => {
            // GTF_OUTPUT_MESSAGE_AMOUNT = 0x209
            Option::Some(__gtf::<u64>(index, GTF_OUTPUT_MESSAGE_AMOUNT))
        },
        // 4 is the `Variable` Output type
        // reusing GTF_OUTPUT_MESSAGE_AMOUNT as there's no simlar const for OutputVariable
        4u8 => {
            Option::Some(__gtf::<u64>(index, GTF_OUTPUT_MESSAGE_AMOUNT))
        },
        _ => {
            Option::None
        },
    }
}

/// Get the id of the current transaction.
pub fn tx_id(index: u64) -> Option<b256> {
    let type = tx_output_type(index);
    match type {
        // 0 is the `Coin` Input type
        0u8 => {
            Option::Some(read::<b256>(__gtf::<u64>(index, GTF_INPUT_COIN_TX_ID)))
        },
        // 1 is the `Contract` Input type
        1u8 => {
            Option::Some(read::<b256>(__gtf::<u64>(index, GTF_INPUT_CONTRACT_TX_ID)))
        },
        _ => {
           Option::None
        },
    }
}
