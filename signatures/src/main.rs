// use the anyhow crate for easy idiomatic error handling
use anyhow::Result;
// use the ethers_core rand for rng
use ethers_core::rand::thread_rng;
// use the ethers_signers crate to manage LocalWallet and Signer
use ethers_signers::{LocalWallet, Signer};
use elliptic_curve::*;

// Use the `tokio::main` macro for using async on the main function
#[tokio::main]
async fn main() -> Result<()> {

    // let secret_key = SecretKey::from_bytes("0x007").unwrap();

    // let public_key = SecretKey::public_key(&secret_key);

    // Generate a random wallet
    let wallet = LocalWallet::new(&mut thread_rng());

    // Declare the message you want to sign.
    let message = "Some data";

    // sign message from your wallet and print out signature produced.
    let signature = wallet.sign_message(message).await?;
    println!("Produced signature {}", signature);

    // verify the signature produced from your wallet.
    signature.verify(message, wallet.address()).unwrap();

    let recovered_addr = signature.recover(message).unwrap();

    assert_eq!(recovered_addr, wallet.address());
    println!("Verified signature produced by {:?}!", wallet.address());
    println!(" {:?}, {:?}", recovered_addr, wallet.address());
    // println!("public Key {:?}", public_key);

    Ok(())
}
