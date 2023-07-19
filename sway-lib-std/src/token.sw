//! Functionality for performing common operations with tokens.
library;

use ::address::Address;
use ::call_frames::contract_id;
use ::contract_id::{ContractId, AssetId};
use ::error_signals::FAILED_TRANSFER_TO_ADDRESS_SIGNAL;
use ::identity::Identity;
use ::revert::revert;
use ::outputs::{Output, output_amount, output_count, output_type};

/// Mint `amount` coins of the current contract's `asset_id` and transfer them
/// to `to` by calling either `force_transfer_to_contract` or
/// `transfer_to_address`, depending on the type of `Identity`.
///
/// # Additional Information
/// 
/// If the `to` Identity is a contract, this will transfer coins to the contract even with no way to retrieve them
/// (i.e: no withdrawal functionality on the receiving contract), possibly leading to
/// the **_PERMANENT LOSS OF COINS_** if not used with care.
///
/// # Arguments
///
/// * `amount`: [u64] - The amount of tokens to mint.
/// * `to`: [Identity] - The recipient identity.
///
/// # Examples
///
/// ```sway
/// use std::{constants::ZERO_B256, token::mint_to};
///
/// fn foo() {
///     // replace the zero Address/ContractId with your desired Address/ContractId
///     let to_address = Identity::Address(Address::from(ZERO_B256));
///     let to_contract_id = Identity::ContractId(ContractId::from(ZERO_B256));
///     mint_to(500, to_address);
///     mint_to(500, to_contract_id);
/// }
/// ```
pub fn mint_to(amount: u64, to: Identity) {
    mint(amount);
    transfer(amount, contract_id(), to);
}

/// Mint `amount` coins of the current contract's `asset_id` and send them
/// UNCONDITIONALLY to the contract at `to`.
///
/// # Additional Information
/// 
/// This will transfer coins to a contract even with no way to retrieve them
/// (i.e: no withdrawal functionality on the receiving contract), possibly leading to
/// the **_PERMANENT LOSS OF COINS_** if not used with care.
///
/// # Arguments
///
/// * `amount`: [u64] - The amount of tokens to mint.
/// * `to`: [ContractId] - The recipient contract.
///
/// # Examples
///
/// ```sway
/// use std::{constants::ZERO_B256, token::mint_to_contract};
///
/// fn foo() {
///     // replace the zero ContractId with your desired ContractId
///     let to = ContractId::from(ZERO_B256);
///     mint_to_contract(500, to);
/// }
/// ```
pub fn mint_to_contract(amount: u64, to: ContractId) {
    mint(amount);
    force_transfer_to_contract(amount, contract_id(), to);
}

/// Mint `amount` coins of the current contract's `asset_id` and send them to
/// the Address `to`.
///
/// # Arguments
///
/// * `amount`: [u64] - The amount of tokens to mint.
/// * `to`: [Address] - The recipient address.
///
/// # Examples
///
/// ```sway
/// use std::{constants::ZERO_B256, token::mint_to_address};
///
/// fn foo() {
///     // replace the zero Address with your desired Address
///     let to = Address::from(ZERO_B256);
///     mint_to_address(500, to);
/// }
/// ```
pub fn mint_to_address(amount: u64, to: Address) {
    mint(amount);
    transfer_to_address(amount, contract_id(), to);
}

/// Mint `amount` coins of the current contract's `asset_id`. The newly minted tokens are owned by the current contract.
///
/// # Arguments
///
/// * `amount`: [u64] - The amount of tokens to mint.
///
/// # Examples
///
/// ```sway
/// use std::token::mint;
///
/// fn foo() {
///     mint(500);
/// }
/// ```
pub fn mint(amount: u64) {
    asm(r1: amount) {
        mint r1;
    }
}

/// Burn `amount` coins of the current contract's `asset_id`. Burns them from the balance of the current contract.
///
/// # Arguments
///
/// * `amount`: [u64] - The amount of tokens to burn.
///
/// # Panics
///
/// * When the contract balance is less than `amount`.
///
/// # Examples
///
/// ```sway
/// use std::token::burn;
///
/// fn foo() {
///     burn(500);
/// }
/// ```
pub fn burn(amount: u64) {
    asm(r1: amount) {
        burn r1;
    }
}

/// Transfer `amount` coins of the type `asset_id` and send them
/// to `to` by calling either `force_transfer_to_contract` or
/// `transfer_to_address`, depending on the type of `Identity`.
///
/// # Additional Information
/// 
/// If the `to` Identity is a contract this may transfer coins to the contract even with no way to retrieve them
/// (i.e. no withdrawal functionality on receiving contract), possibly leading
/// to the **_PERMANENT LOSS OF COINS_** if not used with care.
///
/// # Arguments
///
/// * `amount`: [u64] - The amount of tokens to transfer.
/// * `asset_id`: [AssetId] - The token to transfer.
/// * `to`: [Identity] - The recipient identity.
///
/// # Panics
///
/// * When `amount` is greater than the contract balance for `asset_id`.
/// * When `amount` is equal to zero.
/// * When there are no free variable outputs when transferring to an `Address`.
///
/// # Examples
///
/// ```sway
/// use std::{constants::{BASE_ASSET_ID, ZERO_B256}, token::transfer};
///
/// fn foo() {
///     // replace the zero Address/ContractId with your desired Address/ContractId
///     let to_address = Identity::Address(Address::from(ZERO_B256));
///     let to_contract_id = Identity::ContractId(ContractId::from(ZERO_B256));
///     transfer(500, BASE_ASSET_ID, to_address);
///     transfer(500, BASE_ASSET_ID, to_contract_id);
/// }
/// ```
pub fn transfer(amount: u64, asset_id: AssetId, to: Identity) {
    match to {
        Identity::Address(addr) => transfer_to_address(amount, asset_id, addr),
        Identity::ContractId(id) => force_transfer_to_contract(amount, asset_id, id),
    };
}

/// UNCONDITIONAL transfer of `amount` coins of type `asset_id` to
/// the contract at `to`.
///
/// # Additional Information
/// 
/// This will transfer coins to a contract even with no way to retrieve them
/// (i.e. no withdrawal functionality on receiving contract), possibly leading
/// to the **_PERMANENT LOSS OF COINS_** if not used with care.
///
/// # Arguments
///
/// * `amount`: [u64] - The amount of tokens to transfer.
/// * `asset_id`: [AssetId] - The token to transfer.
/// * `to`: [ContractId] - The recipient contract.
///
/// # Reverts
///
/// * When `amount` is greater than the contract balance for `asset_id`.
/// * When `amount` is equal to zero.
///
/// # Examples
///
/// ```sway
/// use std::{constants::{BASE_ASSET_ID, ZERO_B256}, token::force_transfer_to_contract};
///
/// fn foo() {
///     // replace the zero ContractId with your desired ContractId
///     let to_contract_id = ContractId::from(ZERO_B256);
///     force_transfer_to_contract(500, BASE_ASSET_ID, to_contract_id);
/// }
/// ```
pub fn force_transfer_to_contract(amount: u64, asset_id: AssetId, to: ContractId) {
    asm(r1: amount, r2: asset_id.value, r3: to.value) {
        tr r3 r1 r2;
    }
}

/// Transfer `amount` coins of type `asset_id` and send them to
/// the address `to`.
///
/// # Arguments
///
/// * `amount`: [u64] - The amount of tokens to transfer.
/// * `asset_id`: [AssetId] - The token to transfer.
/// * `to`: [Address] - The recipient address.
///
/// # Panics
///
/// * When `amount` is greater than the contract balance for `asset_id`.
/// * When `amount` is equal to zero.
/// * When there are no free variable outputs.
///
/// # Examples
///
/// ```sway
/// use std::{constants::{BASE_ASSET_ID, ZERO_B256}, token::transfer_to_address};
///
/// fn foo() {
///     // replace the zero Address with your desired Address
///     let to_address = Address::from(ZERO_B256);
///     transfer_to_address(500, BASE_ASSET_ID, to_address);
/// }
/// ```
pub fn transfer_to_address(amount: u64, asset_id: AssetId, to: Address) {
    // maintain a manual index as we only have `while` loops in sway atm:
    let mut index = 0;

    // If an output of type `OutputVariable` is found, check if its `amount` is
    // zero. As one cannot transfer zero coins to an output without a panic, a
    // variable output with a value of zero is by definition unused.
    let number_of_outputs = output_count();
    while index < number_of_outputs {
        if let Output::Variable = output_type(index) {
            if output_amount(index) == 0 {
                asm(r1: to.value, r2: index, r3: amount, r4: asset_id.value) {
                    tro r1 r2 r3 r4;
                };
                return;
            }
        }
        index += 1;
    }

    revert(FAILED_TRANSFER_TO_ADDRESS_SIGNAL);
}
