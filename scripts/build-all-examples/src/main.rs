//! Runs `forc build` and `forc fmt --check` for all projects under the Sway `examples` directory.
//!
//! NOTE: This expects `forc`, `forc-fmt`, and `cargo` to be available in `PATH`.

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[clap(name = "build-all-examples", about = "Forc Examples Builder")]
struct Cli {
    /// the command to run
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    BuildExamples(BuildExamplesCommand),
}

#[derive(Debug, Parser)]
struct BuildExamplesCommand {
    #[clap(long)]
    pub paths: Option<Vec<String>>,
}

fn get_sway_path() -> PathBuf {
    let mut curr_path = std::env::current_dir().unwrap();
    loop {
        if curr_path.ends_with("sway") {
            return curr_path;
        }
        curr_path = curr_path.parent().unwrap().to_path_buf()
    }
}

fn build_and_report_result(entry: fs::DirEntry) -> (PathBuf, bool) {
    let success = false;
    let path = entry.path();

    if !path.is_dir() || !dir_contains_forc_manifest(&path) {
        return (path, success);
    }

    let build_output = std::process::Command::new("forc")
        .args(["build", "--path"])
        .arg(&path)
        .output()
        .expect("failed to run `forc build` for example project");

    let fmt_output = std::process::Command::new("forc")
        .args(["fmt", "--check", "--path"])
        .arg(&path)
        .output()
        .expect("failed to run `forc fmt --check` for example project");

    // Print output on failure so we can read it in CI.
    let success = if !build_output.status.success() || !fmt_output.status.success() {
        io::stdout().write_all(&build_output.stdout).unwrap();
        io::stdout().write_all(&fmt_output.stdout).unwrap();
        io::stdout().write_all(&build_output.stderr).unwrap();
        io::stdout().write_all(&fmt_output.stderr).unwrap();
        false
    } else {
        true
    };

    (path, success)
}

fn build_examples(command: BuildExamplesCommand) {
    let BuildExamplesCommand { paths } = command;
    let paths_vec = Vec::from(paths);

    let mut summary: Vec<(PathBuf, bool)> = vec![];
    for res in fs::read_dir(paths).expect("failed to walk examples directory") {
        let entry = match res {
            Ok(entry) => entry,
            _ => continue,
        };
        summary.push(build_and_report_result(entry))
    }
}

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();
    let examples_dir = get_sway_path().join("examples");

    match cli.command {
        Commands::BuildExamples(command) => build_examples(command)?,
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
