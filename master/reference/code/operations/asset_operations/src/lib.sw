library;

// ANCHOR: mint_import
use std::asset::mint;
// ANCHOR_END: mint_import
// ANCHOR: mint_to_import
use std::asset::mint_to;
// ANCHOR_END: mint_to_import
// ANCHOR: mint_to_address_import
use std::asset::mint_to_address;
// ANCHOR_END: mint_to_address_import
// ANCHOR: mint_to_contract_import
use std::asset::mint_to_contract;
// ANCHOR_END: mint_to_contract_import
// ANCHOR: burn_import
use std::asset::burn;
// ANCHOR_END: burn_import
// ANCHOR: transfer_import
use std::asset::transfer;
// ANCHOR_END: transfer_import
// ANCHOR: transfer_to_address_import
use std::asset::transfer_to_address;
// ANCHOR_END: transfer_to_address_import
// ANCHOR: force_transfer_to_contract_import
use std::asset::force_transfer_to_contract;
// ANCHOR_END: force_transfer_to_contract_import

use std::constants::ZERO_B256;

fn minting() {
    // ANCHOR: mint
    let amount = 10;
    mint(ZERO_B256, amount);
    // ANCHOR_END: mint
}

fn minting_to_address() {
    // ANCHOR: mint_to_address
    let amount = 10;
    let address = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let user = Address::from(address);

    mint_to_address(user, ZERO_B256, amount);
    // ANCHOR_END: mint_to_address
}

fn minting_to_contract() {
    // ANCHOR: mint_to_contract
    let amount = 10;
    let address = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let pool = ContractId::from(address);

    mint_to_contract(pool, ZERO_B256, amount);
    // ANCHOR_END: mint_to_contract
}

fn minting_to() {
    // ANCHOR: mint_to
    let amount = 10;
    let address = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let user = Identity::Address(Address::from(address));
    let pool = Identity::ContractId(ContractId::from(address));

    mint_to(user, ZERO_B256, amount);
    mint_to(pool, ZERO_B256, amount);
    // ANCHOR_END: mint_to
}

fn burning() {
    // ANCHOR: burn
    let amount = 10;
    burn(ZERO_B256, amount);
    // ANCHOR_END: burn
}

fn transferring_to_address() {
    // ANCHOR: transfer_to_address
    let amount = 10;
    let address = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let asset = AssetId::base();
    let user = Address::from(address);

    transfer_to_address(user, asset, amount);
    // ANCHOR_END: transfer_to_address
}

fn transferring_to_contract() {
    // ANCHOR: force_transfer_to_contract
    let amount = 10;
    let address = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let asset = AssetId::base();
    let pool = ContractId::from(address);

    force_transfer_to_contract(pool, asset, amount);
    // ANCHOR_END: force_transfer_to_contract
}

fn transferring_to() {
    // ANCHOR: transfer
    let amount = 10;
    let address = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let asset = AssetId::base();
    let user = Identity::Address(Address::from(address));
    let pool = Identity::ContractId(ContractId::from(address));

    transfer(user, asset, amount);
    transfer(pool, asset, amount);
    // ANCHOR_END: transfer
}
