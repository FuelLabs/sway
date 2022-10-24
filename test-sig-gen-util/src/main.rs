//! This utility generates data used for testing Sway's ec_recover function (stdlib/ecr.sw).
//! NOT to be used for key-generation as this is NEITHER SECURE NOR RANDOM !!!

use fuel_crypto::Hasher;

use fuel_vm::{crypto, prelude::*};

use anyhow::Result;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use sha3::{Digest, Keccak256};
use std::str::FromStr;

// A keccak-256 method for generating EVM signatures
fn keccak_hash<B>(data: B) -> Bytes32
where
    B: AsRef<[u8]>,
{
    // create a Keccak256 object
    let mut hasher = Keccak256::new();
    // write input message
    hasher.update(data);
    <[u8; Bytes32::LEN]>::from(hasher.finalize()).into()
}

fn main() -> Result<()> {
    let secp = Secp256k1::new();
    let secret =
        SecretKey::from_str("3b940b5586823dfd02ae3b461bb4336b5ecbaefd6627aa922efc048fec0c881c")
            .unwrap();
    let public = PublicKey::from_secret_key(&secp, &secret).serialize_uncompressed();
    let public = Bytes64::try_from(&public[1..]).expect("Failed to parse public key!");
    // 64 byte fuel address is the sha-256 hash of the public key.
    let address = Hasher::hash(&public[..]);
    let evm_pubkeyhash = keccak_hash(&public[..]);

    let message = b"The gift of words is the gift of deception and illusion.";
    let e = Hasher::hash(&message[..]);
    let sig = crypto::secp256k1_sign_compact_recoverable(secret.as_ref(), e.as_ref())
        .expect("Failed to generate signature");

    tracing::info!("Secret Key: {:?}", secret);
    tracing::info!("Public Key: {:?}", public);
    tracing::info!("Fuel Address (sha2-256): {:?}", address);
    tracing::info!("EVM pubkey hash (keccak256): {:?}", evm_pubkeyhash);
    tracing::info!("Message Hash: {:?}", e);
    tracing::info!("Signature: {:?}", sig);

    Ok(())
}
