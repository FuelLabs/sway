use clap::ValueEnum;

pub mod new_key;
pub mod parse_secret;

pub const BLOCK_PRODUCTION: &str = "block_production";
pub const P2P: &str = "p2p";

#[derive(Clone, Debug, Default, ValueEnum)]
#[clap(rename_all = "snake_case")]
pub enum KeyType {
    #[default]
    BlockProduction,
    Peering,
}
