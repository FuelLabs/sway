use clap::ValueEnum;
use fuel_core_keygen::KeyType as ExternalKeyType;

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum KeyType {
    #[default]
    BlockProduction,
    Peering,
}

impl From<KeyType> for &'static str {
    fn from(key_type: KeyType) -> Self {
        match key_type {
            KeyType::BlockProduction => "block-production",
            KeyType::Peering => "p2p",
        }
    }
}

impl Into<ExternalKeyType> for KeyType {
    fn into(self) -> ExternalKeyType {
        match self {
            KeyType::BlockProduction => ExternalKeyType::BlockProduction,
            KeyType::Peering => ExternalKeyType::Peering,
        }
    }
}

/// Parse a secret key to view the associated public key
#[derive(Debug, clap::Args)]
#[clap(author, version, about)]
pub struct ParseSecret {
    /// A private key in hex format
    pub secret: String,
    /// Print the JSON in pretty format
    #[clap(long = "pretty", short = 'p')]
    pub pretty: bool,
    /// Key type to generate. It can either be `block-production` or `peering`.
    #[clap(
        long = "key-type",
        short = 'k',
        value_enum,
        default_value = <KeyType as std::convert::Into<&'static str>>::into(KeyType::BlockProduction),
    )]
    pub key_type: KeyType,
}

/// Generate a random new secret & public key in the format expected by fuel-core
#[derive(Debug, clap::Args)]
#[clap(author, version, about)]
pub struct NewKey {
    /// Print the JSON in pretty format
    #[clap(long = "pretty", short = 'p')]
    pub pretty: bool,
    /// Key type to generate. It can either be `block-production` or `peering`.
    #[clap(
        long = "key-type",
        short = 'k',
        value_enum,
        default_value = <KeyType as std::convert::Into<&'static str>>::into(KeyType::BlockProduction),
    )]
    pub key_type: KeyType,
}
