library token;
//! Functionality for performing common operations on tokens.

use ::address::Address;
use ::contract_id::ContractId;
use ::panic::panic;
use ::tx::*;
use ::context::call_frames::contract_id;

/// Mint `amount` coins of the current contract's `asset_id` and send them (!!! UNCONDITIONALLY !!!) to the contract at `destination`.
/// This will allow the transfer of coins even if there is no way to retrieve them !!!
/// Use of this function can lead to irretrievable loss of coins if not used with caution.
pub fn mint_to_contract(amount: u64, destination: ContractId) {
    mint(amount);
    force_transfer(amount, contract_id(), destination);
}

/// Mint `amount` coins of the current contract's `asset_id` and send them to the Address `recipient`.
pub fn mint_to_address(amount: u64, recipient: Address) {
    mint(amount);
    transfer_to_output(amount, contract_id(), recipient);
}

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

/// !!! UNCONDITIONAL transfer of `amount` coins of type `asset_id` to contract at `destination`.
/// This will allow the transfer of coins even if there is no way to retrieve them !!!
/// Use of this function can lead to irretrievable loss of coins if not used with caution.
pub fn force_transfer(amount: u64, asset_id: ContractId, destination: ContractId) {
    asm(r1: amount, r2: asset_id.value, r3: destination.value) {
        tr r3 r1 r2;
    }
}

/// Transfer `amount` coins of tof type `asset_id` and send them to the address `recipient`.
pub fn transfer_to_output(amount: u64, asset_id: ContractId, recipient: Address) {
    const OUTPUT_VARIABLE_TYPE: u8 = 4;

    // maintain a manual index as we only have `while` loops in sway atm:
    let mut index = 0;
    let mut output_index = 0;
    let mut output_found = false;

    // If an output of type `OutputVariable` is found, check if its `amount` is
    // zero. As one cannot transfer zero coins to an output without a panic, a
    // variable output with a value of zero is by definition unused.
    let outputs_count = tx_outputs_count();
    while index < outputs_count {
        let output_pointer = tx_output_pointer(index);
        if tx_output_type(output_pointer) == OUTPUT_VARIABLE_TYPE && tx_output_amount(output_pointer) == 0 {
            output_index = index;
            output_found = true;
            index = outputs_count; // break early and use the output we found
            // use `break;` when it's implemented #587
        };
        index = index + 1;
    }

    if !output_found {
        panic(0);
    } else {
        asm(r1: recipient.value, r2: output_index, r3: amount, r4: asset_id.value) {
            tro r1 r2 r3 r4;
        };
    }
}
