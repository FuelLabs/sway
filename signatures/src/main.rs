// use the anyhow crate for easy idiomatic error handling
use anyhow::Result;
use libsecp256k1::*;
use secp256k1_test::{Message as SecpMessage, Secp256k1};
use sha256::digest_bytes;

// Use the `tokio::main` macro for using async on the main function
#[tokio::main]
async fn main() -> Result<()> {
    let secp256k1 = Secp256k1::new();

    let message_arr = [42u8; 32];
    let secret_key = secp256k1_test::key::ONE_KEY; // the number 1 as a secret key
    let public_key = secp256k1_test::key::PublicKey::from_secret_key(&secp256k1, &secret_key);
    let message = SecpMessage::from_slice(&message_arr).unwrap();
    let signature = secp256k1.sign_recoverable(&message, &secret_key);

    println!("privkey: {}", secret_key);
    println!("pubkey: {}", public_key);
    println!("message: {:?}", message);
    println!("Signature: {:?}", signature);

    let pubkey_a = public_key.serialize_uncompressed();
    let address = digest_bytes(&pubkey_a);

    assert_eq!(pubkey_a.len(), 65);
    println!("Address: {:?}", address);
    // Derived Address: "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0"

    let ctx_message = Message::parse(&message_arr);
    let (rec_id, signature_a) = signature.serialize_compact();
    assert_eq!(signature_a.len(), 64);

    let ctx_sig = Signature::parse_standard(&signature_a).expect("signature is valid");

    // let ctx_pubkey = recover(
    //     &ctx_message,
    //     &ctx_sig,
    //     &RecoveryId::parse(rec_id.to_i32() as u8).unwrap(),
    // )
    // .unwrap();
    // let sp = ctx_pubkey.serialize();

    // let sps: &[u8] = &sp;
    // let gps: &[u8] = &pubkey_a;
    // assert_eq!(sps, gps);
    Ok(())
}
