//! This file will be hosted here until
//! https://github.com/FuelLabs/sway/issues/5170 is fixed
use super::KeyType;
use anyhow::Result;
use fuel_core_types::{
    fuel_crypto::{
        rand::{prelude::StdRng, SeedableRng},
        SecretKey,
    },
    fuel_tx::Input,
};
use libp2p_identity::{secp256k1, Keypair, PeerId};
use serde_json::json;
use std::ops::Deref;

/// Generate a random new secret & public key in the format expected by fuel-core
#[derive(Debug, clap::Args)]
#[clap(author, version, about = "Creates a new key for use with fuel-core")]
pub struct Arg {
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
    let mut rng = StdRng::from_entropy();
    let secret = SecretKey::random(&mut rng);
    let public_key = secret.public_key();
    let secret_str = secret.to_string();

    let output = match arg.key_type {
        KeyType::BlockProduction => {
            let address = Input::owner(&public_key);
            json!({
                "secret": secret_str,
                "address": address,
                "type": <KeyType as std::convert::Into<&'static str>>::into(KeyType::BlockProduction),
            })
        }
        KeyType::Peering => {
            let mut bytes = *secret.deref();
            let p2p_secret = secp256k1::SecretKey::try_from_bytes(&mut bytes)
                .expect("Should be a valid private key");
            let p2p_keypair = secp256k1::Keypair::from(p2p_secret);
            let libp2p_keypair = Keypair::from(p2p_keypair);
            let peer_id = PeerId::from_public_key(&libp2p_keypair.public());
            json!({
                "secret": secret_str,
                "peer_id": peer_id.to_string(),
                "type": <KeyType as std::convert::Into<&'static str>>::into(KeyType::Peering),
            })
        }
    };
    Ok(output)
}
