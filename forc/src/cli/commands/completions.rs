use clap::{Command as ClapCommand, CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell};
use forc_util::ForcResult;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, clap::ValueEnum)]
enum Target {
    /// Bourne Again Shell (bash)
    Bash,
    /// Elvish shell
    Elvish,
    /// Friendly Interactive Shell (fish)
    Fish,
    /// PowerShell
    PowerShell,
    /// Z Shell (zsh)
    Zsh,
    /// Fig
    Fig,
}

impl ToString for Target {
    fn to_string(&self) -> String {
        match self {
            Target::Bash => "bash".to_string(),
            Target::Elvish => "elvish".to_string(),
            Target::Fish => "fish".to_string(),
            Target::PowerShell => "powershell".to_string(),
            Target::Zsh => "zsh".to_string(),
            Target::Fig => "fig".to_string(),
        }
    }
}

/// Generate tab-completion scripts for your shell
#[derive(Debug, Parser)]
pub struct Command {
    /// Specify shell to enable tab-completion for
    ///
    /// [possible values: zsh, bash, fish, powershell, elvish]
    ///
    /// For more info: https://fuellabs.github.io/sway/latest/forc/commands/forc_completions.html
    #[clap(short = 'T', long, value_enum)]
    target: Target,
}

pub(crate) fn exec(command: Command) -> ForcResult<()> {
    let mut cmd = super::super::Opt::command();
    match command.target {
        Target::Fig => print_completions(clap_complete_fig::Fig, &mut cmd),
        Target::Bash => print_completions(Shell::Bash, &mut cmd),
        Target::Elvish => print_completions(Shell::Elvish, &mut cmd),
        Target::PowerShell => print_completions(Shell::PowerShell, &mut cmd),
        Target::Zsh => print_completions(Shell::Zsh, &mut cmd),
        Target::Fish => print_completions(Shell::Fish, &mut cmd),
    }
    Ok(())
}

fn print_completions<G: Generator>(gen: G, cmd: &mut ClapCommand) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}
