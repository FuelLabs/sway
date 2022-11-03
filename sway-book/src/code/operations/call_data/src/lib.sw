library call_data;

// ANCHOR: import
use std::auth::msg_sender;
// ANCHOR_END: import

// ANCHOR: access_control
const OWNER = Identity::Address(Address::from(ADMIN));

fn update() {
    require(msg_sender().unwrap() == OWNER, "Owner Only");
    // code
}
// ANCHOR_END: access_control
