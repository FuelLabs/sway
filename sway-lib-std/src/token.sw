//! Functionality for performing common operations with tokens.
library;

use ::address::Address;
use ::alias::SubId;
use ::call_frames::contract_id;
use ::contract_id::{AssetId, ContractId};
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
/// * `to`: [Identity] - The recipient identity.
/// * `sub_id`: [SubId] - The sub identfier of the asset which to mint.
/// * `amount`: [u64] - The amount of tokens to mint.
///
/// # Examples
///
/// ```sway
/// use std::{constants::ZERO_B256, token::mint_to};
///
/// fn foo() {
///     let to_address = Identity::Address(Address::from(ZERO_B256));
///     let to_contract_id = Identity::ContractId(ContractId::from(ZERO_B256));
///     mint_to(to_address, ZERO_B256, 500);
///     mint_to(to_contract_id, ZERO_B256, 500);
/// }
/// ```
pub fn mint_to(to: Identity, sub_id: SubId, amount: u64) {
    mint(sub_id, amount);
    transfer(to, AssetId::new(contract_id(), sub_id), amount);
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
/// * `to`: [ContractId] - The recipient contract.
/// * `sub_id`: [SubId] - The sub identfier of the asset which to mint.
/// * `amount`: [u64] - The amount of tokens to mint.
///
/// # Examples
///
/// ```sway
/// use std::{constants::ZERO_B256, token::mint_to_contract};
///
/// fn foo() {
///     let to = ContractId::from(ZERO_B256);
///     mint_to_contract(to, ZERO_B256, 500);
/// }
/// ```
pub fn mint_to_contract(to: ContractId, sub_id: SubId, amount: u64) {
    mint(sub_id, amount);
    force_transfer_to_contract(to, AssetId::new(contract_id(), sub_id), amount);
}

/// Mint `amount` coins of the current contract's `asset_id` and send them to
/// the Address `to`.
///
/// # Arguments
///
/// * `to`: [Address] - The recipient address.
/// * `sub_id`: [SubId] - The sub identfier of the asset which to mint.
/// * `amount`: [u64] - The amount of tokens to mint.
///
/// # Examples
///
/// ```sway
/// use std::{constants::ZERO_B256, token::mint_to_address};
///
/// fn foo() {
///     let to = Address::from(ZERO_B256);
///     mint_to_address(to, ZERO_B256, 500);
/// }
/// ```
pub fn mint_to_address(to: Address, sub_id: SubId, amount: u64) {
    mint(sub_id, amount);
    transfer_to_address(to, AssetId::new(contract_id(), sub_id), amount);
}

/// Mint `amount` coins of the current contract's `sub_id`. The newly minted tokens are owned by the current contract.
///
/// # Arguments
///
/// * `sub_id`: [SubId] - The sub identfier of the asset which to mint.
/// * `amount`: [u64] - The amount of tokens to mint.
///
/// # Examples
///
/// ```sway
/// use std::{constants::ZERO_B256, token::mint};
///
/// fn foo() {
///     mint(ZERO_B256, 500);
/// }
/// ```
pub fn mint(sub_id: SubId, amount: u64) {
    asm(r1: amount, r2: sub_id) {
        mint r1 r2;
    }
}

/// Burn `amount` coins of the current contract's `sub_id`. Burns them from the balance of the current contract.
///
/// # Arguments
///
/// * `sub_id`: [SubId] - The sub identfier of the asset which to burn.
/// * `amount`: [u64] - The amount of tokens to burn.
///
/// # Reverts
///
/// * When the contract balance is less than `amount`.
///
/// # Examples
///
/// ```sway
/// use std::{constants::ZERO_B256, token::burn};
///
/// fn foo() {
///     burn(ZERO_B256, 500);
/// }
/// ```
pub fn burn(sub_id: SubId, amount: u64) {
    asm(r1: amount, r2: sub_id) {
        burn r1 r2;
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
/// * `to`: [Identity] - The recipient identity.
/// * `asset_id`: [AssetId] - The token to transfer.
/// * `amount`: [u64] - The amount of tokens to transfer.
///
/// # Reverts
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
///     let to_address = Identity::Address(Address::from(ZERO_B256));
///     let to_contract_id = Identity::ContractId(ContractId::from(ZERO_B256));
///     transfer(to_address, BASE_ASSET_ID, 500);
///     transfer(to_contract_id, BASE_ASSET_ID, 500);
/// }
/// ```
pub fn transfer(to: Identity, asset_id: AssetId, amount: u64) {
    match to {
        Identity::Address(addr) => transfer_to_address(addr, asset_id, amount),
        Identity::ContractId(id) => force_transfer_to_contract(id, asset_id, amount),
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
/// * `to`: [ContractId] - The recipient contract.
/// * `asset_id`: [AssetId] - The token to transfer.
/// * `amount`: [u64] - The amount of tokens to transfer.
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
///     let to_contract_id = ContractId::from(ZERO_B256);
///     force_transfer_to_contract(to_contract_id, BASE_ASSET_ID, 500);
/// }
/// ```
pub fn force_transfer_to_contract(to: ContractId, asset_id: AssetId, amount: u64) {
    asm(r1: amount, r2: asset_id, r3: to.value) {
        tr   r3 r1 r2;
    }
}

/// Transfer `amount` coins of type `asset_id` and send them to
/// the address `to`.
///
/// # Arguments
///
/// * `to`: [Address] - The recipient address.
/// * `asset_id`: [AssetId] - The token to transfer.
/// * `amount`: [u64] - The amount of tokens to transfer.
///
/// # Reverts
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
///     let to_address = Address::from(ZERO_B256);
///     transfer_to_address(to_address, BASE_ASSET_ID, 500);
/// }
/// ```
pub fn transfer_to_address(to: Address, asset_id: AssetId, amount: u64) {
    // maintain a manual index as we only have `while` loops in sway atm:
    let mut index = 0;

    // If an output of type `OutputVariable` is found, check if its `amount` is
    // zero. As one cannot transfer zero coins to an output without a panic, a
    // variable output with a value of zero is by definition unused.
    let number_of_outputs = output_count();
    while index < number_of_outputs {
        if let Output::Variable = output_type(index) {
            if output_amount(index) == 0 {
                asm(r1: to.value, r2: index, r3: amount, r4: asset_id) {
                    tro  r1 r2 r3 r4;
                };
                return;
            }
        }
        index += 1;
    }

    revert(FAILED_TRANSFER_TO_ADDRESS_SIGNAL);
}
