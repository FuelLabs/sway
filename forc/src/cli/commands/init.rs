use crate::ops::forc_init;
use anyhow::Result;
use clap::Parser;

static TEMPLATE_HELP: &str = r#"Initialize a new project from a template.
Option 1: Create a new project using supported examples from FuelLabs
example templates: 
 - counter
Option 2: Create a new project from a GitHub URL containing a sway project.
Option 3: Create a new project from a sway project on your local machine."#;

/// Create a new Forc project.
#[derive(Debug, Parser)]
pub struct Command {
    /// Initialize a new project from a template
    #[clap(short, long, help = TEMPLATE_HELP)]
    pub template: Option<String>,
    /// The name of your project
    pub project_name: String,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    forc_init::init(command)?;
    Ok(())
}
