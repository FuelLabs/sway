library token;
//! Functionality for performing common operations on tokens.

use ::address::Address;
use ::chain::panic;

// @todo if tx format changes, the magic number "384" must be changed !
// TransactionScript outputsCount has a 6 word/384-bit offset
const OUTPUT_LENGTH_LOCATION = 384;
// output types: https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/compressed_tx_format.md#output
const OUTPUT_VARIABLE_TYPE = 4;

/// Mint `n` coins of the current contract's token_id.
pub fn mint(n: u64) {
    asm(r1: n) {
        mint r1;
    }
}

/// Burn `n` coins of the current contract's token_id.
pub fn burn(n: u64) {
    asm(r1: n) {
        burn r1;
    }
}

/// Transfer amount `coins` of type `token_id` to address `recipient`.
pub fn transfer_to_output(coins: u64, token_id: b256, recipient: Address) {
    // get length of outputs from TransactionScript outputsCount:
    let length: u8 = asm(outputs_length, outputs_length_ptr: OUTPUT_LENGTH_LOCATION) {
        lw outputs_length outputs_length_ptr;
        outputs_length: u8
    };
    // maintain a manual index as we only have `while` loops in sway atm:
    let mut index: u8 = 0;
    let mut outputIndex = 0;
    let mut output_found = false;

    while index < length {
        // If an output of type "outputVariable" is found, check if its`amount` is zero.
        // You can't transfer zero coins to an output without a panic, so a variable output with a zero value is by definition unused.
        // if an ouput is found of type "OutputVariable" && the amount is zero:
        if asm(slot: index, type, target: OUTPUT_VARIABLE_TYPE, bytes: 8, res) {
            xos t slot;
            meq res type target bytes;
            res: bool
        } && asm(slot: index, a, amount_ptr, output, is_zero, bytes: 8) {
                xos output slot;
                addi amount_ptr output i64;
                lw a amount_ptr;
                meq is_zero a zero bytes;
                is_zero: bool
            } {
                outputIndex = index;
                output_found = true;
                return;
            } else {
                index = index + 1;
            }
    }
    // If no suitable output was found, revert.
    if !output_found { panic(0) };

    asm(amount: coins, id: token_id, recipient, output: index) {
        tro recipient output amount id;
    }
}

/// !!! UNCONDITIONAL transfer of amount `coins` of type `token_id` to contract at `contract_id`.
/// This will allow the transfer of coins even if there is no way to retrieve them !!!
/// Use of this function can lead to irretrievable loss of coins if not used with caution.
// @todo use type `ContractId` if implemented.
pub fn force_transfer(coins: u64, token_id: b256, contract_id: b256) {
    asm(coins, token_id, contract_id) {
        tr contract_id coins token_id;
    }
}
