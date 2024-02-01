use crate::ops::forc_template;
use clap::Parser;
use forc_util::ForcResult;

forc_util::cli_examples! {
    [Create a new Forc project from an option template => forc "template demo --template-name option"]
    setup {
        let _ = std::fs::remove_dir_all("tests/demo");
    }
    teardown {
        let _ = std::fs::remove_dir_all("tests/demo");
    }
}

/// Create a new Forc project from a git template.
#[derive(Debug, Parser)]
#[clap(bin_name = "forc template", version, after_help = help())]
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

pub(crate) fn exec(command: Command) -> ForcResult<()> {
    forc_template::init(command)?;
    Ok(())
}
