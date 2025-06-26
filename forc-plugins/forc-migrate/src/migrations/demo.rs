//! This module contains demo migrations used for learning and testing the migration tool.

#![allow(deprecated)]

use std::vec;

use crate::{
    internal_error,
    matching::{lexed_match, lexed_match_mut, with_name, with_name_mut},
    migrations::{visit_all_modules_mut, MutProgramInfo, Occurrence},
    modifying::*,
};
use anyhow::{bail, Ok, Result};
use sway_ast::Module;
use sway_core::{language::ty::TyModule, Engines};
use sway_types::{Span, Spanned};

use super::{ContinueMigrationProcess, DryRun, MigrationStep, MigrationStepKind};

#[allow(dead_code)]
pub(super) const INSERT_EMPTY_FUNCTION_STEP: MigrationStep = MigrationStep {
    title: "Insert `empty_function` at the end of every module",
    duration: 0,
    kind: MigrationStepKind::CodeModification(
        insert_empty_function_step,
        &[],
        ContinueMigrationProcess::IfNoManualMigrationActionsNeeded,
    ),
    help: &[
        "Migration will insert an empty function named `empty_function` at the end of",
        "every module.",
        " ",
        "E.g., `fn empty_function() {}`.",
        " ",
        "If a function with that name already exists in the module, it will be",
        "renamed to `empty_function_old`, and a new one will be inserted.",
        " ",
        "If both functions already exist, the migration does not do anything.",
    ],
};

fn insert_empty_function_step(
    program_info: &mut MutProgramInfo,
    dry_run: DryRun,
) -> Result<Vec<Occurrence>> {
    fn insert_empty_function_step_impl(
        _engines: &Engines,
        module: &mut Module,
        _ty_module: &TyModule,
        dry_run: DryRun,
    ) -> Result<Vec<Occurrence>> {
        let mut result = vec![];

        let existing_empty_function =
            lexed_match::functions(module, with_name("empty_function")).next();
        let existing_empty_old_function =
            lexed_match::functions(module, with_name("empty_function_old")).next();

        // If the module is empty, in the report, point at the module kind
        // (`contract`, `script`, `predicate`, or `library`), otherwise,
        // point at the last item.
        let report_span = match module.items.last() {
            Some(annotated_item) => annotated_item.span(),
            None => module.semicolon_token.span(),
        };

        match (existing_empty_function, existing_empty_old_function) {
            (Some(_), Some(_)) => {
                // Code transformations must be idempotent. In this demo, if both functions
                // already exist, we don't do anything.
                return Ok(vec![]);
            }
            (Some(_), None) => {
                // `empty_function` exists, but old do not.
                // Rename the existing `empty_function` to `empty_function_old`, and insert a new `empty_function`.

                // We report the occurrence of the code relevant for migration...
                result.push(report_span.clone().into());

                // ...and proceed with the code change only if it is not a dry-run.
                if dry_run == DryRun::Yes {
                    return Ok(result);
                }

                let Some(existing_empty_function) =
                    lexed_match_mut::functions(module, with_name_mut("empty_function")).next()
                else {
                    bail!(internal_error("Existing `empty_function` cannot be found."));
                };

                modify(existing_empty_function).set_name("empty_function_old");

                let insert_span = Span::empty_at_end(&report_span);
                let empty_function = New::function(insert_span, "empty_function");
                modify(module).append_function(empty_function);
            }
            (None, _) => {
                // `empty_function` does not exist, create a new one.

                result.push(report_span.clone().into());

                if dry_run == DryRun::Yes {
                    return Ok(result);
                }

                let insert_span = Span::empty_at_end(&report_span);
                let empty_function = New::function(insert_span, "empty_function");
                modify(module).append_function(empty_function);
            }
        }

        Ok(result)
    }

    let res = visit_all_modules_mut(program_info, dry_run, insert_empty_function_step_impl)?;

    Ok(res.into_iter().flatten().collect())
}
