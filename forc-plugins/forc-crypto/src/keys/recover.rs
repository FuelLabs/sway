use crate::args::read_content_or_filepath;
use anyhow::Result;
use fuel_crypto::{fuel_types::Address, Message, Signature};
use fuels_core::types::bech32::Bech32Address;
use serde_json::json;

forc_util::cli_examples! {
    [ Recovers a public key from a message and its signature => crypto r#"recover-public-key \
        0xb0b2f29b52d95c1cba47ea7c7edeec6c84a0bd196df489e219f6f388b69d760479b994f4bae2d5f2abef7d5faf7d9f5ee3ea47ada4d15b7a7ee2777dcd7b36bb \
        "Blah blah blah""#]
}

/// Parse a secret key to view the associated public key
#[derive(Debug, clap::Args)]
#[clap(
    author,
    version,
    about = "Recovers a public key from a message and its signature",
    after_long_help = help(),
)]
pub struct Arg {
    /// A private key in hex format
    signature: Signature,
    /// A message
    message: Option<String>,
}

pub fn handler(arg: Arg) -> Result<serde_json::Value> {
    let message = Message::new(read_content_or_filepath(arg.message));
    let public_key = Signature::recover(&arg.signature, &message)?;

    let bytes = *public_key.hash();

    let bech32 = Bech32Address::from(Address::from(bytes));
    let addr = Address::from(bytes);

    Ok(json!({
        "PublicKey": public_key.to_string(),
        "Bench32": bech32.to_string(),
        "Address": addr.to_string(),
    }))
}
