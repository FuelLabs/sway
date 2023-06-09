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
/// > **_WARNING:_**
/// >
/// > If the `to` Identity is a contract, this will transfer coins to the contract even with no way to retrieve them
/// > (i.e: no withdrawal functionality on the receiving contract), possibly leading to
/// > the **_PERMANENT LOSS OF COINS_** if not used with care.
///
/// ### Arguments
///
/// * `amount` - The amount of tokens to mint.
/// * `to` - The `Identity` to which to send the tokens.
/// * `sub_id` - The  sub identfier of the asset which to mint.
///
/// ### Examples
///
/// ```sway
/// use std::{constants::ZERO_B256, token::mint_to};
///
/// // replace the zero Address/ContractId with your desired Address/ContractId
/// let to_address = Identity::Address(Address::from(ZERO_B256));
/// let to_contract_id = Identity::ContractId(ContractId::from(ZERO_B256));
/// mint_to(500, to_address, ZERO_B256);
/// mint_to(500, to_contract_id, ZERO_B256);
/// ```
pub fn mint_to(amount: u64, to: Identity, sub_id: b256) {
    mint(amount, sub_id);

    let asset_id = sha256(contract_id(), sub_id);
    transfer(amount, asset_id, to);
}

/// Mint `amount` coins of the current contract's `asset_id` and send them
/// UNCONDITIONALLY to the contract at `to`.
///
/// > **_WARNING:_**
/// >
/// > This will transfer coins to a contract even with no way to retrieve them
/// > (i.e: no withdrawal functionality on the receiving contract), possibly leading to
/// > the **_PERMANENT LOSS OF COINS_** if not used with care.
///
/// ### Arguments
///
/// * `amount` - The amount of tokens to mint.
/// * `to` - The `ContractId` to which to send the tokens.
/// * `sub_id` - The  sub identfier of the asset which to mint.
///
/// ### Examples
///
/// ```sway
/// use std::{constants::ZERO_B256, token::mint_to_contract};
///
/// // replace the zero ContractId with your desired ContractId
/// let to = ContractId::from(ZERO_B256);
/// mint_to_contract(500, to, ZERO_B256);
/// ```
pub fn mint_to_contract(amount: u64, to: ContractId, sub_id: b256) {
    mint(amount, sub_id);

    let asset_id = sha256((contract_id(), sub_id));
    force_transfer_to_contract(amount, contract_id(), to);
}

/// Mint `amount` coins of the current contract's `asset_id` and send them to
/// the Address `to`.
///
/// ### Arguments
///
/// * `amount` - The amount of tokens to mint.
/// * `to` - The `Address` to which to send the tokens.
/// * `sub_id` - The  sub identfier of the asset which to mint.
///
/// ### Examples
///
/// ```sway
/// use std::{constants::ZERO_B256, token::mint_to_address};
///
/// // replace the zero Address with your desired Address
/// let to = Address::from(ZERO_B256);
/// mint_to_address(500, to, ZERO_B256);
/// ```
pub fn mint_to_address(amount: u64, to: Address, sub_id: b256) {
    mint(amount, sub_id);

    let asset_id = sha256((contract_id(), sub_id));
    transfer_to_address(amount, asset_id, to);
}

/// Mint `amount` coins of the current contract's `sub_id`. The newly minted tokens are owned by the current contract.
///
/// ### Arguments
///
/// * `amount` - The amount of tokens to mint.
/// * `sub_id` - The  sub identfier of the asset which to mint.
///
/// ### Examples
///
/// ```sway
/// use std::{constants::ZERO_B256, token::mint};
///
/// mint(500, ZERO_B256);
/// ```
pub fn mint(amount: u64, sub_id: b256) {
    let asset_id = sha256((contract_id(), sub_id));
    asm(r1: amount, r2: asset_id) {
        mint r1 r2;
    }
}

/// Burn `amount` coins of the current contract's `sub_id`. Burns them from the balance of the current contract.
///
/// ### Arguments
///
/// * `amount` - The amount of tokens to burn.
/// * `sub_id` - The sub identfier of the asset which to burn.
///
/// ### Reverts
///
/// Reverts if the contract balance is less than `amount`.
///
/// ### Examples
///
/// ```sway
/// use std::{constants::ZERO_B256, token::burn};
///
/// burn(500, ZERO_B256);
/// ```
pub fn burn(amount: u64, sub_id: b256) {
    let asset_id = sha256((contract_id(), sub_id));
    asm(r1: amount, r2: asset_id) {
        burn r1 r2;
    }
}

/// Transfer `amount` coins of the type `asset_id` and send them
/// to `to` by calling either `force_transfer_to_contract` or
/// `transfer_to_address`, depending on the type of `Identity`.
///
/// > **_WARNING:_**
/// >
/// > If the `to` Identity is a contract this may transfer coins to the contract even with no way to retrieve them
/// > (i.e. no withdrawal functionality on receiving contract), possibly leading
/// > to the **_PERMANENT LOSS OF COINS_** if not used with care.
///
/// ### Arguments
///
/// * `amount` - The amount of tokens to transfer.
/// * `asset_id` - The `AssetId` of the token to transfer.
/// * `to` - The `Identity` of the recipient.
///
/// ### Reverts
///
/// * If `amount` is greater than the contract balance for `asset_id`.
/// * If `amount` is equal to zero.
/// * If there are no free variable outputs when transferring to an `Address`.
///
/// ### Examples
///
/// ```sway
/// use std::{constants::{BASE_ASSET_ID, ZERO_B256}, token::transfer};
///
/// // replace the zero Address/ContractId with your desired Address/ContractId
/// let to_address = Identity::Address(Address::from(ZERO_B256));
/// let to_contract_id = Identity::ContractId(ContractId::from(ZERO_B256));
/// transfer(500, BASE_ASSET_ID, to_address);
/// transfer(500, BASE_ASSET_ID, to_contract_id);
/// ```
pub fn transfer(amount: u64, asset_id: b256, to: Identity) {
    match to {
        Identity::Address(addr) => transfer_to_address(amount, asset_id, addr),
        Identity::ContractId(id) => force_transfer_to_contract(amount, asset_id, id),
    };
}

/// UNCONDITIONAL transfer of `amount` coins of type `asset_id` to
/// the contract at `to`.
///
/// > **_WARNING:_**
/// >
/// > This will transfer coins to a contract even with no way to retrieve them
/// > (i.e. no withdrawal functionality on receiving contract), possibly leading
/// > to the **_PERMANENT LOSS OF COINS_** if not used with care.
///
/// ### Arguments
///
/// * `amount` - The amount of tokens to transfer.
/// * `asset_id` - The `AssetId` of the token to transfer.
/// * `to` - The `ContractId` of the recipient contract.
///
/// ### Reverts
///
/// * If `amount` is greater than the contract balance for `asset_id`.
/// * If `amount` is equal to zero.
///
/// ### Examples
///
/// ```sway
/// use std::{constants::{BASE_ASSET_ID, ZERO_B256}, token::force_transfer_to_contract};
///
/// // replace the zero ContractId with your desired ContractId
/// let to_contract_id = Identity::ContractId(ContractId::from(ZERO_B256));
/// force_transfer_to_contract(500, BASE_ASSET_ID, to_contract_id);
/// ```
pub fn force_transfer_to_contract(amount: u64, asset_id: b256, to: ContractId) {
    asm(r1: amount, r2: asset_id, r3: to.value) {
        tr r3 r1 r2;
    }
}

/// Transfer `amount` coins of type `asset_id` and send them to
/// the address `to`.
///
/// ### Arguments
///
/// * `amount` - The amount of tokens to transfer.
/// * `asset_id` - The `AssetId` of the token to transfer.
/// * `to` - The `Address` of the recipient user.
///
/// ### Reverts
///
/// * If `amount` is greater than the contract balance for `asset_id`.
/// * If `amount` is equal to zero.
/// * If there are no free variable outputs.
///
/// ### Examples
///
/// ```sway
/// use std::{constants::{BASE_ASSET_ID, ZERO_B256}, token::transfer_to_address};
///
/// // replace the zero Address with your desired Address
/// let to_address = Identity::Address(Address::from(ZERO_B256));
/// transfer_to_address(500, BASE_ASSET_ID, to_address);
/// ```
pub fn transfer_to_address(amount: u64, asset_id: b256, to: Address) {
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
                    tro r1 r2 r3 r4;
                };
                return;
            }
        }
        index += 1;
    }

    revert(FAILED_TRANSFER_TO_ADDRESS_SIGNAL);
}
