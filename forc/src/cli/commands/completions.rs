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
        // Each shell gets its own directories, so that the tests, which run in
        // parallel, do not interfere with each other.
        let bin_root = std::env::temp_dir().join(format!("forc-completions-{shell}-bin"));
        let home = std::env::temp_dir().join(format!("forc-completions-{shell}-home"));

        std::fs::create_dir_all(&bin_root).expect("create bin root");
        suppress_sudo_hint(&bin_root);

        let runtime = R::new(bin_root, home).expect("runtime");
        build_script_and_test(runtime, shell, "forc", &Forc::possible_values());
    }

    /// Suppresses Ubuntu's "sudo hint" in the shells spawned by the tests, by
    /// placing a `groups` stub into `bin_root`.
    ///
    /// Ubuntu's system-wide `/etc/bash.bashrc`, which an interactive `bash`
    /// sources even when started with `--rcfile`, prints the sudo hint ("To
    /// run a command as administrator (user \"root\"), use \"sudo
    /// <command>\". ...") when the user is in the `sudo` group and `$HOME`
    /// contains neither `.sudo_as_admin_successful` nor `.hushlogin`. That is
    /// the case, e.g., on the Ubuntu 26.04 CI runner images, where the hint
    /// breaks the tests in two ways: it pollutes the captured completion
    /// output, and - being printed well before the completions are rendered -
    /// it makes `completest`'s wait-for-output-quiescence logic consider the
    /// shell done and kill it before the completions are even rendered.
    ///
    /// The hint is conditioned on the output of `groups`, which the shell
    /// resolves via `PATH`, and `completest` puts `bin_root` at the front of
    /// the `PATH` of the shells it spawns. A `groups` stub that reports no
    /// sudo-like group therefore suppresses the hint, deterministically and
    /// only within the spawned shells.
    fn suppress_sudo_hint(bin_root: &std::path::Path) {
        use std::os::unix::fs::PermissionsExt;

        let groups_stub = bin_root.join("groups");
        std::fs::write(&groups_stub, "#!/bin/sh\necho users\n").expect("write groups stub");
        let mut permissions = std::fs::metadata(&groups_stub)
            .expect("groups stub metadata")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&groups_stub, permissions).expect("make groups stub executable");
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
            if let Ok(output) = runtime.complete(&format!("{command_to_complete} \t\t"), &term) {
                output
            } else {
                println!("Skipping {shell}");
                return;
            };

        for expectation in expectations {
            assert!(
                output.contains(expectation),
                "Failed find {expectation} in {output}"
            );
        }
    }
}
