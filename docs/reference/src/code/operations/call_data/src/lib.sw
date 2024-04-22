library;

// ANCHOR: import_asset
use std::call_frames::msg_asset_id;
// ANCHOR_END: import_asset

// ANCHOR: import_amount
use std::context::msg_amount;
// ANCHOR_END: import_amount

// ANCHOR: access_control
const OWNER = Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000000));

fn update() {
    require(msg_sender().unwrap() == OWNER, "Owner Only");
    // code
}
// ANCHOR_END: access_control

// ANCHOR: deposit
fn deposit() {
    if msg_asset_id() == AssetId::base() {
        // code
    } else {
        // code
    }
}
// ANCHOR_END: deposit

// ANCHOR: deposit_amount
fn purchase() {
    require(msg_amount() == 100_000_000, "Incorrect amount sent");
    // code
}
// ANCHOR_END: deposit_amount
