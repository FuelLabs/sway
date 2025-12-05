use super::KeyType;
use anyhow::Result;
use fuel_core_types::{fuel_crypto::SecretKey, fuel_tx::Input};
use libp2p_identity::{secp256k1, Keypair, PeerId};
use serde_json::json;
use std::{ops::Deref, str::FromStr};

forc_types::cli_examples! {
    crate::Command {
        [ Parses the secret of a block production  => "forc crypto parse-secret \"f5204427d0ab9a311266c96a377f7c329cb8a41b9088225b6fcf40eefb423e28\"" ]
        [ Parses the secret of a peering  => "forc crypto parse-secret -k peering \"f5204427d0ab9a311266c96a377f7c329cb8a41b9088225b6fcf40eefb423e28\"" ]
    }
}

/// Parses a private key to view the associated public key
#[derive(Debug, clap::Args)]
#[clap(
    version,
    after_help = help(),
)]
pub struct Arg {
    /// A private key in hex format
    secret: String,
    /// Key type to generate. It can either be `block-production` or `peering`.
    #[clap(
        long = "key-type",
        short = 'k',
        value_enum,
        default_value = <&'static str>::from(KeyType::BlockProduction),
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
