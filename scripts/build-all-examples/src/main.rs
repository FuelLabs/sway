//! Runs `forc build` and `forc fmt --check` for all projects under the Sway `examples` directory.
//!
//! NOTE: This expects `forc`, `forc-fmt`, and `cargo` to be available in `PATH`.

use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

fn get_sway_path() -> PathBuf {
    let mut curr_path = std::env::current_dir().unwrap();
    loop {
        if curr_path.ends_with("sway") {
            return curr_path;
        }
        curr_path = curr_path.parent().unwrap().to_path_buf()
    }
}

fn main() {
    let examples_dir = get_sway_path().join("examples");

    // Track discovered projects and whether or not they were successful.
    let mut summary: Vec<(PathBuf, bool)> = vec![];

    for res in fs::read_dir(examples_dir).expect("failed to walk examples directory") {
        let entry = match res {
            Ok(entry) => entry,
            _ => continue,
        };
        let path = entry.path();
        if !path.is_dir() || !dir_contains_forc_manifest(&path) {
            continue;
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

        summary.push((path, success));
    }

    println!("\nBuild and check formatting of all examples summary:");
    let mut successes = 0;
    for (path, success) in &summary {
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
        std::process::exit(1);
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
