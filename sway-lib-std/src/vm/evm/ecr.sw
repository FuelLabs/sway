library ecr;

use ::address::Address;
use ::b512::B512;
use ::context::registers::error;
use ::ecr::{EcRecoverError, ec_recover};
use ::hash::{HashMethod, hash_pair};
use ::result::*;

/// Recover the address derived from the private key used to sign a message.
/// Returns a `Result` to let the caller choose an error handling strategy.
/// Ethereum addresses are 20 bytes long, so these are left-padded to fit in a 32 byte Address type.
pub fn ec_recover_address(signature: B512, msg_hash: b256) -> Result<Address, EcRecoverError> {
    let pub_key_result = ec_recover(signature, msg_hash);

    if let Result::Err(e) = pub_key_result {
        // propagate the error if it exists
        Result::Err(e)
    } else {
        let pub_key = pub_key_result.unwrap();

        // Note that Ethereum addresses are derived from the Keccak256 hash of the pubkey (not sha256)
        let address = hash_pair((pub_key.bytes)[0], (pub_key.bytes)[1], HashMethod::Keccak256);

        // Zero out first 12 bytes for ethereum address
        asm(r1: address) {
            mcli r1 i12;
        };

        Result::Ok(~Address::from(address))
    }
}
