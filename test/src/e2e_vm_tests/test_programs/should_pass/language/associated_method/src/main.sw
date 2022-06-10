script;

use std::chain::auth::*;
use std::identity::*;
use std::result::*;

fn dummy() -> Identity {
    let res = msg_sender();
    res.unwrap()
}

fn main() -> u64 {
    let identity = dummy();
    return 42;
}
