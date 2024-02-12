use clap::{Command as ClapCommand, ValueEnum};
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell as BuiltInShell};
use forc_util::cli::CommandInfo;
use forc_util::ForcResult;
use std::collections::HashMap;
use std::str::FromStr;

use crate::cli::plugin::find_all;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
enum Shell {
    BuiltIn(BuiltInShell),
    Fig,
}

impl FromStr for Shell {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fig" => Ok(Shell::Fig),
            other => Ok(Shell::BuiltIn(<clap_complete::Shell as FromStr>::from_str(
                other,
            )?)),
        }
    }
}

impl ValueEnum for Shell {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Shell::BuiltIn(BuiltInShell::Bash),
            Shell::BuiltIn(BuiltInShell::Elvish),
            Shell::BuiltIn(BuiltInShell::Fish),
            Shell::BuiltIn(BuiltInShell::PowerShell),
            Shell::BuiltIn(BuiltInShell::Zsh),
            Shell::Fig,
        ]
    }

    fn to_possible_value<'a>(&self) -> Option<clap::PossibleValue<'a>> {
        match self {
            Shell::BuiltIn(shell) => shell.to_possible_value(),
            Shell::Fig => Some(clap::PossibleValue::new("fig")),
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
    #[clap(short = 'S', long)]
    shell: Shell,
}

pub(crate) fn exec(command: Command) -> ForcResult<()> {
    let mut cmd = CommandInfo::new(&super::super::Opt::command());

    let mut plugins = HashMap::new();
    find_all().for_each(|path| {
        let mut proc = std::process::Command::new(path.clone());
        proc.env("CLI_DUMP_DEFINITION", "1");
        if let Ok(proc) = proc.output() {
            if let Ok(mut command_info) = serde_json::from_slice::<CommandInfo>(&proc.stdout) {
                command_info.name = if let Some(name) = command_info.name.strip_prefix("forc-") {
                    name.to_string()
                } else {
                    command_info.name
                };
                plugins.insert(command_info.name.to_owned(), command_info);
            }
        }
    });

    let mut plugins = plugins.into_values().collect::<Vec<_>>();
    cmd.subcommands.append(&mut plugins);

    let mut cmd = cmd.to_clap();
    match command.shell {
        Shell::Fig => print_completions(clap_complete_fig::Fig, &mut cmd),
        Shell::BuiltIn(shell) => print_completions(shell, &mut cmd),
    }
    Ok(())
}

fn print_completions<G: Generator>(gen: G, cmd: &mut ClapCommand) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}
