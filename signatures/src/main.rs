// use the anyhow crate for easy idiomatic error handling
use anyhow::Result;
// use the ethers_core rand for rng
use ethers_core::rand::thread_rng;
// use the ethers_signers crate to manage LocalWallet and Signer
use ethers_signers::{LocalWallet, Signer};
// use elliptic_curve::*;
use hex_literal::hex;
use libsecp256k1::{PublicKey, SecretKey, Signature};
use sha2::*;

// Use the `tokio::main` macro for using async on the main function
#[tokio::main]
async fn main() -> Result<()> {
    let seed = [
        0x42, 0xfa, 0xf7, 0xc1, 0x63, 0x0a, 0x38, 0x26, 0xc2, 0xa1, 0x41, 0xee, 0x18, 0x72, 0xb7,
        0x6d, 0xc4, 0x7c, 0xc2, 0x7d, 0xf5, 0x8b, 0x9a, 0x8f, 0xbd, 0x8a, 0xe0, 0xeb, 0x1e, 0x5a,
        0x65, 0xc8,
    ];

    let seckey: SecretKey = SecretKey::parse(&seed).unwrap();
    let pubkey: PublicKey = PublicKey::from_secret_key(&seckey);
    println!("The public Key is: {:?}", pubkey);
    // The concated public Key is: 1847637560306280107622622502715552825033773557495040513239277815084649981513331589016081394344042535895449449754985848875729991940298763648723723322522
    //its hex is 0x907F3D3FD34158C4A4348823692272EA4211FD1543D2412E60F9CA87C92283C7802B042FCE39869C4C652024F8F635F3EA885858116133D114DD1D39D849A

    // let mut key_hasher = Sha256::new();
    // key_hasher.update(&pubkey);

    // create a Sha256 object
    let mut hasher = Sha256::new();

    // write input message
    hasher.update(b"Hello from Fuel! The number is 42.");

    // read hash digest and consume hasher
    let result = hasher.finalize();

    assert_eq!(
        result[..],
        hex!(
            "
    ded50450211738811b352a1c4534bd38c9d285063c8aa45517fec882ac91294d
    "
        )[..]
    );

    // @todo find Sha256 function. For now calculate this with online tool: https://emn178.github.io/online-tools/sha256.html
    // let address = Sha256(pubkey)
    // let address = 0x7fbf2c5091da720d117dae2ea8e47d6e7b8b96dbe96dfbe7ae6f24c0ee5e132d;

    // Generate a random wallet
    let wallet = LocalWallet::new(&mut thread_rng());
    println!("Wallet addr:    {:?}", wallet.address());

    // Declare the message you want to sign.
    let message = "Hello from Fuel! The number is 42.";

    // sign message from your wallet and print out signature produced.
    let signature = wallet.sign_message(message).await?;
    println!("Signature       {}", signature);

    // verify the signature produced from your wallet.
    signature.verify(message, wallet.address()).unwrap();

    let recovered_addr = signature.recover(message).unwrap();

    assert_eq!(recovered_addr, wallet.address());
    println!("Signed by:      {:?}", wallet.address());
    println!("Recovered addr: {:?}", recovered_addr);

    Ok(())
}
