//! This file is migrated from https://github.com/FuelLabs/fuel-core/blob/master/bin/keygen/src/keygen.rs
use super::{KeyType, BLOCK_PRODUCTION, P2P};
use anyhow::Result;
use fuel_core_types::{fuel_crypto::SecretKey, fuel_tx::Input};
use libp2p_identity::{secp256k1, Keypair, PeerId};
use serde_json::json;
use std::{ops::Deref, str::FromStr};

/// Parse a secret key to view the associated public key
#[derive(Debug, clap::Args)]
#[clap(
    author,
    version,
    about = "Parses a private key to view the associated public key"
)]
pub struct Arg {
    secret: String,
    #[clap(long = "pretty", short = 'p')]
    pretty: bool,
    #[clap(
        long = "key-type",
        short = 'k',
        value_enum,
        default_value = BLOCK_PRODUCTION
    )]
    key_type: KeyType,
}

pub fn handler(arg: Arg) -> Result<String> {
    let secret = SecretKey::from_str(&arg.secret)?;
    let output = match arg.key_type {
        KeyType::BlockProduction => {
            let address = Input::owner(&secret.public_key());
            let output = json!({
                "address": address.to_string(),
                "type": BLOCK_PRODUCTION
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
                "type": P2P
            });
            output
        }
    };
    Ok(if arg.pretty {
        serde_json::to_string_pretty(&output)
    } else {
        serde_json::to_string(&output)
    }?)
}
