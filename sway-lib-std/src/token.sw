//! Functionality for performing common operations with tokens.
library token;

use ::address::Address;
use ::context::call_frames::contract_id;
use ::contract_id::ContractId;
use ::identity::Identity;
use ::revert::revert;
use ::outputs::{Output, output_amount, output_count, output_type};

/// Mint `amount` coins of the current contract's `asset_id` and transfer them
/// to `to` by calling either force_transfer_to_contract() or
/// transfer_to_output(), depending on the type of `Identity`.
pub fn mint_to(amount: u64, to: Identity) {
    mint(amount);
    transfer(amount, contract_id(), to);
}

/// Mint `amount` coins of the current contract's `asset_id` and send them
/// UNCONDITIONALLY to the contract at `to`.
/// 
/// CAUTION !!!
/// 
/// This will transfer coins to a contract even with no way to retrieve them
/// (i.e: no withdrawal functionality on the receiving contract), possibly leading to
/// the PERMANENT LOSS OF COINS if not used with care.
pub fn mint_to_contract(amount: u64, to: ContractId) {
    mint(amount);
    force_transfer_to_contract(amount, contract_id(), to);
}

/// Mint `amount` coins of the current contract's `asset_id` and send them to
/// the Address `to`.
pub fn mint_to_address(amount: u64, to: Address) {
    mint(amount);
    transfer_to_output(amount, contract_id(), to);
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

/// Transfer `amount` coins of the current contract's `asset_id` and send them
/// to `to` by calling either force_transfer_to_contract() or
/// transfer_to_output(), depending on the type of `Identity`.
/// 
/// CAUTION !!!
/// 
/// This may transfer coins to a contract even with no way to retrieve them
/// (i.e. no withdrawal functionality on receiving contract), possibly leading
/// to the PERMANENT LOSS OF COINS if not used with care.
pub fn transfer(amount: u64, asset_id: ContractId, to: Identity) {
    match to {
        Identity::Address(addr) => transfer_to_output(amount, asset_id, addr),
        Identity::ContractId(id) => force_transfer_to_contract(amount, asset_id, id),
    };
}

/// UNCONDITIONAL transfer of `amount` coins of type `asset_id` to
/// the contract at `to`.
/// 
/// CAUTION !!!
/// 
/// This will transfer coins to a contract even with no way to retrieve them
/// (i.e. no withdrawal functionality on receiving contract), possibly leading
/// to the PERMANENT LOSS OF COINS if not used with care.
pub fn force_transfer_to_contract(amount: u64, asset_id: ContractId, to: ContractId) {
    asm(r1: amount, r2: asset_id.value, r3: to.value) {
        tr r3 r1 r2;
    }
}

/// Transfer `amount` coins of type `asset_id` and send them to
/// the address `to`.
pub fn transfer_to_output(amount: u64, asset_id: ContractId, to: Address) {
    // maintain a manual index as we only have `while` loops in sway atm:
    let mut index = 0;
    let mut output_index = 0;
    let mut output_found = false;

    // If an output of type `OutputVariable` is found, check if its `amount` is
    // zero. As one cannot transfer zero coins to an output without a panic, a
    // variable output with a value of zero is by definition unused.
    let outputs = output_count();
    while index < outputs {
        let type_of_output = output_type(index);
        if let Output::Variable = type_of_output {
            if output_amount(index) == 0 {
                output_index = index;
                output_found = true;
                break; // break early and use the output we found
            }
        }
        index += 1;
    }

    if !output_found {
        revert(0);
    } else {
        asm(r1: to.value, r2: output_index, r3: amount, r4: asset_id.value) {
            tro r1 r2 r3 r4;
        };
    }
}
