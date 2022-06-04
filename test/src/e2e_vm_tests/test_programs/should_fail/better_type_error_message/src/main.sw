script;

use std::chain::auth::*;
use std::identity::*;
//use std::result::*;

fn bogus() -> Identity {
    let sender: Result<Identity, AuthError> = msg_sender();
    sender.unwrap()
}

fn main() {
}
