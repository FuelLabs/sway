use crate::args::read_content_or_filepath;
use anyhow::Result;
use fuel_crypto::{fuel_types::Address, Message, Signature};
use fuels_core::types::bech32::Bech32Address;
use serde_json::json;

forc_util::cli_examples! {
    [ Get the public key from a message and its signature => crypto r#"get-public-key \
        0x1eff08081394b72239a0cf7ff6b499213dcb7a338bedbd75d072d504588ef27a1f74d5ceb2f111ec02ede097fb09ed00aa9867922ed39299dae0b1afc0fa8661 \
        "This is a message that is signed""# ]
}

/// Parse a secret key to view the associated public key
#[derive(Debug, clap::Args)]
#[clap(
    author,
    version,
    about = "Get the public key from a message and its signature",
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
