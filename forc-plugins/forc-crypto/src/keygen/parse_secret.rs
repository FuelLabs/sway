//! This file will be hosted here until
//! https://github.com/FuelLabs/sway/issues/5170 is fixed
use super::KeyType;
use anyhow::Result;
use fuel_core_types::{fuel_crypto::SecretKey, fuel_tx::Input};
use libp2p_identity::{secp256k1, Keypair, PeerId};
use serde_json::json;
use std::{ops::Deref, str::FromStr};

const ABOUT: &str = "Parses a private key to view the associated public key";

pub(crate) fn examples() -> String {
    let secret = "{secret}";
    format!(
        r#"# {}

    ## Parses the secret for a block-production secret
    forc crypto parse-secret "{secret}"

    ## Parses the secret for a block-production secret
    forc crypto parse-secret "{secret}" -k peering
    "#,
        ABOUT
    )
}

fn after_long_help() -> &'static str {
    Box::leak(
        format!(
            r#"EXAMPLES:
    {}"#,
            examples()
        )
        .into_boxed_str(),
    )
}

/// Parse a secret key to view the associated public key
#[derive(Debug, clap::Args)]
#[clap(
    author,
    version,
    about = ABOUT,
    after_long_help = after_long_help(),
)]
pub struct Arg {
    /// A private key in hex format
    secret: String,
    /// Key type to generate. It can either be `block-production` or `peering`.
    #[clap(
        long = "key-type",
        short = 'k',
        value_enum,
        default_value = KeyType::BlockProduction.into(),
    )]
    key_type: KeyType,
}

pub fn handler(arg: Arg) -> Result<serde_json::Value> {
    let secret = SecretKey::from_str(&arg.secret)?;
    let output = match arg.key_type {
        KeyType::BlockProduction => {
            let address = Input::owner(&secret.public_key());
            let output = json!({
                "address": address.to_string(),
                "type": <KeyType as std::convert::Into<&'static str>>::into(KeyType::BlockProduction),
            });
            output
        }
        KeyType::Peering => {
            let mut bytes = *secret.deref();
            let p2p_secret = secp256k1::SecretKey::try_from_bytes(&mut bytes)
                .expect("Should be a valid private key");
            let p2p_keypair = secp256k1::Keypair::from(p2p_secret);
            let libp2p_keypair = Keypair::from(p2p_keypair);
            let peer_id = PeerId::from_public_key(&libp2p_keypair.public());
            let output = json!({
                "peer_id": peer_id.to_string(),
                "type": <KeyType as std::convert::Into<&'static str>>::into(KeyType::Peering),
            });
            output
        }
    };
    Ok(output)
}
