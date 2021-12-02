use crate::utils::constants;
use std::env;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command as ProcessCommand;
use std::process::Stdio;
use std::thread;
use structopt::{self, StructOpt};

/// Run Rust-based tests on current project.
/// As of now, `forc test` is a simple wrapper on
/// `cargo test`; `forc init` also creates a rust
/// package under your project, named `tests`.
/// You can opt to either run these Rust tests by
/// using `forc test` or going inside the package
/// and using `cargo test`.
#[derive(Debug, StructOpt)]
pub(crate) struct Command {
    /// If specified, only run tests containing this string in their names
    pub test_name: Option<String>,
}

pub(crate) fn exec(command: Command) -> Result<(), String> {
    let test_path = env::current_dir()
        .unwrap()
        .join(Path::new(constants::TEST_DIRECTORY));

    // Change current directory to this project's test directory
    env::set_current_dir(&test_path).unwrap();

    // Cargo args setup
    let mut args: Vec<String> = vec!["test".into()];
    match command.test_name.to_owned() {
        Some(name) => args.push(name.clone()),
        None => {}
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
