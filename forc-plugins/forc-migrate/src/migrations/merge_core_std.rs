#![allow(dead_code)]

use std::{sync::Arc, vec};

use crate::{
    migrations::{InteractionResponse, MutProgramInfo, Occurrence},
    print_single_choice_menu,
    visiting::{
        InvalidateTypedElement, LexedFnCallInfo, LexedFnCallInfoMut, ProgramVisitor,
        ProgramVisitorMut, TreesVisitor, TreesVisitorMut, VisitingContext,
    },
};
use anyhow::{Ok, Result};
use sway_ast::{Expr, ItemImpl, ItemUse, UseTree};
use sway_core::language::ty::{TyExpression, TyImplSelfOrTrait, TyUseStatement};
use sway_types::{Ident, Spanned};

use super::{ContinueMigrationProcess, DryRun, MigrationStep, MigrationStepKind, ProgramInfo};

pub(super) const REPLACE_CORE_WITH_STD_IN_PATHS: MigrationStep = MigrationStep {
    title: "Replace `core` with `std` in paths",
    duration: 1,
    kind: MigrationStepKind::Interaction(
        replace_core_with_std_in_paths_instruction,
        replace_core_with_std_in_paths_interaction,
        &[],
        ContinueMigrationProcess::IfNoManualMigrationActionsNeeded,
    ),
    help: &[
        "Migration will replace all occurrences of `core` with `std` in paths.",
        " ",
        "E.g.:",
        "  use core::ops::*;",
        "will become:",
        "  use std::ops::*;",
        " ",
        "Run this migration only if you are switching fully to the `merge_core_std` feature,",
        "and do not plan to use the old, separated, standard libraries anymore.",
    ],
};

fn replace_core_with_std_in_paths_instruction(
    program_info: &ProgramInfo,
) -> Result<Vec<Occurrence>> {
    struct Visitor;
    impl TreesVisitor<Occurrence> for Visitor {
        fn visit_use(
            &mut self,
            _ctx: &VisitingContext,
            lexed_use: &ItemUse,
            _ty_use: Option<&TyUseStatement>,
            output: &mut Vec<Occurrence>,
        ) -> Result<InvalidateTypedElement> {
            let path_prefix = match &lexed_use.tree {
                UseTree::Path { prefix, .. } => Some(prefix),
                _ => None,
            };

            let Some(path_prefix) = path_prefix else {
                return Ok(InvalidateTypedElement::No);
            };

            if lexed_use.root_import.is_none() && path_prefix.as_str() == "core" {
                output.push(path_prefix.span().into());
            }

            Ok(InvalidateTypedElement::No)
        }

        fn visit_impl(
            &mut self,
            _ctx: &VisitingContext,
            lexed_impl: &ItemImpl,
            _ty_impl: Option<Arc<TyImplSelfOrTrait>>,
            output: &mut Vec<Occurrence>,
        ) -> Result<InvalidateTypedElement> {
            let Some((trait_path, _for_token)) = &lexed_impl.trait_opt else {
                return Ok(InvalidateTypedElement::No);
            };

            if trait_path.root_opt.is_none() && trait_path.prefix.name.as_str() == "core" {
                output.push(trait_path.prefix.span().into());
            }

            Ok(InvalidateTypedElement::No)
        }

        fn visit_fn_call(
            &mut self,
            _ctx: &VisitingContext,
            lexed_fn_call: &Expr,
            _ty_fn_call: Option<&TyExpression>,
            output: &mut Vec<Occurrence>,
        ) -> Result<InvalidateTypedElement> {
            let lexed_fn_call_info = LexedFnCallInfo::new(lexed_fn_call)?;
            let fn_path = match lexed_fn_call_info.func {
                Expr::Path(path) => Some(path),
                _ => None,
            };

            let Some(fn_path) = fn_path else {
                return Ok(InvalidateTypedElement::No);
            };

            if fn_path.root_opt.is_none()
                && !fn_path.suffix.is_empty()
                && fn_path.prefix.name.as_str() == "core"
            {
                output.push(fn_path.prefix.span().into());
            }

            Ok(InvalidateTypedElement::No)
        }
    }

    ProgramVisitor::visit_program(program_info, DryRun::Yes, &mut Visitor {})
}

fn replace_core_with_std_in_paths_interaction(
    program_info: &mut MutProgramInfo,
) -> Result<(InteractionResponse, Vec<Occurrence>)> {
    println!("All the occurrences of `core` shown above will be replaced with `std`.");
    println!();
    println!("Do you want to replace those occurrences and switch fully to the `merge_core_std` feature?");
    println!();

    if print_single_choice_menu(&[
        "Yes, replace `core` with `std` and switch fully to the `merge_core_std` feature.",
        "No, continue using `core` and `std` as separated libraries.",
    ]) != 0
    {
        return Ok((InteractionResponse::PostponeStep, vec![]));
    }

    // Execute the migration step.
    struct Visitor;
    impl TreesVisitorMut<Occurrence> for Visitor {
        // In all of the cases, we keep the old span and just override the name.
        fn visit_use(
            &mut self,
            _ctx: &VisitingContext,
            lexed_use: &mut ItemUse,
            _ty_use: Option<&TyUseStatement>,
            output: &mut Vec<Occurrence>,
        ) -> Result<InvalidateTypedElement> {
            let path_prefix = match &mut lexed_use.tree {
                UseTree::Path { prefix, .. } => Some(prefix),
                _ => None,
            };

            let Some(path_prefix) = path_prefix else {
                return Ok(InvalidateTypedElement::No);
            };

            if lexed_use.root_import.is_none() && path_prefix.as_str() == "core" {
                output.push(path_prefix.span().into());

                *path_prefix = Ident::new_with_override("std".to_string(), path_prefix.span());
            }

            Ok(InvalidateTypedElement::Yes)
        }

        fn visit_impl(
            &mut self,
            _ctx: &VisitingContext,
            lexed_impl: &mut ItemImpl,
            _ty_impl: Option<Arc<TyImplSelfOrTrait>>,
            output: &mut Vec<Occurrence>,
        ) -> Result<InvalidateTypedElement> {
            let Some((trait_path, _for_token)) = &mut lexed_impl.trait_opt else {
                return Ok(InvalidateTypedElement::No);
            };

            if trait_path.root_opt.is_none() && trait_path.prefix.name.as_str() == "core" {
                output.push(trait_path.prefix.span().into());

                trait_path.prefix.name =
                    Ident::new_with_override("std".to_string(), trait_path.prefix.span());
            }

            Ok(InvalidateTypedElement::Yes)
        }

        fn visit_fn_call(
            &mut self,
            _ctx: &VisitingContext,
            lexed_fn_call: &mut Expr,
            _ty_fn_call: Option<&TyExpression>,
            output: &mut Vec<Occurrence>,
        ) -> Result<InvalidateTypedElement> {
            let lexed_fn_call_info = LexedFnCallInfoMut::new(lexed_fn_call)?;
            let fn_path = match lexed_fn_call_info.func {
                Expr::Path(path) => Some(path),
                _ => None,
            };

            let Some(fn_path) = fn_path else {
                return Ok(InvalidateTypedElement::No);
            };

            if fn_path.root_opt.is_none()
                && !fn_path.suffix.is_empty()
                && fn_path.prefix.name.as_str() == "core"
            {
                output.push(fn_path.prefix.span().into());

                fn_path.prefix.name =
                    Ident::new_with_override("std".to_string(), fn_path.prefix.span());
            }

            Ok(InvalidateTypedElement::Yes)
        }
    }

    ProgramVisitorMut::visit_program(program_info, DryRun::No, &mut Visitor {})
        .map(|result| (InteractionResponse::ExecuteStep, result))
}
