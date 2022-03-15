use crate::ops::forc_build;
use anyhow::Result;
use clap::Parser;
use std::io::{BufRead, BufReader};
use std::process::Command as ProcessCommand;
use std::process::Stdio;
use std::thread;

/// Run Rust-based tests on current project.
/// As of now, `forc test` is a simple wrapper on
/// `cargo test`; `forc init` also creates a rust
/// package under your project, named `tests`.
/// You can opt to either run these Rust tests by
/// using `forc test` or going inside the package
/// and using `cargo test`.
#[derive(Debug, Parser)]
pub(crate) struct Command {
    /// If specified, only run tests containing this string in their names
    pub test_name: Option<String>,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    // Ensure the project builds before running tests.
    forc_build::build(Default::default())?;

    // Cargo args setup
    let mut args: Vec<String> = vec!["test".into()];
    if let Some(name) = command.test_name {
        args.push(name);
    };
    args.push("--color".into());
    args.push("always".into());
    args.push("--".into());
    args.push("--nocapture".into());

    let mut child = ProcessCommand::new("cargo")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let out = BufReader::new(child.stdout.take().unwrap());
    let err = BufReader::new(child.stderr.take().unwrap());

    // Reading stderr on a separate thread so we keep things non-blocking
    let thread = thread::spawn(move || {
        err.lines().for_each(|line| println!("{}", line.unwrap()));
    });

    out.lines().for_each(|line| println!("{}", line.unwrap()));
    thread.join().unwrap();

    Ok(())
}
