//! This module contains common API for defining and implementing individual
//! [MigrationStep]s.
//!
//! Migration steps are defined in the submodules. Every submodule has the name
//! of the corresponding breaking change Sway feature and contains all the
//! migration steps needed to migrate to that feature.
//!
//! The special [demo] submodule contains demo migrations used for learning and testing
//! the migration tool.

mod demo;
mod error_type;
mod merge_core_std;
mod new_hashing;
mod partial_eq;
mod references;
mod storage_domains;
mod try_from_bytes_for_b256;
mod str_array_layout;

use std::{collections::HashSet, sync::Arc};

use anyhow::{bail, Result};
use duplicate::duplicate_item;
use itertools::Itertools;
use sway_ast::Module;
use sway_core::{
    language::{
        lexed::{LexedModule, LexedProgram},
        ty::{TyModule, TyProgram},
    },
    Engines,
};
use sway_features::Feature;
use sway_types::Span;

use crate::internal_error;

pub(crate) struct ProgramInfo<'a> {
    /// The name of the current package being migrated.
    pub pkg_name: String,
    pub lexed_program: Arc<LexedProgram>,
    pub ty_program: Arc<TyProgram>,
    pub engines: &'a Engines,
}

/// Wrapper over [ProgramInfo] that provides write access
/// to the [LexedProgram], but only read access to the
/// [TyProgram] and the [Engines]. It is used in migrations
/// that modify the source code by altering the lexed program.
pub(crate) struct MutProgramInfo<'a> {
    /// The name of the current package being migrated.
    pub pkg_name: &'a str,
    pub lexed_program: &'a mut LexedProgram,
    pub ty_program: &'a TyProgram,
    pub engines: &'a Engines,
}

impl ProgramInfo<'_> {
    pub(crate) fn as_mut(&mut self) -> MutProgramInfo {
        MutProgramInfo {
            pkg_name: &self.pkg_name,
            // Because the `ProgramsCacheEntry` clones the `programs`, the compilation will always
            // result in two strong `Arc` references to the `lexed_program`.
            // Therefore, we must use `Arc::make_mut` to get the copy-on-write behavior.
            lexed_program: Arc::make_mut(&mut self.lexed_program),
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
    /// If the `kind` is a [MigrationStepKind::CodeModification], start the help
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
            MigrationStepKind::CodeModification(_, manual_migration_actions, _)
                if !manual_migration_actions.is_empty() =>
            {
                Semiautomatic
            }
            MigrationStepKind::CodeModification(..) => Automatic,
            MigrationStepKind::Interaction(..) => Semiautomatic,
        }
    }

    pub(crate) fn has_manual_actions(&self) -> bool {
        match self.kind {
            MigrationStepKind::Instruction(_) => true,
            MigrationStepKind::CodeModification(_, [], _) => false,
            MigrationStepKind::CodeModification(_, _, _) => true,
            MigrationStepKind::Interaction(_, _, [], _) => false,
            MigrationStepKind::Interaction(_, _, _, _) => true,
        }
    }
}

/// Denotes that a migration step that changes the source code should
/// be executed in a dry-run mode, means just returning the places in code
/// to be changed, but without performing the actual change.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum DryRun {
    Yes,
    No,
}

/// Developer's response during an interactive migration step.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum InteractionResponse {
    /// There was no interaction with the developer.
    None,
    /// Developer opted for executing the migration step and change the code.
    ExecuteStep,
    /// Developer communicated that the code change is not needed.
    StepNotNeeded,
    /// Developer opted for postponing the migration step.
    PostponeStep,
}

/// A single occurrence of a [MigrationStep] report.
pub(crate) struct Occurrence {
    /// The [Span] of the occurrence in the original source code.
    pub span: Span,
    /// An optional help message that provides additional
    /// information about the occurrence.
    ///
    /// For most of migration steps, this will be `None`.
    /// Use it only if it brings valuable additional information
    /// about the particular [Occurrence].
    pub msg: Option<String>,
}

impl Occurrence {
    pub fn new(span: Span, msg: String) -> Self {
        Occurrence {
            span,
            msg: Some(msg),
        }
    }

    pub fn msg_or_empty(&self) -> String {
        self.msg.clone().unwrap_or_default()
    }
}

impl From<Span> for Occurrence {
    fn from(span: Span) -> Self {
        Occurrence { span, msg: None }
    }
}

/// A function that analyses a program given by the [ProgramInfo] and returns
/// the [Occurrence]s of all the places in the program code that need to be addressed
/// during a manual migration step.
///
/// The function does not modify the original program, and can use either the
/// [ProgramInfo::lexed_program] or the [ProgramInfo::ty_program], or both,
/// to perform the analysis.
type InstructionFn = for<'a> fn(&'a ProgramInfo<'a>) -> Result<Vec<Occurrence>>;

/// A function that analyses a program given by the [MutProgramInfo] and returns
/// the [Occurrence]s of all the places in the **original** program code that will be changed
/// during an automatic or semiautomatic migration step.
///
/// The function modifies the [LexedProgram] to perform the required code change,
/// unless the [DryRun] parameter is set to [DryRun::Yes].
type CodeModificationFn = for<'a> fn(&'a mut MutProgramInfo<'a>, DryRun) -> Result<Vec<Occurrence>>;

/// A function that interacts with the developer, eventually modifying the original
/// program given by [MutProgramInfo]. The developer's input decides if the modification
/// will happen or not.
///
/// Returns the [Occurrence]s of all the places in the **original** program code that are
/// changed during the interaction, if any, together with the developer's [InteractionResponse].
type InteractionFn =
    for<'a> fn(&'a mut MutProgramInfo<'a>) -> Result<(InteractionResponse, Vec<Occurrence>)>;

/// A function that visits the [Module] and its corresponding [TyModule],
/// potentially alters the lexed module, and returns a
/// [Result] containing related information about the visited module.
///
/// For its usages, see [visit_modules_mut].
type ModuleVisitorMutFn<T> =
    for<'a> fn(&'a Engines, &'a mut Module, &'a TyModule, DryRun) -> Result<T>;

/// A function that visits the [Module] and its corresponding [TyModule],
/// and returns a [Result] containing related information about the visited module.
///
/// For its usages, see [visit_modules].
type ModuleVisitorFn<T> = for<'a> fn(&'a Engines, &'a Module, &'a TyModule, DryRun) -> Result<T>;

/// Defines if the migration process can continue after a code modification
/// migration step.
#[derive(PartialEq, Eq, Clone, Copy)]
pub(crate) enum ContinueMigrationProcess {
    /// Continue if the step has no manual migration actions specified.
    /// This is the default and most common option.
    IfNoManualMigrationActionsNeeded,
    /// Always stop the migration. This is usually needed only after the
    /// steps that represent intermediate migration to an experimental
    /// feature for the purpose of early adoption.
    ///
    /// E.g., such step will keep the original code marked with
    /// experimental feature set to false, and insert the new implementation
    /// marked with experimental feature set to true.
    ///
    /// Continuing migration after such a step would be confusing,
    /// because the next step would usually offer immediate removal of the
    /// changes done in the step.
    Never,
}

pub(crate) enum MigrationStepKind {
    /// A migration step that provides instructions to developers,
    /// and explains a manual action they should take.
    Instruction(InstructionFn),
    /// A migration step that automatically modifies the original source code,
    /// and eventually gives additional instructions to developers,
    /// for manual post-migration actions.
    ///
    /// The [CodeModificationFn] modifies and overwrites the original source code.
    /// The second parameter are the _manual migration actions_.
    /// Those actions need to be done by developers after the automatic part
    /// of the migration is executed.
    ///
    /// Manual migration actions start with a small letter and end with a dot.
    ///
    /// E.g.: change function callers, by adding `&mut` to passed parameters.
    ///
    /// **If a [MigrationStepKind::CodeModification] does not have
    /// _manual migration actions_ it is considered to be a fully automated migration,
    /// after witch the migration process can safely continue, unless marked as
    /// [ContinueMigrationProcess::Never].**
    CodeModification(
        CodeModificationFn,
        &'static [&'static str],
        ContinueMigrationProcess,
    ),
    /// A migration step that first provides instructions to developers,
    /// and afterwards interacts with them, giving additional instructions
    /// and asking for additional input.
    ///
    /// Based on the input gotten during the interaction, the [InteractionFn]
    /// can modify the original source code.
    ///
    /// The second parameter are the _manual migration actions_.
    /// Those actions still need to be done by developers after the automatic part
    /// of the migration is executed during the interaction.
    ///
    /// Manual migration actions start with a small letter and end with a dot.
    ///
    /// E.g.: change function callers, by adding `&mut` to passed parameters.
    ///
    /// **If a [MigrationStepKind::Interaction] does not have
    /// _manual migration actions_ it is considered to be finished after the interaction,
    /// after witch the migration process can safely continue, unless marked as
    /// [ContinueMigrationProcess::Never].**
    ///
    /// Note that in a general case, the [InstructionFn] and the [InteractionFn]
    /// can return different [Span]s. E.g., during the instruction a single
    /// span can be returned pointing to a module in which the change needs
    /// to be done, while the interaction will return the actual places in the
    /// module that were modified.
    Interaction(
        InstructionFn,
        InteractionFn,
        &'static [&'static str],
        ContinueMigrationProcess,
    ),
}

/// A convenient method for visiting all the modules within a program.
/// The `visitor` will be called for every module, and the method will return the
/// [Vec] containing the results of all the individual visitor calls.
#[deprecated(note = "use `crate::visiting::ProgramVisitor/Mut::visit_program()` instead")]
#[allow(deprecated)]
pub(crate) fn visit_all_modules<T>(
    program_info: &ProgramInfo,
    dry_run: DryRun,
    visitor: ModuleVisitorFn<T>,
) -> Result<Vec<T>> {
    visit_modules(
        program_info.engines,
        &program_info.lexed_program.root,
        &program_info.ty_program.root_module,
        dry_run,
        visitor,
    )
}

/// A convenient method for visiting all the modules within a program.
/// The `visitor` will be called for every module, and the method will return the
/// [Vec] containing the results of all the individual visitor calls.
///
/// Visitors can mutate the [LexedProgram].
#[deprecated(note = "use `crate::visiting::ProgramVisitor/Mut::visit_program()` instead")]
#[allow(deprecated)]
pub(crate) fn visit_all_modules_mut<T>(
    program_info: &mut MutProgramInfo,
    dry_run: DryRun,
    visitor: ModuleVisitorMutFn<T>,
) -> Result<Vec<T>> {
    visit_modules_mut(
        program_info.engines,
        &mut program_info.lexed_program.root,
        &program_info.ty_program.root_module,
        dry_run,
        visitor,
    )
}

/// A convenient method for visiting the `lexed_module` and its corresponding `ty_module`,
/// and all their submodules, recursively.
/// The `visitor` will be called for every module, and the method will return the
/// [Vec] containing the results of all the individual visitor calls.
#[duplicate_item(
    __visit_modules      __ModuleVisitorFn     __ref_type(type)  __ref(value)  __iter;
    [visit_modules]      [ModuleVisitorFn]     [&type]           [&value]      [iter];
    [visit_modules_mut]  [ModuleVisitorMutFn]  [&mut type]       [&mut value]  [iter_mut];
)]
#[deprecated(note = "use `crate::visiting::ProgramVisitor/Mut::visit_program()` instead")]
#[allow(deprecated)]
pub(crate) fn __visit_modules<T>(
    engines: &Engines,
    lexed_module: __ref_type([LexedModule]),
    ty_module: &TyModule,
    dry_run: DryRun,
    visitor: __ModuleVisitorFn<T>,
) -> Result<Vec<T>> {
    fn visit_modules_rec<T>(
        engines: &Engines,
        lexed_module: __ref_type([LexedModule]),
        ty_module: &TyModule,
        dry_run: DryRun,
        visitor: __ModuleVisitorFn<T>,
        result: &mut Vec<T>,
    ) -> Result<()> {
        let visitor_result = visitor(
            engines,
            __ref([lexed_module.tree.value]),
            ty_module,
            dry_run,
        )?;
        result.push(visitor_result);
        let mut lexed_submodules = lexed_module.submodules.__iter().collect_vec();
        let mut ty_submodules = ty_module.submodules.iter().collect_vec();

        if lexed_submodules.len() != ty_submodules.len() {
            bail!(internal_error(format!(
                "Lexed module has \"{}\" submodules, and typed module has \"{}\" submodules.",
                lexed_submodules.len(),
                ty_submodules.len(),
            )));
        }

        // The order of submodules is not guaranteed to be the same, hence, sorting by name to
        // ensure the same ordering.
        lexed_submodules.sort_by(|a, b| a.0.cmp(&b.0));
        ty_submodules.sort_by(|a, b| a.0.cmp(&b.0));

        let lexed_submodules = lexed_submodules.__iter();
        let ty_submodules = ty_submodules.iter();
        for (lexed_submodule, ty_submodule) in lexed_submodules.zip(ty_submodules) {
            if lexed_submodule.0 != ty_submodule.0 {
                bail!(internal_error(format!(
                    "Lexed module \"{}\" does not match with the typed module \"{}\".",
                    lexed_submodule.0, ty_submodule.0,
                )));
            }
            visit_modules_rec(
                engines,
                __ref([lexed_submodule.1.module]),
                &ty_submodule.1.module,
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
        lexed_module,
        ty_module,
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

/// The list of the migration steps, grouped by the Sway feature that causes
/// the breaking changes behind the migration steps.
const MIGRATION_STEPS: MigrationSteps = &[(
    Feature::NewHashing,
    &[new_hashing::REVIEW_EXISTING_USAGES_OF_STORAGE_MAP_SHA256_AND_KECCAK256],
), (
    Feature::StrArrayNoPadding,
    &[str_array_layout::REVIEW_EXISTING_USAGES_OF_STORAGE_STR_ARRAY],
)];
