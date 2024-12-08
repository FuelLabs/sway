use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
pub struct LocalCmd {
    #[clap(long = "chain-config")]
    pub chain_config: Option<PathBuf>,
    #[clap(long = "port")]
    pub port: Option<u16>,
}
