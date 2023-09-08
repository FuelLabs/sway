//! A `forc` plugin for running the Sway code formatter.

use anyhow::{bail, Result};
use clap::Parser;
use forc_pkg::{manifest::ManifestFile, WorkspaceManifestFile};
use prettydiff::{basic::DiffOp, diff_lines};
use std::{
    default::Default,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use taplo::formatter as taplo_fmt;
use tracing::{debug, error, info};

use forc_tracing::{init_tracing_subscriber, println_error, println_green, println_red};
use forc_util::{find_parent_manifest_dir, is_sway_file};
use sway_core::{BuildConfig, BuildTarget};
use sway_utils::{constants, get_sway_files};
use swayfmt::Formatter;

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
    /// Formats a single .sw file with the default settings.
    /// If not specified, current working directory will be formatted using a Forc.toml configuration.
    pub file: Option<String>,
}

fn main() {
    init_tracing_subscriber(Default::default());
    if let Err(err) = run() {
        println_error("Formatting skipped due to error.");
        println_error(&format!("{}", err));
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let app = App::parse();

    let dir = match app.path.as_ref() {
        Some(path) => PathBuf::from(path),
        None => std::env::current_dir()?,
    };

    let mut formatter = Formatter::from_dir(&dir)?;
    if let Some(f) = app.file.as_ref() {
        let file_path = &PathBuf::from(f);

        // If we're formatting a single file, find the nearest manifest if within a project.
        // Otherwise, we simply provide 'None' to format_file().
        let manifest_file = find_parent_manifest_dir(file_path)
            .map(|path| path.join(constants::MANIFEST_FILE_NAME));

        if is_sway_file(file_path) {
            format_file(&app, file_path.to_path_buf(), manifest_file, &mut formatter)?;
            return Ok(());
        }

        bail!(
            "Provided file '{}' is not a valid Sway file",
            file_path.display()
        );
    };

    let manifest_file = forc_pkg::manifest::ManifestFile::from_dir(&dir)?;

    match manifest_file {
        ManifestFile::Workspace(ws) => {
            format_workspace_at_dir(&app, &ws, &dir)?;
        }
        ManifestFile::Package(_) => {
            format_pkg_at_dir(&app, &dir, &mut formatter)?;
        }
    }

    Ok(())
}

/// Recursively get a Vec<PathBuf> of subdirectories that contains a Forc.toml.
fn get_sway_dirs(workspace_dir: PathBuf) -> Vec<PathBuf> {
    let mut dirs_to_format = vec![];
    let mut dirs_to_search = vec![workspace_dir];

    while let Some(next_dir) = dirs_to_search.pop() {
        if let Ok(read_dir) = fs::read_dir(next_dir) {
            for entry in read_dir.filter_map(|res| res.ok()) {
                let path = entry.path();

                if path.is_dir() {
                    dirs_to_search.push(path.clone());
                    if path.join(constants::MANIFEST_FILE_NAME).exists() {
                        dirs_to_format.push(path);
                    }
                }
            }
        }
    }

    dirs_to_format
}

/// Format a file, given its path.
/// Returns:
/// - Ok(true) if executed successfully and formatted,
/// - Ok(false) if executed successfully and not formatted,
/// - Err if it fails to execute at all.
fn format_file(
    app: &App,
    file: PathBuf,
    manifest_file: Option<PathBuf>,
    formatter: &mut Formatter,
) -> Result<bool> {
    let file = file.canonicalize()?;
    if let Ok(file_content) = fs::read_to_string(&file) {
        let mut edited = false;
        let file_content: Arc<str> = Arc::from(file_content);
        let build_config = manifest_file.map(|f| {
            BuildConfig::root_from_file_name_and_manifest_path(
                file.clone(),
                f,
                BuildTarget::default(),
            )
        });
        match Formatter::format(formatter, file_content.clone(), build_config.as_ref()) {
            Ok(formatted_content) => {
                if app.check {
                    if *file_content != formatted_content {
                        info!("File was edited by formatter: \n{:?}\n", file);
                        display_file_diff(&file_content, &formatted_content)?;
                        edited = true;
                    }
                } else {
                    write_file_formatted(&file, &formatted_content)?;
                }

                return Ok(edited);
            }
            Err(err) => {
                // TODO: Support formatting for incomplete/invalid sway code.
                // https://github.com/FuelLabs/sway/issues/5012
                debug!("{}", err);
                if let Some(file) = file.to_str() {
                    bail!("Failed to compile {}\n{}", file, err);
                } else {
                    bail!("Failed to compile.\n{}", err);
                }
            }
        }
    }

    bail!("Could not read file: {:?}", file)
}

/// Format the workspace at the given directory.
fn format_workspace_at_dir(app: &App, workspace: &WorkspaceManifestFile, dir: &Path) -> Result<()> {
    let mut contains_edits = false;
    let mut formatter = Formatter::from_dir(dir)?;
    let mut members = vec![];

    for member_path in workspace.member_paths()? {
        members.push(member_path)
    }

    // Format files at the root - we do not want to start calling format_pkg_at_dir() here,
    // since this would mean we format twice on each subdirectory.
    if let Ok(read_dir) = fs::read_dir(dir) {
        for entry in read_dir.filter_map(|res| res.ok()) {
            let path = entry.path();
            if is_sway_file(&path) {
                format_file(
                    app,
                    path,
                    Some(workspace.dir().to_path_buf()),
                    &mut formatter,
                )?;
            }
        }
    }

    // Format subdirectories. We do not call format on members directly here, since
    // in workspaces, it is perfectly valid to have subdirectories containing Sway files,
    // yet not be a member of the workspace.
    for sub_dir in get_sway_dirs(dir.to_path_buf()) {
        if sub_dir.join(constants::MANIFEST_FILE_NAME).exists() {
            // Here, we cannot simply call Formatter::from_dir() and rely on defaults
            // if there is no swayfmt.toml in the sub directory because we still want
            // to use the swayfmt.toml at the workspace root (if any).
            // In order of priority: member > workspace > default.
            formatter = Formatter::from_dir(&sub_dir)?;
        }
        format_pkg_at_dir(app, &sub_dir, &mut formatter)?;
    }

    let manifest_file = dir.join(constants::MANIFEST_FILE_NAME);

    // Finally, format the root manifest using taplo formatter
    contains_edits |= format_manifest(app, manifest_file)?;

    if app.check && contains_edits {
        // One or more files are not formatted, exit with error
        bail!("Files contain formatting violations.");
    }

    Ok(())
}

/// Format the given manifest at a path.
/// Returns:
/// - Ok(true) if executed successfully and formatted,
/// - Ok(false) if executed successfully and not formatted,
/// - Err if it fails to execute at all.
fn format_manifest(app: &App, manifest_file: PathBuf) -> Result<bool> {
    if let Ok(manifest_content) = fs::read_to_string(&manifest_file) {
        let mut edited = false;
        let taplo_alphabetize = taplo_fmt::Options {
            reorder_keys: true,
            ..Default::default()
        };
        let formatted_content = taplo_fmt::format(&manifest_content, taplo_alphabetize);
        if !app.check {
            write_file_formatted(&manifest_file, &formatted_content)?;
        } else if formatted_content != manifest_content {
            edited = true;
            error!(
                "Improperly formatted manifest file: {}",
                manifest_file.display()
            );
            display_file_diff(&manifest_content, &formatted_content)?;
        } else {
            info!(
                "Manifest Forc.toml formatted correctly: {}",
                manifest_file.display()
            )
        }

        return Ok(edited);
    };

    bail!("failed to format manifest: {:?}", manifest_file)
}

/// Format the package at the given directory.
fn format_pkg_at_dir(app: &App, dir: &Path, formatter: &mut Formatter) -> Result<()> {
    match find_parent_manifest_dir(dir) {
        Some(path) => {
            let manifest_path = path.clone();
            let manifest_file = manifest_path.join(constants::MANIFEST_FILE_NAME);
            let files = get_sway_files(path);
            let mut contains_edits = false;

            for file in files {
                contains_edits |= format_file(app, file, Some(manifest_file.clone()), formatter)?;
            }
            // format manifest using taplo formatter
            contains_edits |= format_manifest(app, manifest_file)?;

            if app.check && contains_edits {
                // One or more files are not formatted, exit with error
                bail!("Files contain formatting violations.");
            }

            Ok(())
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
                    info!("{}", o)
                }
            }
            DiffOp::Insert(new) => {
                count_of_updates += 1;
                for n in new {
                    println_green(&format!("+{n}"));
                }
            }
            DiffOp::Remove(old) => {
                count_of_updates += 1;
                for o in old {
                    println_red(&format!("-{o}"));
                }
            }
            DiffOp::Replace(old, new) => {
                count_of_updates += 1;
                for o in old {
                    println_red(&format!("-{o}"));
                }
                for n in new {
                    println_green(&format!("+{n}"));
                }
            }
        }
    }
    Ok(())
}

fn write_file_formatted(file: &Path, formatted_content: &str) -> Result<()> {
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
