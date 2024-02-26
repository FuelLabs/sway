use std::collections::HashMap;

use clap::{Command as ClapCommand, CommandFactory, Parser};
use clap_complete::{generate, Generator, Shell};
use forc_util::{cli::CommandInfo, ForcResult};

use crate::cli::plugin::find_all;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, clap::ValueEnum)]
enum Target {
    /// Bourne Again SHell (bash)
    Bash,
    /// Elvish shell
    Elvish,
    /// Friendly Interactive SHell (fish)
    Fish,
    /// PowerShell
    PowerShell,
    /// Z SHell (zsh)
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
    let mut cmd = CommandInfo::new(&super::super::Opt::command());
    let mut plugins = HashMap::new();
    find_all().for_each(|path| {
        if let Ok(proc) = std::process::Command::new(path.clone())
            .arg("--cli-definition")
            .output()
        {
            if let Ok(mut command_info) = serde_json::from_slice::<CommandInfo>(&proc.stdout) {
                command_info.name = if let Some(name) = path.file_name().and_then(|x| {
                    x.to_string_lossy()
                        .strip_prefix("forc-")
                        .map(|x| x.to_owned())
                }) {
                    name
                } else {
                    command_info.name
                };
                if !plugins.contains_key(&command_info.name) {
                    plugins.insert(command_info.name.to_owned(), command_info);
                }
            }
        }
    });

    let mut plugins = plugins.into_values().collect::<Vec<_>>();
    cmd.subcommands.append(&mut plugins);
    let mut cmd = cmd.to_clap();

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
