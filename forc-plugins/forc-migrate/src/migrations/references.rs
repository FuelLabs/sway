#![allow(deprecated)]

use std::vec;

use crate::migrations::{visit_all_modules_mut, MutProgramInfo, Occurrence};
use anyhow::{Ok, Result};
use itertools::Itertools;
use sway_ast::{
    keywords::{AmpersandToken, Keyword, MutToken, Token},
    Module,
};
use sway_core::{language::ty::TyModule, Engines};
use sway_types::{Span, Spanned};

use super::{ContinueMigrationProcess, DryRun, MigrationStep, MigrationStepKind};

#[allow(dead_code)]
pub(super) const REPLACE_REF_MUT_FN_PARAMETERS_STEP: MigrationStep = MigrationStep {
    title: "Replace `ref mut` function parameters with `&mut`",
    duration: 5,
    kind: MigrationStepKind::CodeModification(
        replace_ref_mut_fn_parameters_step,
        &[
            "change function callers, by adding `&mut` to passed parameters.",
            "change function bodies, by dereferencing (`*`) parameters where needed.",
        ],
        ContinueMigrationProcess::IfNoManualMigrationActionsNeeded,
    ),
    help: &[
        "Migration will replace `ref mut` function parameters with `&mut`.",
        "E.g., `ref mut x: u64` will become `x: &mut u64`.",
    ],
};

// TODO: This is an incomplete implementation of the migration step.
//       It does not search for all possible occurrences of `ref mut`.
//       It is provided as an example of how complex migrations that
//       transform code can be written. The complete implementation
//       will be provided by the time the "references" experimental
//       feature get out of the experimental phase.
//
//       Also, this migration step will be disabled for the next
//       breaking change version of Sway. It is currently enabled for
//       the sake of testing and trying out the `forc migrate` tool.

// TODO: Simplify this migration by using matchers and modifiers.
fn replace_ref_mut_fn_parameters_step(
    program_info: &mut MutProgramInfo,
    dry_run: DryRun,
) -> Result<Vec<Occurrence>> {
    fn replace_ref_mut_fn_parameters_step_impl(
        _engines: &Engines,
        module: &mut Module,
        _ty_module: &TyModule,
        dry_run: DryRun,
    ) -> Result<Vec<Occurrence>> {
        let mut result = vec![];

        // TODO: Current implementation inspects only module functions. Extend it
        //       to cover all functions (in traits, self-impls, trait-impls, etc.).

        for module_fn in module
            .items
            .iter_mut()
            .map(|annotated| &mut annotated.value)
            .filter_map(|decl| match decl {
                sway_ast::ItemKind::Fn(module_fn) => Some(module_fn),
                _ => None,
            })
        {
            let fn_args = &mut module_fn.fn_signature.arguments.inner;

            let fn_args = match fn_args {
                sway_ast::FnArgs::Static(punctuated) => punctuated,
                sway_ast::FnArgs::NonStatic { .. } => unreachable!(
                    "Module functions are always static and cannot have the `self` argument."
                ),
            };

            let mut fn_args = fn_args.iter_mut().collect_vec();

            if fn_args.is_empty() {
                continue;
            }

            for fn_arg in fn_args.iter_mut() {
                match &mut fn_arg.pattern {
                    sway_ast::Pattern::Var {
                        reference: ref_opt @ Some(_),
                        mutable: mut_opt @ Some(_),
                        name,
                    } => {
                        // Note that we cannot bind is `Some`s, because we would be mutually borrowing twice,
                        // once in, e.g., `ref_opt` and once in `Some` for its part.
                        // That's why, unfortunately, the `expect`.
                        let result_span = Span::join(
                            ref_opt
                                .as_ref()
                                .expect("`ref_opt` is `Some` in the match arm pattern")
                                .span(),
                            &name.span(),
                        );
                        result.push(result_span.into());

                        // Replace `ref mut` with `&mut` if it is not a dry-run.
                        if dry_run == DryRun::No {
                            *ref_opt = None;
                            *mut_opt = None;

                            // We will insert the `&` and `mut` tokens right before the existing argument type.
                            let insert_span = Span::empty_at_start(&fn_arg.ty.span());

                            // Modify the original type to the reference to it.
                            fn_arg.ty = sway_ast::Ty::Ref {
                                ampersand_token: AmpersandToken::new(insert_span.clone()),
                                mut_token: Some(MutToken::new(insert_span)),
                                ty: Box::new(fn_arg.ty.clone()),
                            };
                        }

                        // TODO: Find the usages of the function and add `&mut` to the passed parameters.

                        // TODO: Dereference the parameters in the function body.
                    }
                    _ => continue,
                }
            }
        }

        Ok(result)
    }

    let res = visit_all_modules_mut(
        program_info,
        dry_run,
        replace_ref_mut_fn_parameters_step_impl,
    )?;

    Ok(res.into_iter().flatten().collect())
}
