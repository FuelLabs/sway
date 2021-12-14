//! This utility generates data used for testing Sway's ec_recover function (stdlib/ecr.sw).
//! NOT to be used for key-generation as this is NEITHER SECURE NOR RANDOM !!!

use anyhow::Result;
use secp256k1_test::{Message as SecpMessage, Secp256k1};
use sha256::digest_bytes;

fn main() -> Result<()> {
    let secp256k1 = Secp256k1::new();
    // @todo improve this to allow starting with a string
    let message_arr = [42u8; 32];
    // @note Not Secure!
    let secret_key = secp256k1_test::key::ONE_KEY; // the number 1 as a secret key
    let public_key = secp256k1_test::key::PublicKey::from_secret_key(&secp256k1, &secret_key);
    let message = SecpMessage::from_slice(&message_arr).unwrap();
    // @note sign_recoverable sig is not 128 bytes long!
    let signature = secp256k1.sign_recoverable(&message, &secret_key);

    let pubkey = public_key.serialize_uncompressed();
    let addr_hash = digest_bytes(&pubkey);
    let address = "0x".to_owned() + &addr_hash;
    assert_eq!(pubkey.len(), 65);
    let recovered = secp256k1.recover(&message, &signature).unwrap();
    assert_eq!(recovered, public_key);
    let sig = signature.serialize_compact();
//    let hex_sig = hex::encode(sig);
//    assert_eq!(sig.len(), 64);

    println!("private key: {}", secret_key);
    println!("public key: {} \n", public_key);
    println!("pubkey: {:?} \n", pubkey);
    println!("message: {:?}", message);
    println!("Address: {:?} \n", address);
    println!("Full Signature: {:?}", signature);
    println!("Serialized Signature: {:?} \n", sig);
//    println!("64-byte Hex Signature: {:?}", hex_sig);
//    assert_eq!(hex_sig.len(), 128);

    Ok(())
}
