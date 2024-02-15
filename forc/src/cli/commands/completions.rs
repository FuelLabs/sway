use crate::cli::plugin::find_all;
use clap::{Command as ClapCommand, CommandFactory, Parser};
use clap_complete::{generate, Generator};
use forc_util::{cli::CommandInfo, ForcResult};
use std::{
    collections::HashMap,
    fs::{metadata, File, OpenOptions},
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
        let mut proc = std::process::Command::new(path.clone());
        proc.env("CLI_DUMP_DEFINITION", "1");
        if let Ok(proc) = proc.output() {
            if let Ok(mut command_info) = serde_json::from_slice::<CommandInfo>(&proc.stdout) {
                command_info.name = if let Some(name) = command_info.name.strip_prefix("forc-") {
                    name.to_string()
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

fn is_writable<P: AsRef<Path>>(path: P) -> bool {
    if let Ok(metadata) = metadata(path) {
        return !metadata.permissions().readonly();
    }
    false
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

    let dir = home::home_dir().map(|p| p.display().to_string()).unwrap();
    let forc_autocomplete_path = match target {
        Target::Zsh => {
            let x = std::process::Command::new("zsh")
                .arg("-c")
                .arg("echo $fpath")
                .output()
                .expect("Cannot read $FPATH env variable")
                .stdout;
            let paths = String::from_utf8_lossy(&x)
                .split(' ')
                .filter(|path| is_writable(Path::new(path)))
                .map(|x| x.to_owned())
                .collect::<Vec<_>>();
            format!(
                "{}/_forc",
                paths.first().expect("No writable path found for zsh")
            )
        }
        _ => format!("{}/.forc.autocomplete", dir),
    };

    let mut file = File::create(&forc_autocomplete_path).expect("Open the shell config file");
    generate_autocomplete_script(target, &mut file);

    let user_shell_config = match target {
        Target::Fish => Some(format!("{}/.config/fish/config.fish", dir)),
        Target::Elvish => Some(format!("{}/.elvish/rc.elv", dir)),
        Target::PowerShell => Some(format!(
            "{}/.config/powershell/Microsoft.PowerShell_profile.ps1",
            dir
        )),
        Target::Bash => Some(format!("{}/.bashrc", dir)),
        Target::Fig => Some(format!("{}/.config/fig/fig.fish", dir)),
        _ => None,
    };

    if let Some(file_path) = user_shell_config {
        let file = File::open(&file_path).expect("Open the shell config file");
        let reader = BufReader::new(file);

        for line in reader.lines().map_while(Result::ok) {
            if line.contains(&forc_autocomplete_path) {
                println!("Forc completions is already installed");
                return Ok(());
            }
        }

        let mut file = OpenOptions::new().append(true).open(&file_path)?;
        writeln!(file, "source {}", forc_autocomplete_path,).unwrap();
    }

    println!("Forc completions is installed successfully");
    println!("\t The script is stored in {}", forc_autocomplete_path);

    Ok(())
}

#[inline]
fn print_completions<G: Generator>(gen: G, cmd: &mut ClapCommand, writer: &mut dyn Write) {
    generate(gen, cmd, cmd.get_name().to_string(), writer);
}
