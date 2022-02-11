//! Runs `forc build` for all projects under the Sway `examples` directory.
//!
//! NOTE: This expects both `forc` and `cargo` to be available in `PATH`.

use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

fn main() {
    let proj_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let examples_dir = proj_dir
        .parent()
        .expect("failed to find examples directory");

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

        let output = std::process::Command::new("forc")
            .args(["build", "--path"])
            .arg(&path)
            .output()
            .expect("failed to run `forc build` for example project");

        // Print output on failure so we can read it in CI.
        let success = if !output.status.success() {
            io::stdout().write_all(&output.stdout).unwrap();
            io::stdout().write_all(&output.stderr).unwrap();
            false
        } else {
            true
        };

        summary.push((path, success));
    }

    println!("\nBuild all examples summary:");
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
