use std::path::PathBuf;

use anyhow::{bail, Ok, Result};
use clap::Parser;
use forc_pkg::{self as pkg, PackageManifestFile};
use forc_pkg::{
    manifest::{GenericManifestFile, ManifestFile},
    source::IPFSNode,
};
use forc_tracing::println_action_green;
use sway_core::{BuildTarget, Engines};
use sway_error::diagnostic::*;
use sway_features::{ExperimentalFeatures, Feature};
use sway_types::{SourceEngine, Span};

use crate::migrations::{MigrationStepKind, MigrationStepsWithOccurrences};
use crate::{
    instructive_error,
    migrations::{MigrationStep, MigrationStepExecution, ProgramInfo},
};

/// Args that can be shared between all commands that `compile` a package. E.g. `check`, `run`.
#[derive(Debug, Default, Parser)]
pub(crate) struct Compile {
    /// Path to the project.
    ///
    /// If not specified, current working directory will be used.
    #[clap(short, long)]
    pub path: Option<String>,
    /// Offline mode, prevents Forc from using the network when managing dependencies.
    /// Meaning it will only try to use previously downloaded dependencies.
    #[clap(long = "offline")]
    pub offline: bool,
    /// Requires that the Forc.lock file is up-to-date. If the lock file is missing, or it
    /// needs to be updated, Forc will exit with an error.
    #[clap(long)]
    pub locked: bool,
    /// The IPFS Node to use for fetching IPFS sources.
    ///
    /// Possible values: FUEL, PUBLIC, LOCAL, <GATEWAY_URL>
    #[clap(long)]
    pub ipfs_node: Option<IPFSNode>,
    #[clap(flatten)]
    pub experimental: sway_features::CliFields,
    /// Silent mode. Don't output any warnings or errors to the command line.
    #[clap(long = "silent", short = 's')]
    pub silent: bool,
}

impl Compile {
    /// Returns the [Compile::path] if provided, otherwise the current directory.
    pub(crate) fn manifest_dir(&self) -> std::io::Result<PathBuf> {
        if let Some(path) = &self.path {
            std::result::Result::Ok(PathBuf::from(path))
        } else {
            std::env::current_dir()
        }
    }

    /// Returns the cumulative [ExperimentalFeatures], from the package manifest
    /// file and the CLI experimental flag.
    pub(crate) fn experimental_features(&self) -> Result<ExperimentalFeatures> {
        let manifest = ManifestFile::from_dir(self.manifest_dir()?)?;
        let pkg_manifest = get_pkg_manifest_file(&manifest)?;

        Ok(ExperimentalFeatures::new(
            &pkg_manifest.project.experimental,
            &self.experimental.experimental,
            &self.experimental.no_experimental,
        )
        .map_err(|err| anyhow::anyhow!("{err}"))?)
    }
}

// Clippy issue. It erroneously assumes that `vec!`s in `instructive_error` calls are not needed.
#[allow(clippy::useless_vec)]
fn get_pkg_manifest_file(manifest: &ManifestFile) -> Result<&PackageManifestFile> {
    match manifest {
        ManifestFile::Package(pkg_manifest) => Ok(pkg_manifest),
        ManifestFile::Workspace(_) => Err(anyhow::anyhow!(instructive_error(
            "`forc migrate` does not support migrating workspaces.",
            &vec![
                &format!("\"{}\" is a workspace.", manifest.dir().to_string_lossy()),
                "Please migrate each workspace member individually.",
            ]
        ))),
    }
}

// Clippy issue. It erroneously assumes that `vec!`s in `instructive_error` calls are not needed.
#[allow(clippy::useless_vec)]
pub(crate) fn compile_package<'a>(
    engines: &'a Engines,
    build_instructions: &Compile,
) -> Result<ProgramInfo<'a>> {
    let manifest_dir = build_instructions.manifest_dir()?;
    let manifest = ManifestFile::from_dir(manifest_dir.clone())?;
    let pkg_manifest = get_pkg_manifest_file(&manifest)?;
    let pkg_name = pkg_manifest.project_name();

    println_action_green(
        "Compiling",
        &format!("{} ({})", pkg_name, manifest.dir().to_string_lossy()),
    );

    let member_manifests = manifest.member_manifests()?;
    let lock_path = manifest.lock_path()?;

    let ipfs_node = build_instructions.ipfs_node.clone().unwrap_or_default();
    let plan = pkg::BuildPlan::from_lock_and_manifests(
        &lock_path,
        &member_manifests,
        build_instructions.locked,
        build_instructions.offline,
        &ipfs_node,
    )?;

    let include_tests = true; // We want to migrate the tests as well.
    let mut compile_results = pkg::check(
        &plan,
        BuildTarget::default(),
        build_instructions.silent,
        None,
        include_tests,
        engines,
        None,
        &build_instructions.experimental.experimental,
        &build_instructions.experimental.no_experimental,
        sway_core::DbgGeneration::Full,
    )?;

    let Some(programs) =
        compile_results
            .pop()
            .and_then(|(programs, handler)| if handler.has_errors() { None } else { programs })
    else {
        bail!(instructive_compilation_error(
            &pkg_manifest.path().to_string_lossy()
        ));
    };

    let core::result::Result::Ok(ty_program) = programs.typed else {
        bail!(instructive_compilation_error(
            &pkg_manifest.path().to_string_lossy()
        ));
    };

    return Ok(ProgramInfo {
        pkg_name: pkg_name.to_string(),
        lexed_program: programs.lexed,
        ty_program,
        engines,
    });

    fn instructive_compilation_error(manifest_dir: &str) -> String {
        instructive_error("The Sway project cannot be compiled.", &vec![
            &format!("`forc migrate` could not compile the Sway project located at \"{manifest_dir}\"."),
            "To see the compilation errors, run `forc build` on the project.",
            "Did you maybe forget to specify experimental features?",
            "If the project uses experimental features, they need to be specified when running `forc migrate`.",
            "E.g.:",
            "    forc migrate run --experimental <feature_1>,<feature_2>",
        ])
    }
}

pub(crate) const PROJECT_IS_COMPATIBLE: &str =
    "Project is compatible with the next breaking change version of Sway";

pub(crate) fn print_features_and_migration_steps(
    features_and_migration_steps: MigrationStepsWithOccurrences,
) {
    let show_migration_effort = features_and_migration_steps
        .iter()
        .flat_map(|(_, steps)| steps.iter().map(|step| step.1))
        .all(|occurrences| occurrences.is_some());

    let mut total_migration_effort = 0;
    for (feature, migration_steps) in features_and_migration_steps {
        println!("{}", feature.name());
        for (migration_step, occurrence) in migration_steps.iter() {
            println!(
                "  {} {}",
                match migration_step.execution() {
                    MigrationStepExecution::Manual => "[M]",
                    MigrationStepExecution::Semiautomatic => "[S]",
                    MigrationStepExecution::Automatic => "[A]",
                },
                migration_step.title
            );

            if show_migration_effort {
                let count = occurrence
                    .expect("if the `show_migration_effort` is true, all occurrences are `Some`");
                // For automatic steps **that have occurrences**, plan ~10 minutes
                // for the review of the automatically changed code.
                let migration_effort_in_mins = if migration_step.duration == 0 && count > 0 {
                    10
                } else {
                    // Otherwise, a very simple linear calculation will give
                    // a decent and useful rough estimate.
                    count * migration_step.duration
                };
                println!(
                    "      Occurrences: {count:>5}    Migration effort (hh::mm): ~{}\n",
                    duration_to_str(migration_effort_in_mins)
                );
                total_migration_effort += migration_effort_in_mins;
            }
        }

        if !show_migration_effort {
            println!();
        }
    }

    if show_migration_effort {
        println!(
            "Total migration effort (hh::mm): ~{}",
            duration_to_str(total_migration_effort)
        );

        // If there are no occurrences in code that require migration,
        // inform that the project is compatible with the next breaking change version of Sway.
        let num_of_occurrences = features_and_migration_steps
            .iter()
            .flat_map(|(_, steps)| steps.iter().map(|step| step.1.unwrap_or(0)))
            .sum::<usize>();
        if num_of_occurrences == 0 {
            println!();
            println!("{PROJECT_IS_COMPATIBLE}.");
        }
    }
}

/// Creates a single migration [Diagnostic] that shows **all the occurrences** in code
/// that require migration effort expected by the `migration_step`.
///
/// Returns `None` if the migration step is not necessary, in other words, if there
/// are no occurrences in code that require this particular migration.
pub(crate) fn create_migration_diagnostic(
    source_engine: &SourceEngine,
    feature: &Feature,
    migration_step: &MigrationStep,
    occurrences_spans: &[Span],
) -> Option<Diagnostic> {
    if occurrences_spans.is_empty() {
        return None;
    }

    let description = format!("[{}] {}", feature.name(), migration_step.title);
    Some(Diagnostic {
        reason: Some(Reason::new(Code::migrations(1), description)),
        issue: Issue::info(source_engine, occurrences_spans[0].clone(), "".into()),
        hints: occurrences_spans
            .iter()
            .skip(1)
            .map(|span| Hint::info(source_engine, span.clone(), "".into()))
            .collect(),
        help: migration_step
            .help
            .iter()
            .map(|help| help.to_string())
            .chain(if migration_step.help.is_empty() {
                vec![]
            } else {
                vec![Diagnostic::help_empty_line()]
            })
            .chain(match migration_step.kind {
                MigrationStepKind::Instruction(_) => vec![],
                MigrationStepKind::CodeModification(_, [], _) => vec![],
                MigrationStepKind::CodeModification(_, manual_migration_actions, _) => {
                    get_manual_migration_actions_help(manual_migration_actions)
                }
                MigrationStepKind::Interaction(_, _, [], _) => vec![
                    "This migration step will interactively modify the code, based on your input."
                        .to_string(),
                    Diagnostic::help_empty_line(),
                ],
                MigrationStepKind::Interaction(_, _, manual_migration_actions, _) => vec![
                    "This migration step will interactively modify the code, based on your input."
                        .to_string(),
                    Diagnostic::help_empty_line(),
                ]
                .into_iter()
                .chain(get_manual_migration_actions_help(manual_migration_actions))
                .collect(),
            })
            .chain(vec![detailed_migration_guide_msg(feature)])
            .collect(),
    })
}

fn get_manual_migration_actions_help(manual_migration_actions: &[&str]) -> Vec<String> {
    ["After the migration, you will still need to:".to_string()]
        .into_iter()
        .chain(
            manual_migration_actions
                .iter()
                .map(|help| format!("- {help}"))
                .chain(vec![Diagnostic::help_empty_line()]),
        )
        .collect()
}

pub(crate) fn detailed_migration_guide_msg(feature: &Feature) -> String {
    format!("For a detailed migration guide see: {}", feature.url())
}

fn duration_to_str(duration_in_mins: usize) -> String {
    let hours = duration_in_mins / 60;
    let minutes = duration_in_mins % 60;

    format!("{hours:#02}:{minutes:#02}")
}

pub(crate) fn max_feature_name_len<T>(features: &[(Feature, T)]) -> usize {
    features
        .iter()
        .map(|(feature, _)| feature.name().len())
        .max()
        .unwrap_or_default()
}
