use clap::ValueEnum;

pub mod new_key;
pub mod parse_secret;

pub const BLOCK_PRODUCTION: &str = "block-production";
pub const P2P: &str = "p2p";

#[derive(Clone, Debug, Default, ValueEnum)]
pub enum KeyType {
    #[default]
    BlockProduction,
    Peering,
}
