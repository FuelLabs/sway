script;

use std::result::Result;
use std::b512::B512;
use std::revert::revert;
use std::logging::log;
use std::ecr::{EcRecoverError, ec_recover, ec_recover_address};
use std::address::Address;

const MSG_HASH = 0xee45573606c96c98ba970ff7cf9511f1b8b25e6bcd52ced30b89df1e4a9c4323;

fn main() {
    let hi = 0xbd0c9b8792876713afa8bff383eebf31c43437823ed761cc3600d0016de5110c;
    let lo = 0x44ac566bd156b4fc71a4a4cb2655d3dd360c695edb17dc3b64d611e122fea23d;
    let signature: B512 = ~B512::from(hi, lo);

    // A recovered public key pair.
    let public_key = ec_recover(signature, MSG_HASH);

    // A recovered Fuel address.
    let result_address: Result<Address, EcRecoverError> = ec_recover_address(signature, MSG_HASH);
    if let Result::Ok(address) = result_address {
        log(address.value);
    } else {
        revert(0);
    }
}
