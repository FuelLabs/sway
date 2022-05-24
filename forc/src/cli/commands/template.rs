use crate::ops::forc_template;
use anyhow::Result;
use clap::Parser;

/// Create a new Forc project from a git template.
#[derive(Debug, Parser)]
pub struct Command {
    /// The template url, should be a git repo.
    #[clap(long, short, default_value = "https://github.com/fuellabs/sway")]
    pub url: String,

    /// The name of the template that needs to be fetched and used from git repo provided.
    #[clap(long, short)]
    pub template_name: Option<String>,

    /// The name of the project that will be created
    pub project_name: String,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    forc_template::init(command)?;
    Ok(())
}
