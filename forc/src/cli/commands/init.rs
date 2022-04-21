use crate::ops::forc_init;
use crate::utils::project_type::ProjectType;
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
    /// The name of your project
    pub project_name: String,
    /// There are four accepted project types: contract, script, predicate or library.
    /// If a project type is not given the default project type is a contract.
    #[clap(parse(try_from_str), default_value_t = ProjectType::Contract, help = PROJECT_HELP)]
    pub project_type: ProjectType,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    forc_init::init(command)?;
    Ok(())
}
