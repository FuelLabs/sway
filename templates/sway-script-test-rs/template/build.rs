//! This build script is used to compile the sway project using the `forc` prior to running tests.

use std::process::Command;

fn main() {
    Command::new("forc").args(&["build"]).status().unwrap();
}