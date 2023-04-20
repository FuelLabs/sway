library;

// ANCHOR: import_sender
use std::auth::msg_sender;
// ANCHOR_END: import_sender

// ANCHOR: import_asset
use std::{call_frames::msg_asset_id, constants::BASE_ASSET_ID};
// ANCHOR_END: import_asset

// ANCHOR: import_amount
use std::context::msg_amount;
// ANCHOR_END: import_amount

// ANCHOR: access_control
const OWNER = Identity::Address(Address::from(ADMIN));

fn update() {
    require(msg_sender().unwrap() == OWNER, "Owner Only");
    // code
}
// ANCHOR_END: access_control

// ANCHOR: deposit
fn deposit() {
    if msg_asset_id() == BASE_ASSET_ID {
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
