//! Helper functions to verify EVM signatures.
library;

use ::b512::B512;
use ::registers::error;
use ::ecr::{ec_recover, EcRecoverError};
use ::hash::*;
use ::result::Result::{self, *};
use ::vm::evm::evm_address::EvmAddress;

fn pub_key_address(tuple: (b256, b256)) -> b256 {
    let mut hasher = Hasher::new();
    tuple.0.hash(hasher);
    tuple.1.hash(hasher);
    hasher.keccak256()
}

/// Recover the EVM address derived from the private key used to sign a message.
/// Returns a `Result` to let the caller choose an error handling strategy.
pub fn ec_recover_evm_address(
    signature: B512,
    msg_hash: b256,
) -> Result<EvmAddress, EcRecoverError> {
    let pub_key_result = ec_recover(signature, msg_hash);

    match pub_key_result {
        Result::Err(e) => Result::Err(e),
        _ => {
            let pub_key = pub_key_result.unwrap();
            // Note that EVM addresses are derived from the Keccak256 hash of the pubkey (not sha256)
            let pubkey_hash = pub_key_address(((pub_key.bytes)[0], (pub_key.bytes)[1]));
            Ok(EvmAddress::from(pubkey_hash))
        }
    }
}
