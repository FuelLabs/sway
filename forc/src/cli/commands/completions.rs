use anyhow::Result;
use clap::Command as ClapCommand;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell};

/// Generate tab-completion scripts for your shell
#[derive(Debug, Parser)]
pub struct Command {
    #[clap(
        short,
        long,
        help("[possible values: zsh, bash, fish, powershell, elvish]")
    )]
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
