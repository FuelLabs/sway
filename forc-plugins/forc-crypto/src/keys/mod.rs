use clap::ValueEnum;

pub mod get_public_key;
pub mod new_key;
pub mod parse_secret;

#[derive(Clone, Debug, Default, ValueEnum)]
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
