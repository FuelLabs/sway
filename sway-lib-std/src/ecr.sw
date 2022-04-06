library ecr;

use ::address::Address;
use ::b512::B512;
use ::constants::ZERO;
use ::hash::{HashMethod, hash_pair};
use ::result::Result;

pub enum EcRecoverError {
    UnrecoverableAddress: (),
    UnrecoverablePublicKey: (),
}

/// Recover the public key derived from the private key used to sign a message
pub fn ec_recover(signature: B512, msg_hash: b256) -> Result<B512, EcRecoverError> {
    let public_key = ~B512::new();

    asm(buffer: public_key.bytes, sig: signature.bytes, hash: msg_hash) {
        ecr buffer sig hash;
    };
    /// if recovered key is empty
    if (public_key.bytes)[0] == ZERO && (public_key.bytes)[1] == ZERO {
        Result::Err(EcRecoverError::UnrecoverablePublicKey)
    } else {
        Result::Ok(public_key)
    }
}

/// Recover the address derived from the private key used to sign a message
pub fn ec_recover_address(signature: B512, msg_hash: b256) -> Result<Address, EcRecoverError> {
    let pub_key_result = ec_recover(signature, msg_hash);

    if let Result::Ok(p) = pub_key_result {
        let address = ~Address::from(hash_pair((p.bytes)[0], (p.bytes)[1], HashMethod::Sha256));
        if address != ~Address::from(ZERO) {
            Result::Ok(address)
        } else {
            Result::Err(EcRecoverError::UnrecoverableAddress)
        }
    } else {
        // propagate the error if it exists
        Result::Err(EcRecoverError::UnrecoverablePublicKey)
    }
}
