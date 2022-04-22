use crate::ops::forc_init;
use anyhow::Result;
use clap::Parser;

const TEMPLATE_HELP: &str = r#"Initialize a new project from a template.

Example Templates: 
 - counter"#;

const PROJECT_HELP: &str = r#"Initialize a new project of a specific type. If a project type is not given the default project type is a contract.

Project Types:
 - contract
 - script
 - predicate
 - library"#;

/// Create a new Forc project.
#[derive(Debug, Parser)]
pub struct Command {
    /// Initialize a new project from a template
    #[clap(short, long, help = TEMPLATE_HELP)]
    pub template: Option<String>,
    /// There are four accepted project types: contract, script, predicate or library.
    /// If a project type is not given the default project type is a contract.
    #[clap(long, help = PROJECT_HELP)]
    pub contract: bool,
    #[clap(long, help = PROJECT_HELP)]
    pub script: bool,
    #[clap(long, help = PROJECT_HELP)]
    pub predicate: bool,
    #[clap(long = "lib", help = PROJECT_HELP)]
    pub library: bool,
    /// The name of your project
    pub project_name: String,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    forc_init::init(command)?;
    Ok(())
}
