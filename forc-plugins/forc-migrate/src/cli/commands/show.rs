use std::collections::HashMap;

use crate::cli::shared::{max_feature_name_len, print_features_and_migration_steps};
use crate::get_migration_steps_or_return;
use crate::migrations::MigrationStepExecution;
use anyhow::{Ok, Result};
use clap::Parser;
use itertools::Itertools;
use sway_error::formatting::{sequence_to_list, sequence_to_str, Enclosing, Indent};

forc_types::cli_examples! {
    crate::cli::Opt {
        [ Show the upcoming breaking change features and their migration steps => "forc migrate show"]
    }
}

/// Show the upcoming breaking change features and their migration steps.
#[derive(Debug, Parser)]
pub(crate) struct Command {}

pub(crate) fn exec(_command: Command) -> Result<()> {
    let migration_steps = get_migration_steps_or_return!();

    let feature_name_len = max_feature_name_len(migration_steps);

    // Convert migration steps to form suitable for printing (adding `None` for time estimates.)
    let migration_steps = migration_steps
        .iter()
        .map(|(feature, steps)| {
            (
                *feature,
                steps.iter().map(|step| (step, None)).collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();

    // Print the list of breaking change features.
    println!("Breaking change features:");
    println!(
        "{}",
        sequence_to_list(
            &migration_steps
                .iter()
                .map(|(feature, _)| format!(
                    "{:feature_name_len$}    ({})",
                    feature.name(),
                    feature.url()
                ))
                .collect_vec(),
            Indent::Single,
            usize::MAX
        )
        .join("\n")
    );
    println!();

    // Print migration steps.
    let mut num_of_steps_per_execution_kind = HashMap::<MigrationStepExecution, usize>::new();
    migration_steps
        .iter()
        .flat_map(|(_, steps)| steps)
        .for_each(|(step, _)| {
            *num_of_steps_per_execution_kind
                .entry(step.execution())
                .or_insert(0) += 1
        });
    let num_of_steps_per_execution_kind = num_of_steps_per_execution_kind
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .sorted_by_key(|(execution, _)| *execution)
        .map(|(execution, count)| {
            format!(
                "{count} {}",
                match execution {
                    MigrationStepExecution::Manual => "manual",
                    MigrationStepExecution::Semiautomatic => "semiautomatic",
                    MigrationStepExecution::Automatic => "automatic",
                },
            )
        })
        .collect_vec();
    println!(
        "Migration steps ({}):",
        sequence_to_str(
            &num_of_steps_per_execution_kind,
            Enclosing::None,
            usize::MAX
        )
    );
    print_features_and_migration_steps(&migration_steps);

    // Print experimental feature flags.
    let features = migration_steps.iter().map(|(feature, _)| feature.name());

    println!("Experimental feature flags:");
    println!(
        "- for Forc.toml:  experimental = {{ {} }}",
        features
            .clone()
            .map(|feature| format!("{feature} = true"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "- for CLI:        --experimental {}",
        features.collect::<Vec<_>>().join(",")
    );

    Ok(())
}
