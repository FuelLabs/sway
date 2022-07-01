use crate::ops::forc_build;
use anyhow::{bail, Result};
use clap::Parser;
use std::io::{BufRead, BufReader};
use std::process;
use std::thread;
use tracing::{error, info};

/// Run Rust-based tests on current project.
/// As of now, `forc test` is a simple wrapper on
/// `cargo test`; `forc new` also creates a rust
/// package under your project, named `tests`.
/// You can opt to either run these Rust tests by
/// using `forc test` or going inside the package
/// and using `cargo test`.
#[derive(Debug, Parser)]
pub(crate) struct Command {
    /// If specified, only run tests containing this string in their names
    pub test_name: Option<String>,
    /// Options passed through to the `cargo test` invocation.
    ///
    /// E.g. Given the following:
    ///
    /// `forc test --cargo-test-opts="--color always"`
    ///
    /// The `--color always` option is forwarded to `cargo test` like so:
    ///
    /// `cargo test --color always`
    #[clap(long)]
    pub cargo_test_opts: Option<String>,
    /// All trailing arguments following `--` are collected within this argument.
    ///
    /// E.g. Given the following:
    ///
    /// `forc test -- foo bar baz`
    ///
    /// The arguments `foo`, `bar` and `baz` are forwarded on to `cargo test` like so:
    ///
    /// `cargo test -- foo bar baz`
    #[clap(raw = true)]
    pub cargo_test_args: Vec<String>,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    // Ensure the project builds before running tests.
    forc_build::build(Default::default())?;

    let mut cmd = process::Command::new("cargo");
    cmd.arg("test");

    // Pass through cargo test options.
    let mut user_specified_color_opt = false;
    if let Some(opts) = command.cargo_test_opts {
        user_specified_color_opt = opts.contains("--color");
        for opt in opts.split_whitespace() {
            cmd.arg(&opt);
        }
    }

    // If the coloring option wasn't specified by the user, enable it ourselves. This is useful as
    // `cargo test`'s coloring is disabled by default when run as a child process.
    if !user_specified_color_opt {
        cmd.args(&["--color", "always"]);
    }

    // Pass through test name.
    if let Some(ref name) = command.test_name {
        cmd.arg(name);
    }

    // Pass through cargo test args.
    if !command.cargo_test_args.is_empty() {
        cmd.arg("--");
        cmd.args(&command.cargo_test_args);
    }

    let mut child = cmd
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .spawn()
        .unwrap();

    let out = BufReader::new(child.stdout.take().unwrap());
    let err = BufReader::new(child.stderr.take().unwrap());

    // Reading stderr on a separate thread so we keep things non-blocking
    let thread = thread::spawn(move || {
        err.lines().for_each(|line| error!("{}", line.unwrap()));
    });

    out.lines().for_each(|line| info!("{}", line.unwrap()));
    thread.join().unwrap();

    let child_success = match child.try_wait() {
        Ok(Some(returned_status)) => returned_status.success(),
        Ok(None) => child.wait().unwrap().success(),
        Err(_) => false,
    };

    match child_success {
        true => Ok(()),
        false => bail!("child test process failed"),
    }
}
