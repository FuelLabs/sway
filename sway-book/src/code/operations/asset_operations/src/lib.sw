library asset_operations;

// ANCHOR: mint_import
use std::token::{mint, mint_to, mint_to_address, mint_to_contract};
// ANCHOR_END: mint_import

fn minting() {
    // ANCHOR: mint
    let amount = 10;
    mint(amount);
    // ANCHOR_END: mint
}

fn minting_to_address() {
    // ANCHOR: mint_to_address
    let amount = 10;
    let address = 0x0000000000000000000000000000000000000000000000000000000000000001;
    mint_to_address(amount, Address::from(address));
    // ANCHOR_END: mint_to_address
}

fn minting_to_contract() {
    // ANCHOR: mint_to_contract
    let amount = 10;
    let address = 0x0000000000000000000000000000000000000000000000000000000000000001;
    mint_to_contract(amount, ContractId::from(address));
    // ANCHOR_END: mint_to_contract
}


fn minting_to() {
    // ANCHOR: mint_to
    let amount = 10;
    let address = 0x0000000000000000000000000000000000000000000000000000000000000001;
    mint_to(amount, Identity::Address(Address::from(address)));
    mint_to(amount, Identity::ContractId(ContractId::from(address)));
    // ANCHOR_END: mint_to
}
