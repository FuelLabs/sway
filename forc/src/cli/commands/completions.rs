use std::fmt::Display;

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

impl Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Target::Bash => "bash".to_string(),
                Target::Elvish => "elvish".to_string(),
                Target::Fish => "fish".to_string(),
                Target::PowerShell => "powershell".to_string(),
                Target::Zsh => "zsh".to_string(),
                Target::Fig => "fig".to_string(),
            }
        )
    }
}

/// Generate tab-completion scripts for your shell
#[derive(Debug, Parser)]
pub struct Command {
    /// Specify shell to enable tab-completion for
    ///
    /// [possible values: zsh, bash, fish, powershell, elvish]
    ///
    /// For more info: https://fuellabs.github.io/sway/v0.18.1/forc/commands/forc_completions.html
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::cli::{Forc, Opt};

    #[test]
    fn bash() {
        testsuite::<completest_pty::BashRuntimeBuilder>(Shell::Bash);
    }

    #[test]
    fn zsh() {
        testsuite::<completest_pty::ZshRuntimeBuilder>(Shell::Zsh);
    }

    #[test]
    fn fish() {
        testsuite::<completest_pty::FishRuntimeBuilder>(Shell::Fish);
    }

    fn testsuite<R>(shell: Shell)
    where
        R: completest_pty::RuntimeBuilder,
    {
        let bin_root = "/tmp/bin".into();
        let home = "/tmp/home".into();
        let runtime = R::new(bin_root, home).expect("runtime");
        build_script_and_test(runtime, shell, "forc", &Forc::possible_values());
    }

    fn build_script_and_test<R>(
        mut runtime: R,
        shell: Shell,
        command_to_complete: &str,
        expectations: &[&str],
    ) where
        R: completest_pty::Runtime,
    {
        let term = completest_pty::Term::new();
        let mut cmd = Opt::command();
        let mut completion_script = Vec::<u8>::new();

        generate(shell, &mut cmd, "forc".to_owned(), &mut completion_script);

        runtime
            .register("forc", &String::from_utf8_lossy(&completion_script))
            .expect("register completion script");

        let output =
            if let Ok(output) = runtime.complete(&format!("{} \t\t", command_to_complete), &term) {
                output
            } else {
                println!("Skipping {}", shell);
                return;
            };

        for expectation in expectations {
            assert!(
                output.contains(expectation),
                "Failed find {} in {}",
                expectation,
                output
            );
        }
    }
}
