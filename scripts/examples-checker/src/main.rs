//! Runs `forc build` or `forc fmt --check` on Sway projects specified by paths or within Sway `examples` directory.
//!
//! NOTE: This expects `forc`, `forc-fmt`, and `cargo` to be available in `PATH`.

use anyhow::{anyhow, Result};
use clap::{ArgEnum, ArgGroup, Parser};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[clap(name = "examples-checker", about = "Forc Examples Checker")]
#[clap(group(
        ArgGroup::new("run").required(true).args(&["paths", "all-examples"])))]
struct Cli {
    /// Targets all Sway examples found at the paths listed
    #[clap(long = "paths", short = 'p', multiple_values = true)]
    pub paths: Vec<PathBuf>,

    /// Targets all Sway examples under /examples
    #[clap(long = "all-examples")]
    pub all_examples: bool,

    /// Forc command to run on examples
    #[clap(arg_enum)]
    command_kind: CommandKind,
}

#[derive(ArgEnum, Clone, PartialEq, Eq)]
enum CommandKind {
    Build,
    Fmt,
}

impl std::fmt::Display for CommandKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CommandKind::Build => write!(f, "forc build"),
            CommandKind::Fmt => write!(f, "forc fmt --check"),
        }
    }
}

// Check if the given directory contains `Forc.toml` at its root.
fn dir_contains_forc_manifest(path: &Path) -> bool {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().file_name().and_then(|s| s.to_str()) == Some("Forc.toml") {
                return true;
            }
        }
    }
    false
}

fn get_sway_path() -> PathBuf {
    let mut curr_path = std::env::current_dir().unwrap();
    loop {
        if curr_path.join("examples").exists() && curr_path.join("sway-core").exists() {
            return curr_path;
        }
        curr_path = curr_path.parent().unwrap().to_path_buf()
    }
}

/// Returns true if command ran successfully, false otherwise.
fn run_forc_command(path: &Path, cmd_args: &[&str]) -> bool {
    if !path.is_dir() || !dir_contains_forc_manifest(path) {
        return false;
    }

    let output = std::process::Command::new("forc")
        .args(cmd_args)
        .arg(path)
        .output()
        .expect("failed to run command for example project");

    if !output.status.success() {
        io::stdout().write_all(&output.stdout).unwrap();
        io::stdout().write_all(&output.stderr).unwrap();
        false
    } else {
        true
    }
}

fn run_forc_build(path: &Path) -> bool {
    run_forc_command(path, &["build", "--path"])
}

fn run_forc_fmt(path: &Path) -> bool {
    run_forc_command(path, &["fmt", "--check", "--path"])
}

fn print_summary(summary: &[(PathBuf, bool)], command_kind: CommandKind) -> Result<()> {
    println!("\nSummary for command {}:", command_kind);
    let mut successes = 0;
    for (path, success) in summary {
        let (checkmark, status) = if *success {
            ("[✓]", "succeeded")
        } else {
            ("[x]", "failed")
        };
        println!("  {}: {} {}!", checkmark, path.display(), status);
        if *success {
            successes += 1;
        }
    }
    let failures = summary.len() - successes;
    let successes_str = if successes == 1 {
        "success"
    } else {
        "successes"
    };
    let failures_str = if failures == 1 { "failure" } else { "failures" };
    println!(
        "{} {}, {} {}",
        successes, successes_str, failures, failures_str
    );

    if failures > 0 {
        return Err(anyhow!("{} failed to run for some examples", command_kind));
    }

    Ok(())
}

fn exec(paths: Vec<PathBuf>, all_examples: bool, command_kind: CommandKind) -> Result<()> {
    let mut summary: Vec<(PathBuf, bool)> = vec![];

    if all_examples {
        let examples_dir = get_sway_path().join("examples");

        for res in fs::read_dir(examples_dir).expect("Failed to read examples directory") {
            let path = match res {
                Ok(entry) => entry.path(),
                _ => continue,
            };

            let success: bool = if command_kind == CommandKind::Build {
                run_forc_build(&path)
            } else {
                run_forc_fmt(&path)
            };

            summary.push((path, success));
        }
    } else {
        for path in paths {
            let success: bool = if command_kind == CommandKind::Build {
                run_forc_build(&path)
            } else {
                run_forc_fmt(&path)
            };

            summary.push((path, success));
        }
    }

    print_summary(&summary, command_kind)?;
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    exec(cli.paths, cli.all_examples, cli.command_kind)?;
    Ok(())
}
