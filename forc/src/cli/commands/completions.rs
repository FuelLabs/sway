use crate::cli::plugin::find_all;
use clap::{Command as ClapCommand, CommandFactory, Parser};
use clap_complete::{generate, Generator};
use forc_util::{cli::CommandInfo, ForcResult};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
};

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
    target: Option<Target>,
}

fn generate_autocomplete_script(target: Target, writer: &mut dyn Write) {
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
    match target {
        Target::Bash => print_completions(clap_complete::Shell::Bash, &mut cmd, writer),
        Target::Zsh => print_completions(clap_complete::Shell::Zsh, &mut cmd, writer),
        Target::Fish => print_completions(clap_complete::Shell::Fish, &mut cmd, writer),
        Target::Elvish => print_completions(clap_complete::Shell::Elvish, &mut cmd, writer),
        Target::PowerShell => print_completions(clap_complete::Shell::PowerShell, &mut cmd, writer),
        Target::Fig => print_completions(clap_complete_fig::Fig, &mut cmd, writer),
    }
}

pub(crate) fn exec(command: Command) -> ForcResult<()> {
    let target = command.target.unwrap_or_else(|| {
        if let Ok(shell) = std::env::var("SHELL") {
            match Path::new(shell.as_str())
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
            {
                "bash" => Target::Bash,
                "zsh" => Target::Zsh,
                "fish" => Target::Fish,
                "pwsh" => Target::PowerShell,
                "elvish" => Target::Elvish,
                _ => Target::Bash,
            }
        } else {
            Target::Bash
        }
    });

    // Where to store the autocomplete script, ZSH requires a special path, the other shells are
    // stored in the same path and they are referenced form their respective config files
    let home_dir = home::home_dir().map(|p| p.display().to_string()).unwrap();
    let forc_autocomplete_path = format!("{}/.forc/_forc", home_dir);
    let mut file = File::create(&forc_autocomplete_path).unwrap_or_else(|_| {
        panic!("Cannot write to the autocomplete file in path {forc_autocomplete_path}")
    });
    generate_autocomplete_script(target, &mut file);

    // Check if the shell config file already has the forc completions. Some shells do not require
    // this step, therefore this maybe None
    let user_shell_config = match target {
        Target::Fish => format!("{}/.config/fish/config.fish", home_dir),
        Target::Elvish => format!("{}/.elvish/rc.elv", home_dir),
        Target::PowerShell => format!(
            "{}/.config/powershell/Microsoft.PowerShell_profile.ps1",
            home_dir
        ),
        Target::Bash => format!("{}/.bash_profile", home_dir),
        Target::Fig => format!("{}/.config/fig/fig.fish", home_dir),
        Target::Zsh => format!("{}/.zshrc", home_dir),
    };
    let autocomplete_activation_script = match target {
        Target::Bash => format!("source {}", forc_autocomplete_path),
        Target::Zsh => format!(
            "fpath=({} \"{}\")\\nautoload -Uz compinit && compinit",
            Path::new(&forc_autocomplete_path)
                .parent()
                .unwrap()
                .display(),
            "$fpath[@]",
        ),
        _ => format!("source {}", forc_autocomplete_path),
    };

    if let Ok(file) = File::open(&user_shell_config) {
        // Update the user_shell_config
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            if line.contains(&autocomplete_activation_script) {
                println!("Forc completions is already installed");
                return Ok(());
            }
        }
    }

    println!("To finish the installation of the autocompletition script, please run the following command:\n");
    println!(
        "\techo '{}' >> {}",
        autocomplete_activation_script, user_shell_config
    );

    Ok(())
}

#[inline]
fn print_completions<G: Generator>(gen: G, cmd: &mut ClapCommand, writer: &mut dyn Write) {
    generate(gen, cmd, cmd.get_name().to_string(), writer);
}
