use clap::Parser;

use crate::{
    cli::{
        self,
        shared::{
            compile_package, create_migration_diagnostic, print_features_and_migration_steps,
        },
    },
    get_migration_steps_or_return,
    migrations::{DryRun, MigrationStepKind},
};
use anyhow::{Ok, Result};
use forc_diagnostic::format_diagnostic;
use itertools::Itertools;
use sway_core::Engines;

forc_types::cli_examples! {
    crate::cli::Opt {
        [ Check the project in the current path => "forc migrate check"]
        [ Check the project located in another path => "forc migrate check --path {path}" ]
    }
}

/// Check the project for code that needs to be migrated.
///
/// Dry-runs the migration steps and prints places in code that need to be reviewed or changed.
#[derive(Debug, Parser)]
pub(crate) struct Command {
    #[clap(flatten)]
    pub check: cli::shared::Compile,
}

pub(crate) fn exec(command: Command) -> Result<()> {
    let migration_steps = get_migration_steps_or_return!();
    let engines = Engines::default();
    let build_instructions = command.check;

    let mut program_info = compile_package(&engines, &build_instructions)?;

    // Dry-run all the migration steps.
    let mut check_result = vec![];
    for (feature, migration_steps) in migration_steps.iter() {
        for migration_step in migration_steps.iter() {
            let migration_point_spans = match migration_step.kind {
                MigrationStepKind::Instruction(instruction) => instruction(&program_info)?,
                MigrationStepKind::CodeModification(modification, ..) => {
                    modification(&mut program_info.as_mut(), DryRun::Yes)?
                }
                MigrationStepKind::Interaction(instruction, ..) => instruction(&program_info)?,
            };

            check_result.push((feature, migration_step, migration_point_spans));
        }
    }

    // For every migration step, display the found occurrences in code that require migration effort, if any.
    for (feature, migration_step, occurrences_spans) in check_result.iter() {
        if let Some(diagnostic) =
            create_migration_diagnostic(engines.se(), feature, migration_step, occurrences_spans)
        {
            format_diagnostic(&diagnostic);
        }
    }

    // Display the summary of the migration effort.
    let features_and_migration_steps = check_result
        .iter()
        .chunk_by(|(feature, _, _)| feature)
        .into_iter()
        .map(|(key, chunk)| {
            (
                **key,
                chunk
                    .map(|(_, migration_step, migration_point_spans)| {
                        (*migration_step, Some(migration_point_spans.len()))
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();

    println!("Migration effort:");
    println!();
    print_features_and_migration_steps(&features_and_migration_steps);

    Ok(())
}
