use crate::args::read_content_filepath_or_stdin;
use anyhow::Result;
use fuel_crypto::{fuel_types::Address, Message, Signature};
use serde_json::json;

forc_util::cli_examples! {
    crate::Command {
        [ Get the public key from a message and its signature => r#"forc crypto get-public-key \
            0x1eff08081394b72239a0cf7ff6b499213dcb7a338bedbd75d072d504588ef27a1f74d5ceb2f111ec02ede097fb09ed00aa9867922ed39299dae0b1afc0fa8661 \
            "This is a message that is signed""# ]
    }
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
    let message = Message::new(read_content_filepath_or_stdin(arg.message));
    let public_key = Signature::recover(&arg.signature, &message)?;

    let bytes = *public_key.hash();

    let addr = Address::from(bytes);

    Ok(json!({
        "PublicKey": public_key.to_string(),
        "Address": addr.to_string(),
    }))
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn expect_output() {
        let arg = Arg {
            signature: Signature::from_str("0x1eff08081394b72239a0cf7ff6b499213dcb7a338bedbd75d072d504588ef27a1f74d5ceb2f111ec02ede097fb09ed00aa9867922ed39299dae0b1afc0fa8661").unwrap(),
            message: Some("This is a message that is signed".to_string()),
        };
        let json = handler(arg).unwrap();
        assert_eq!(
            "4ef69bcba1cf6d1b4c05d746362d6e1af6245f2f71506f1516c7d52bf5d9ed9a",
            json.as_object()
                .unwrap()
                .get("Address")
                .unwrap()
                .as_str()
                .unwrap(),
        )
    }
}
