//! This utility generates data used for testing Sway's ec_recover function (stdlib/ecr.sw).
//! NOT to be used for key-generation as this is NEITHER SECURE NOR RANDOM !!!

use std::str::FromStr;

use anyhow::Result;
use secp256k1_test::{Message as SecpMessage, Secp256k1};

use secp256k1_test::{
    recovery::{RecoverableSignature, RecoveryId},
    Message, constants::CURVE_ORDER,
};
use sha2::{Sha256};
use sha3::{Digest, Keccak256};

fn main() -> Result<()> {
    let secp = Secp256k1::new();

    // sha2
    let mut hasher = Sha256::new();
    hasher.update(b"Hello from Fuel-V2!");
    let result = hasher.finalize();

    // let message_arr = "It's a small(er) world";
    // let message_hashed = Keccak256::digest(message_arr).to_vec();
    let secret_key = secp256k1_test::key::SecretKey::from_str("45a915e4d060149eb4365960e6a7a45f334393093061116b197e3240065ff2d8")?;

    // @note secp256k1::key::PublicKey is only 33 bytes.
    // corresponding 64-byte public key is:
    // hi: 043a514176466fa815ed481ffad09110a2d344f6c9b78c1d14afc351c3a51be33
    // lo: d8072e77939dc03ba44790779b7a1025baf3003f6732430e20cd9b76d953391b3

    // secret key matched Fuel address: 9a40a306c3e17f2c7b5b10ff06226b7f019974d7cff65d94e1987800bf757793
    // https://emn178.github.io/online-tools/sha256.html

    let mut hash = [0; 32];
    hash[..].copy_from_slice(&result);
    // hash[..].copy_from_slice(&message_hashed);

    let message = SecpMessage::from_slice(&hash).unwrap();
    // @note sign_recoverable sig is not 128 bytes long! (130 bytes)
    let signature = secp.sign_recoverable(&message, &secret_key);
    let (rec_id, data) = signature.serialize_compact();
	let mut sig = [0; 65];

	// no need to check if s is low, it always is
	sig[0..64].copy_from_slice(&data[0..64]);
	sig[64] = rec_id.to_i32() as u8;
    let hex_sig = hex::encode(sig);
    let compact_sig = craft_compact_sig(sig);


    println!("private key: {}", secret_key);
    println!("message: {:?}", message);
    println!("Signature: {:?} \n", hex_sig);
    println!("Compact Signature: {:?} \n", compact_sig);

    //recovery
    let (rec,eth) = secp256k1_ecdsa_recover(&sig,&hash)?;
    println!("rec key:{:?}",hex::encode(rec));
    println!("eth key:{:?}",hex::encode(eth));

    Ok(())
}

fn craft_compact_sig(sig: [u8; 65]) -> u64 {
    // split sig into r, s & yParity(v)
    let r = &sig[0..31];
    let s = &sig[32..63];
    let y_parity = &sig[64];
    println!("r: {:?}", hex::encode(r));
    println!("s: {:?}", hex::encode(s));
    // println!("yParity: {:?}", hex::encode(y_parity));

    // check if `s` is large & fix if needed ? (ie: assert s * 2 < CURVE_ORDER)
    if s * 2 >= CURVE_ORDER {
        s = -s % CURVE_ORDER;
    };

    let y_parity_and_s = (y_parity << 255) | s;
    // let compact = r +(concat!) y_parity_and_s;

    // Only low-s values in signatures are valid (i.e. s <= secp256k1.n//2); s can be replaced with -s mod secp256k1.n during the signing process if it is high. Given this, the first bit of s will always be 0, and can be used to store the 1-bit v value.
    1
    // compact
}

fn secp256k1_ecdsa_recover(
    sig: &[u8; 65],
    msg: &[u8; 32],
) -> Result<(Vec<u8>,Vec<u8>), secp256k1_test::Error> {
    let sig = RecoverableSignature::from_compact(
        &sig[0..64],
        RecoveryId::from_i32((sig[64]) as i32)?,
    )?;

    let secp = Secp256k1::new();
    let public = secp.recover(&Message::from_slice(&msg[..32])?, &sig)?;

    let mut eth = vec![0; 20];
    let rec = &public.serialize_uncompressed();
    eth.copy_from_slice(&Keccak256::digest(&rec[1..])[12..]);
    Ok((rec.to_vec(),eth))
}
