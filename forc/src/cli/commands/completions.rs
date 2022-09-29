use anyhow::Result;
use clap::Command as ClapCommand;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell};

/// Generate tab-completion scripts for your shell
#[derive(Debug, Parser)]
pub struct Command {
    /// Specify shell to enable tab-completion for
    ///
    /// [possible values: zsh, bash, fish, powershell, elvish]
    ///
    /// For more info: https://fuellabs.github.io/sway/latest/forc/commands/forc_completions.html
    #[clap(short = 'S', long)]
    shell: Shell,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    let mut cmd = super::super::Opt::command();
    print_completions(command.shell, &mut cmd);
    Ok(())
}

fn print_completions<G: Generator>(gen: G, cmd: &mut ClapCommand) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}
