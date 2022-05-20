use crate::ops::forc_template;
use anyhow::Result;
use clap::Parser;

/// Create a new Forc project from a git template.
#[derive(Debug, Parser)]
pub struct Command {
    /// The template url, should be a git repo.
    #[clap(long, short)]
    pub url: String,

    /// The name of the project that will be created
    #[clap(long, short)]
    pub project_name: String,

    /// The name of the template that needs to be fetched and used from git repo provided.
    #[clap(long, short)]
    pub template_name: Option<String>,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    forc_template::init(command)?;
    Ok(())
}
