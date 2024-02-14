use clap::{Command as ClapCommand, ValueEnum};
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell as BuiltInShell};
use forc_util::cli::CommandInfo;
use forc_util::ForcResult;
use std::collections::HashMap;
use std::{fmt::Display, str::FromStr};

use crate::cli::plugin::find_all;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
enum Target {
    BuiltIn(BuiltInShell),
    Fig,
}

impl Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("no values are skipped")
            .get_name()
            .fmt(f)
    }
}

impl FromStr for Target {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fig" => Ok(Target::Fig),
            other => Ok(Target::BuiltIn(
                <clap_complete::Shell as FromStr>::from_str(other)?,
            )),
        }
    }
}

impl ValueEnum for Target {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Target::BuiltIn(BuiltInShell::Bash),
            Target::BuiltIn(BuiltInShell::Elvish),
            Target::BuiltIn(BuiltInShell::Fish),
            Target::BuiltIn(BuiltInShell::PowerShell),
            Target::BuiltIn(BuiltInShell::Zsh),
            Target::Fig,
        ]
    }

    fn to_possible_value<'a>(&self) -> Option<clap::PossibleValue<'a>> {
        match self {
            Target::BuiltIn(shell) => shell.to_possible_value(),
            Target::Fig => Some(clap::PossibleValue::new("fig")),
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
    #[clap(short = 'T', long)]
    target: Target,
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
    match command.target {
        Target::Fig => print_completions(clap_complete_fig::Fig, &mut cmd),
        Target::BuiltIn(shell) => print_completions(shell, &mut cmd),
    }
    Ok(())
}

fn print_completions<G: Generator>(gen: G, cmd: &mut ClapCommand) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut std::io::stdout());
}
