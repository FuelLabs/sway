//! A `forc` plugin for running the Sway code formatter.

use anyhow::{bail, Result};
use clap::Parser;
use forc_util::{find_manifest_dir, println_green, println_red};
use prettydiff::{basic::DiffOp, diff_lines};
use std::default::Default;
use std::path::PathBuf;
use std::{fs, path::Path, sync::Arc};
use sway_core::BuildConfig;
use sway_fmt::{get_formatted_data, FormattingOptions};
use sway_utils::{constants, get_sway_files};
use taplo::formatter as taplo_fmt;

#[derive(Debug, Parser)]
#[clap(
    name = "forc-fmt",
    about = "Forc plugin for running the Sway code formatter.",
    version
)]
pub struct App {
    /// Run in 'check' mode.
    ///
    /// - Exits with `0` if input is formatted correctly.
    /// - Exits with `1` and prints a diff if formatting is required.
    #[clap(short, long)]
    pub check: bool,
    /// Path to the project, if not specified, current working directory will be used.
    #[clap(short, long)]
    pub path: Option<String>,
}

fn main() -> Result<()> {
    let app = App::parse();
    let dir = match app.path.clone() {
        Some(p) => PathBuf::from(p),
        None => std::env::current_dir()?,
    };
    format_pkg_at_dir(app, &dir)
}

/// Format the package at the given directory.
fn format_pkg_at_dir(app: App, dir: &Path) -> Result<()> {
    match find_manifest_dir(dir) {
        Some(path) => {
            let manifest_path = path.clone();
            let manifest_file = manifest_path.join(constants::MANIFEST_FILE_NAME);
            let files = get_sway_files(path);
            let mut contains_edits = false;

            for file in files {
                if let Ok(file_content) = fs::read_to_string(&file) {
                    // todo read options from manifest file
                    let formatting_options = FormattingOptions::default();
                    let file_content: Arc<str> = Arc::from(file_content);
                    let build_config = BuildConfig::root_from_file_name_and_manifest_path(
                        file.clone(),
                        manifest_path.clone(),
                    );
                    match get_formatted_data(
                        file_content.clone(),
                        formatting_options,
                        Some(&build_config),
                    ) {
                        Ok((_, formatted_content)) => {
                            if app.check {
                                if *file_content != *formatted_content {
                                    contains_edits = true;
                                    println!("\n{:?}\n", file);
                                    display_file_diff(&file_content, &formatted_content)?;
                                }
                            } else {
                                format_file(&file, &formatted_content)?;
                            }
                        }
                        Err(err) => {
                            // there could still be Sway files that are not part of the build
                            eprintln!("\nThis file: {:?} is not part of the build", file);
                            eprintln!("{}", err.join("\n"));
                        }
                    }
                }
            }
            // format manifest using taplo formatter
            if let Ok(file_content) = fs::read_to_string(&manifest_file) {
                let taplo_alphabetize = taplo_fmt::Options {
                    reorder_keys: true,
                    ..Default::default()
                };
                let formatted_content = taplo_fmt::format(&file_content, taplo_alphabetize);
                if !app.check {
                    format_file(&manifest_file, &formatted_content)?;
                } else if formatted_content != file_content {
                    contains_edits = true;
                    eprintln!("\nManifest Forc.toml improperly formatted");
                    display_file_diff(&file_content, &formatted_content)?;
                } else {
                    println!("\nManifest Forc.toml properly formatted")
                }
            }

            if app.check {
                if contains_edits {
                    // One or more files are not formatted, exit with error
                    bail!("Files contain formatting violations.");
                } else {
                    // All files are formatted, exit cleanly
                    Ok(())
                }
            } else {
                Ok(())
            }
        }
        _ => bail!("Manifest file does not exist"),
    }
}

fn display_file_diff(file_content: &str, formatted_content: &str) -> Result<()> {
    let changeset = diff_lines(file_content, formatted_content);
    let mut count_of_updates = 0;
    for diff in changeset.diff() {
        // max 100 updates
        if count_of_updates >= 100 {
            break;
        }
        match diff {
            DiffOp::Equal(old) => {
                for o in old {
                    println!("{}", o)
                }
            }
            DiffOp::Insert(new) => {
                count_of_updates += 1;
                for n in new {
                    println_green(&format!("+{}", n));
                }
            }
            DiffOp::Remove(old) => {
                count_of_updates += 1;
                for o in old {
                    println_red(&format!("-{}", o));
                }
            }
            DiffOp::Replace(old, new) => {
                count_of_updates += 1;
                for o in old {
                    println_red(&format!("-{}", o));
                }
                for n in new {
                    println_green(&format!("+{}", n));
                }
            }
        }
    }
    Result::Ok(())
}

fn format_file(file: &Path, formatted_content: &str) -> Result<()> {
    fs::write(file, formatted_content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::taplo_fmt;
    use std::default::Default;

    #[test]
    fn test_forc_indentation() {
        let correct_forc_manifest = r#"
[project]
authors = ["Fuel Labs <contact@fuel.sh>"]
license = "Apache-2.0"
name = "Fuel example project"


[dependencies]
core = { git = "https://github.com/FuelLabs/sway-lib-core", version = "v0.0.1" }
std = { git = "https://github.com/FuelLabs/sway-lib-std", version = "v0.0.1" }
"#;
        let taplo_alphabetize = taplo_fmt::Options {
            reorder_keys: true,
            ..Default::default()
        };
        let formatted_content = taplo_fmt::format(correct_forc_manifest, taplo_alphabetize.clone());
        assert_eq!(formatted_content, correct_forc_manifest);
        let indented_forc_manifest = r#"
        [project]
    authors = ["Fuel Labs <contact@fuel.sh>"]
                    license = "Apache-2.0"
    name = "Fuel example project"


    [dependencies]
        core = { git = "https://github.com/FuelLabs/sway-lib-core", version = "v0.0.1" }
                    std = { git = "https://github.com/FuelLabs/sway-lib-std", version = "v0.0.1" }
"#;
        let formatted_content =
            taplo_fmt::format(indented_forc_manifest, taplo_alphabetize.clone());
        assert_eq!(formatted_content, correct_forc_manifest);
        let whitespace_forc_manifest = r#"
[project]
 authors=["Fuel Labs <contact@fuel.sh>"]
license   =                                   "Apache-2.0"
name = "Fuel example project"


[dependencies]
core = {git="https://github.com/FuelLabs/sway-lib-core",version="v0.0.1"}
std         =     {   git     =  "https://github.com/FuelLabs/sway-lib-std"  , version = "v0.0.1"           }
"#;
        let formatted_content = taplo_fmt::format(whitespace_forc_manifest, taplo_alphabetize);
        assert_eq!(formatted_content, correct_forc_manifest);
    }

    #[test]
    fn test_forc_alphabetization() {
        let correct_forc_manifest = r#"
[project]
authors = ["Fuel Labs <contact@fuel.sh>"]
license = "Apache-2.0"
name = "Fuel example project"


[dependencies]
core = { git = "https://github.com/FuelLabs/sway-lib-core", version = "v0.0.1" }
std = { git = "https://github.com/FuelLabs/sway-lib-std", version = "v0.0.1" }
"#;
        let taplo_alphabetize = taplo_fmt::Options {
            reorder_keys: true,
            ..Default::default()
        };
        let formatted_content = taplo_fmt::format(correct_forc_manifest, taplo_alphabetize.clone());
        assert_eq!(formatted_content, correct_forc_manifest);
        let disordered_forc_manifest = r#"
[project]
name = "Fuel example project"
license = "Apache-2.0"
authors = ["Fuel Labs <contact@fuel.sh>"]


[dependencies]
std = { git = "https://github.com/FuelLabs/sway-lib-std", version = "v0.0.1" }
core = { git = "https://github.com/FuelLabs/sway-lib-core", version = "v0.0.1" }
    "#;
        let formatted_content = taplo_fmt::format(disordered_forc_manifest, taplo_alphabetize);
        assert_eq!(formatted_content, correct_forc_manifest);
    }
}
