use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
pub struct LocalCmd {
    #[clap(long)]
    pub chain_config: Option<PathBuf>,
    #[clap(long)]
    pub port: Option<u16>,
    #[clap(long)]
    /// If a db path is provided local node runs in persistent mode.
    pub db_path: Option<PathBuf>,
}
