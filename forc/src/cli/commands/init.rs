use crate::ops::forc_init;
use anyhow::Result;
use clap::Parser;

const TEMPLATE_HELP: &str = r#"Initialize a new project from a template.

Example Templates: 
 - counter"#;

/// Create a new Forc project.
#[derive(Debug, Parser)]
pub struct Command {
    /// Initialize a new project from a template
    #[clap(short, long, help = TEMPLATE_HELP)]
    pub template: Option<String>,
    /// The default program type, excluding all flags or adding this flag creates a basic contract program.
    #[clap(long)]
    pub contract: bool,
    /// Adding this flag creates an empty script program.
    #[clap(long)]
    pub script: bool,
    /// Adding this flag creates an empty predicate program.
    #[clap(long)]
    pub predicate: bool,
    /// Adding this flag creates an empty library program.
    #[clap(long)]
    pub library: bool,
    /// The name of your project
    pub project_name: String,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    forc_init::init(command)?;
    Ok(())
}
