//! This utility generates data used for testing Sway's ec_recover function (stdlib/ecr.sw).
//! NOT to be used for key-generation as this is NEITHER SECURE NOR RANDOM !!!

use anyhow::Result;
use secp256k1_test::{Message as SecpMessage, Secp256k1};
// use sha256::digest_bytes;

fn main() -> Result<()> {
    let secp = Secp256k1::new();
    let message_arr = [42u8; 32];
    // @note Not Secure!
    let secret_key = secp256k1_test::key::ONE_KEY; // the number 1 as a secret key

    let message = SecpMessage::from_slice(&message_arr).unwrap();
    // @note sign_recoverable sig is not 128 bytes long! (130 bytes)
    let signature = secp.sign_recoverable(&message, &secret_key);
    let sig = signature.serialize_compact();


    println!("private key: {}", secret_key);
    println!("message: {:?}", message);
    println!("Full Signature: {:?}", signature);
    println!("Serialized Signature: {:?} \n", sig);


    Ok(())
}
