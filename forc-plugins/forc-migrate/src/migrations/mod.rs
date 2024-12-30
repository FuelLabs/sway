//! This module contains common API for defining and implementing individual
//! [MigrationStep]s.
//!
//! Migration steps are defined in the submodules. Every submodule has the name
//! of the corresponding breaking change Sway feature and contains all the
//! migration steps needed to migrate that feature.
//!
//! The special [demo] submodule contains demo migrations used for learning and testing
//! the migration tool.

mod demo;
mod references;
mod storage_domains;

use std::collections::HashSet;

use anyhow::Result;
use sway_ast::Module;
use sway_core::{
    language::{
        lexed::{LexedModule, LexedProgram},
        ty::TyProgram,
    },
    Engines,
};
use sway_features::Feature;
use sway_types::Span;

pub(crate) struct ProgramInfo<'a> {
    pub lexed_program: LexedProgram,
    pub ty_program: TyProgram,
    pub engines: &'a Engines,
}

/// Wrapper over [ProgramInfo] that provides write access
/// to the [LexedProgram], but only read access to the
/// [TyProgram] and the [Engines]. It is used in migrations
/// that transform the source code by altering the lexed
/// program.
pub(crate) struct MutProgramInfo<'a> {
    pub lexed_program: &'a mut LexedProgram,
    #[allow(dead_code)]
    pub ty_program: &'a TyProgram,
    pub engines: &'a Engines,
}

impl<'a> ProgramInfo<'a> {
    pub(crate) fn as_mut(&mut self) -> MutProgramInfo {
        MutProgramInfo {
            lexed_program: &mut self.lexed_program,
            ty_program: &self.ty_program,
            engines: self.engines,
        }
    }
}

/// A single migration step in the overall migration process.
pub(crate) struct MigrationStep {
    /// Migration step unique title.
    ///
    /// Formulated as a continuation of a suggestion to a developer: You should \<title\>.
    ///
    /// Titles are short, start with a capital letter and do not end in punctuation.
    ///
    /// E.g.: Replace `ref mut` function parameters with `&mut`
    ///
    /// In particular, titles of the manual migration steps start with "Review".
    pub title: &'static str,
    /// An estimated time (in minutes) needed for the manual part of migrating
    /// a single typical occurrence of the change represented by this step.
    ///
    /// The estimate includes **all** the manual effort.
    ///
    /// E.g., to replace a single `ref mut` function parameter with `&mut`, the migration
    /// will change the function signature. The manual part of the effort will be changing
    /// the callers and eventually adding dereferencing in the function body.
    ///
    /// Fully automated migration steps, and only them, can have `duration` set to zero.
    pub duration: usize,
    pub kind: MigrationStepKind,
    /// A short help for the migration step.
    ///
    /// If the `kind` is a [MigrationStepKind::CodeTransformation], start the help
    /// with "Migration will", to point out that the migration is a (semi)automatic one
    /// and causes changes in the source file.
    ///
    /// E.g.: Migration will replace `ref mut` function parameters with `&mut`.
    ///
    /// It is advisable to provide the short help, but it is not mandatory.
    /// Every migration step will have an automatic help line that points to
    /// the detailed migration guide provided in the feature tracking issue.
    pub help: &'static [&'static str],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub(crate) enum MigrationStepExecution {
    Manual,
    Semiautomatic,
    Automatic,
}

impl MigrationStep {
    pub(crate) fn execution(&self) -> MigrationStepExecution {
        use MigrationStepExecution::*;
        match self.kind {
            MigrationStepKind::Instruction(_) => Manual,
            MigrationStepKind::CodeTransformation(_, manual_migration_actions)
                if !manual_migration_actions.is_empty() =>
            {
                Semiautomatic
            }
            MigrationStepKind::CodeTransformation(_, _) => Automatic,
        }
    }

    pub(crate) fn has_manual_actions(&self) -> bool {
        match self.kind {
            MigrationStepKind::Instruction(_) => true,
            MigrationStepKind::CodeTransformation(_, []) => false,
            MigrationStepKind::CodeTransformation(_, _) => true,
        }
    }
}

/// Denotes that a migration step that changes the source code should
/// be executed in a dry-run mode, means just returning the places in code
/// to be changed, but without performing the actual change.
#[derive(Clone, Copy)]
pub(crate) enum DryRun {
    Yes,
    No,
}

/// A function that analyses a program given by the [ProgramInfo] and returns
/// the [Span]s of all the places in the program code that need to be addressed
/// during a manual migration step.
///
/// The function does not modify the original program, and can use either the
/// [ProgramInfo::lexed_program] or the [ProgramInfo::ty_program], or both,
/// to perform the analysis.
type InstructionFn = for<'a> fn(&'a ProgramInfo<'a>) -> Result<Vec<Span>>;

/// A function that analyses a program given by the [MutProgramInfo] and returns
/// the [Span]s of all the places in the **original** program code that will be changed
/// during an automatic or semiautomatic migration step.
///
/// The function modifies the [LexedProgram] to perform the required code change,
/// unless the [DryRun] parameter is set to [DryRun::Yes].
type CodeTransformationFn = for<'a> fn(&'a mut MutProgramInfo<'a>, DryRun) -> Result<Vec<Span>>;

/// A function that visits the [Module], potentially alters it, and returns a
/// [Result] containing related information about the [Module].
///
/// For its usages, see [visit_lexed_modules_mut].
type ModuleVisitorFn<T> = for<'a> fn(&'a Engines, &'a mut Module, DryRun) -> Result<T>;

pub(crate) enum MigrationStepKind {
    /// A migration step that provides instructions to developers,
    /// and explains a manual action they should take.
    Instruction(InstructionFn),
    /// A migration step that automatically transforms the original source code,
    /// and eventually gives additional instructions to developers,
    /// for manual post-migration actions.
    ///
    /// The [CodeTransformationFn] transforms and overwrites the original source code.
    /// The second parameter are the _manual migration actions_.
    /// Those actions need to be done by developers after the automatic part
    /// of the migration is executed.
    ///
    /// Manual migration actions start with a small letter and end with a dot.
    ///
    /// E.g.: change function callers, by adding `&mut` to passed parameters.
    ///
    /// **If a [MigrationStepKind::CodeTransformation] does not have
    /// _manual migration actions_ it is considered to be a fully automated migration,
    /// after witch the migration process can safely continue.**
    CodeTransformation(CodeTransformationFn, &'static [&'static str]),
}

/// A convenient method for visiting all the [LexedModule]s within a [LexedProgram].
/// The `visitor` will be called for every module, and the method will return the
/// [Vec] containing the results of all the visitor calls.
///
/// The `visitor` can mutate the modules.
pub(crate) fn visit_lexed_modules_mut<T>(
    engines: &Engines,
    lexed_program: &mut LexedProgram,
    dry_run: DryRun,
    visitor: ModuleVisitorFn<T>,
) -> Result<Vec<T>> {
    fn visit_modules_rec<T>(
        engines: &Engines,
        lexed_module: &mut LexedModule,
        dry_run: DryRun,
        visitor: ModuleVisitorFn<T>,
        result: &mut Vec<T>,
    ) -> Result<()> {
        let visitor_result = visitor(engines, &mut lexed_module.tree, dry_run)?;
        result.push(visitor_result);
        for (_, lexed_submodule) in lexed_module.submodules.iter_mut() {
            visit_modules_rec(
                engines,
                &mut lexed_submodule.module,
                dry_run,
                visitor,
                result,
            )?;
        }
        Ok(())
    }

    let mut result = vec![];
    visit_modules_rec(
        engines,
        &mut lexed_program.root,
        dry_run,
        visitor,
        &mut result,
    )?;
    Ok(result)
}

/// Registered [MigrationStep]s.
pub(crate) type MigrationSteps = &'static [(Feature, &'static [MigrationStep])];

/// Keeps the number of occurrences of each [MigrationStep]
/// after the analysis is executed.
pub(crate) type MigrationStepsWithOccurrences<'a> =
    &'a [(Feature, Vec<(&'a MigrationStep, Option<usize>)>)];

/// Returns a non-empty set of consistent migration steps.
///
/// All the CLI commands require at least one migration step.
/// This macro conveniently short-circuits and returns,
/// if there are no migration steps defined.
///
/// Panics if the migration steps are not consistent.
#[macro_export]
macro_rules! get_migration_steps_or_return {
    () => {{
        let migration_steps = $crate::migrations::get_migration_steps();

        if migration_steps.is_empty() {
            println!("There are currently no migration steps defined for the upcoming breaking change version of Sway.");
            return Ok(());
        }

        migration_steps
    }};
}

pub(crate) fn get_migration_steps() -> MigrationSteps {
    assert_migration_steps_consistency(MIGRATION_STEPS);
    MIGRATION_STEPS
}

/// Panics if the migration steps are not consistent.
fn assert_migration_steps_consistency(migration_steps: MigrationSteps) {
    if migration_steps.is_empty() {
        return;
    }

    // Each experimental feature can appear only once in the migration steps.
    let num_of_features_in_migration_steps = migration_steps.len();
    let num_of_unique_features_in_migration_steps = migration_steps
        .iter()
        .map(|(feature, _)| feature)
        .collect::<HashSet<_>>()
        .len();
    if num_of_features_in_migration_steps != num_of_unique_features_in_migration_steps {
        panic!("Inconsistent migration steps: each experimental feature can appear only once in the migration steps.");
    }

    // Migration step titles must be unique.
    let num_of_migration_steps = migration_steps
        .iter()
        .map(|(_, steps)| steps.len())
        .sum::<usize>();
    let num_of_migration_steps_with_unique_title = migration_steps
        .iter()
        .flat_map(|(_, steps)| steps.iter().map(|step| step.title))
        .collect::<HashSet<_>>()
        .len();
    if num_of_migration_steps != num_of_migration_steps_with_unique_title {
        panic!("Inconsistent migration steps: migration step titles must be unique.");
    }

    // Only fully automatic steps can have duration set to zero.
    let has_non_automatic_steps_with_zero_duration = migration_steps
        .iter()
        .flat_map(|(_, steps)| {
            steps.iter().map(|step| {
                (
                    matches!(step.execution(), MigrationStepExecution::Automatic),
                    step.duration,
                )
            })
        })
        .any(|(is_automatic, duration)| !is_automatic && duration == 0);
    if has_non_automatic_steps_with_zero_duration {
        panic!("Inconsistent migration steps: only fully automatic steps can have duration set to zero.");
    }
}

/*
   ------------------------------ Migration Steps -------------------------------
   Below are the actual migration steps. Change those steps for every new
   breaking change version of Sway, by removing the previous steps and adding the
   ones relevant for the next breaking change version.
*/

/// The list of the migration steps, grouped by the Sway features that cause
/// the breaking changes behind the migration steps.
const MIGRATION_STEPS: MigrationSteps = &[
    (
        Feature::StorageDomains,
        &[self::storage_domains::REVIEW_STORAGE_SLOT_KEYS_STEP],
    ),
    (
        Feature::References,
        &[self::references::REPLACE_REF_MUT_FN_PARAMETERS_STEP],
    ),
];
