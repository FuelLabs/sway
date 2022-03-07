library token;
//! Functionality for performing common operations on tokens.

use ::contract_id::ContractId;
use ::address::Address;
use ::chain::panic;

/// Mint `amount` coins of the current contract's `asset_id`.
pub fn mint(amount: u64) {
    asm(r1: amount) {
        mint r1;
    }
}

/// Burn `amount` coins of the current contract's `asset_id`.
pub fn burn(amount: u64) {
    asm(r1: amount) {
        burn r1;
    }
}

/// !!! UNCONDITIONAL transfer of `amount` coins of type `asset_id` to contract at `contract_id`.
/// This will allow the transfer of coins even if there is no way to retrieve them !!!
/// Use of this function can lead to irretrievable loss of coins if not used with caution.
pub fn force_transfer(amount: u64, asset_id: ContractId, contract_id: ContractId) {
    asm(r1: amount, r2: asset_id.value, r3: contract_id.value) {
        tr r3 r1 r2;
    }
}

/// Transfer `amount` coins of type `asset_id` to address `recipient`.
pub fn transfer_to_output(amount: u64, asset_id: ContractId, recipient: Address) {
    // note: if tx format changes, the magic number "56" must be changed !
    // TransactionScript outputsCount has a 56 byte(7 words * 8 bytes) offset
    // Transaction Script: https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/tx_format.md#transactionscript
    // Output types: https://github.com/FuelLabs/fuel-specs/blob/master/specs/protocol/tx_format.md#output
    const OUTPUT_LENGTH_LOCATION = 56;
    const OUTPUT_VARIABLE_TYPE = 4u8;

    // get length of outputs from TransactionScript outputsCount:
    let length: u8 = asm(outputs_length, outputs_length_ptr: OUTPUT_LENGTH_LOCATION) {
        lb outputs_length outputs_length_ptr i0;
        outputs_length: u8
    };
    // maintain a manual index as we only have `while` loops in sway atm:
    let mut index: u8 = 0u8;
    let mut output_index = 0;
    let mut output_found = false;

    // If an output of type `OutputVariable` is found, check if its `amount` is zero.
    // As one cannot transfer zero coins to an output without a panic, a variable output with a value of zero is by definition unused.
    while index < length {
        let output_start = asm(n: index, offset) {
            xos offset n; // get the offset to the nth output
            offset: u64
        };

        let type = asm(offset: output_start, t) {
            lb t offset i0; // load the type of the output at 'offset' into t
            t: u8
        };

        // if an ouput is found of type `OutputVariable`:
        if type == OUTPUT_VARIABLE_TYPE {
            let amount = asm(n: index, a, amount_ptr, output: output_start) {
                addi amount_ptr output i40;
                lw a amount_ptr i0;
                a: u64
            };

            // && if the amount is zero:
            if amount == 0 {
                // then store the index of the output and record the fact that we found a suitable output.
                output_index = index;
                output_found = true;
                // todo: use "break" keyword when it lands ( tracked here: https://github.com/FuelLabs/sway/issues/587 )
                index = length; // break early and use the output we found
            } else {
                // otherwise, increment the index and continue the loop.
                index = index + 1;
            };
        } else {
            index = length; // break early as there are no suitable outputs.
        };
    }

    if !output_found {
        // If no suitable output was found, revert.
        panic(0);
    } else {
        // perform the transfer
        asm(amnt: amount, id: asset_id.value, recipient: recipient.value, output: index) {
            tro recipient output amnt id;
        };
    }
}
