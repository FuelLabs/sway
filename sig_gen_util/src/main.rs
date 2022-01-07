//! This utility generates data used for testing Sway's ec_recover function (stdlib/ecr.sw).
//! NOT to be used for key-generation as this is NEITHER SECURE NOR RANDOM !!!

use fuel_tx::crypto::Hasher;

use fuel_vm::crypto;
use fuel_vm::prelude::*;

use std::str::FromStr;
use secp256k1::{PublicKey, Secp256k1, SecretKey};
use anyhow::Result;

fn main() -> Result<()> {

    let secp = Secp256k1::new();
    let secret = SecretKey::from_str("3b940b5586823dfd02ae3b461bb4336b5ecbaefd6627aa922efc048fec0c881c").unwrap();
    let public = PublicKey::from_secret_key(&secp, &secret).serialize_uncompressed();
    let public = Bytes64::try_from(&public[1..]).expect("Failed to parse public key!");
    let address = Hasher::hash(&public[..]);

    let message = b"The gift of words is the gift of deception and illusion.";
    let e = Hasher::hash(&message[..]);
    let sig =
        crypto::secp256k1_sign_compact_recoverable(secret.as_ref(), e.as_ref()).expect("Failed to generate signature");

    println!("Secret Key: {:?}", secret);
    println!("Public Key: {:?}", public);
    println!("Address: {:?}", address);
    println!("Message Hash: {:?}", e);
    println!("Signature: {:?}", sig);

    Ok(())
}
