use crate::{ops::forc_index_deploy, utils::defaults};
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

/// Create a new Forc project in an existing directory.
#[derive(Debug, Parser)]
pub struct Command {
    /// URL at which to upload index assets
    #[clap(long, default_value = defaults::DEFAULT_INDEXER_URL, help = "URL at which to upload index assets.")]
    pub url: String,

    /// Path of the index manifest to upload
    #[clap(long, help = "Path of the index manifest to upload.")]
    pub manifest: PathBuf,

    /// Authentication header value
    #[clap(long, help = "Authentication header value.")]
    pub auth: Option<String>,
}

pub fn exec(command: Command) -> Result<()> {
    forc_index_deploy::init(command)?;
    Ok(())
}
