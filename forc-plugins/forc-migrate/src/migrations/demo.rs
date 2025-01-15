//! This module contains demo migrations used for learning and testing the migration tool.

use std::vec;

use crate::migrations::{visit_lexed_modules_mut, MutProgramInfo};
use anyhow::{Ok, Result};
use sway_ast::{
    attribute::Annotated,
    keywords::{FnToken, Keyword},
    Braces, CodeBlockContents, FnSignature, ItemFn, Module, Parens, Punctuated,
};
use sway_core::Engines;
use sway_types::{Ident, Span, Spanned};

use super::{DryRun, MigrationStep, MigrationStepKind};

#[allow(dead_code)]
pub(super) const INSERT_EMPTY_FUNCTION_STEP: MigrationStep = MigrationStep {
    title: "Insert `empty_function` at the end of every module",
    duration: 0,
    kind: MigrationStepKind::CodeModification(insert_empty_function_step, &[]),
    help: &[
        "Migration will insert an empty function named `empty_function`",
        "at the end of every module, unless the function with the same",
        "name already exists in the module.",
        "E.g., `fn empty_function() {}`.",
    ],
};

fn insert_empty_function_step(
    program_info: &mut MutProgramInfo,
    dry_run: DryRun,
) -> Result<Vec<Span>> {
    fn insert_empty_function_step_impl(
        _engines: &Engines,
        module: &mut Module,
        dry_run: DryRun,
    ) -> Result<Vec<Span>> {
        // TODO: Simplify this demo migration by using matchers and modifiers.
        let mut result = vec![];

        // Code transformations must be idempotent. In this demo, if the function
        // with the name `empty_function` already exists, we do not insert it.
        let existing_empty_function = module
            .items
            .iter()
            .map(|annotated| &annotated.value)
            .filter_map(|decl| match decl {
                sway_ast::ItemKind::Fn(module_fn) => Some(module_fn),
                _ => None,
            })
            .find(|module_fn| module_fn.fn_signature.name.as_str() == "empty_function");

        if existing_empty_function.is_some() {
            return Ok(result);
        }

        // If the module is empty, insert right after the module kind,
        // otherwise, after the last item.
        let result_span = match module.items.last() {
            Some(annotated_item) => annotated_item.span(),
            None => module.semicolon_token.span(),
        };

        result.push(result_span.clone());

        if matches!(dry_run, DryRun::Yes) {
            return Ok(result);
        }

        // Not a dry-run, proceed with the code change.

        let insert_span = Span::empty_at_end(&result_span);

        // Construct the `empty_function`.
        // Note that we are using the `insert_span` for all the required spans.
        let empty_function = sway_ast::ItemKind::Fn(ItemFn {
            fn_signature: FnSignature {
                visibility: None,
                fn_token: FnToken::new(insert_span.clone()),
                name: Ident::new_with_override("empty_function".into(), insert_span.clone()),
                generics: None,
                arguments: Parens {
                    inner: sway_ast::FnArgs::Static(Punctuated {
                        value_separator_pairs: vec![],
                        final_value_opt: None,
                    }),
                    span: insert_span.clone(),
                },
                return_type_opt: None,
                where_clause_opt: None,
            },
            body: Braces {
                inner: CodeBlockContents {
                    statements: vec![],
                    final_expr_opt: None,
                    span: insert_span.clone(),
                },
                span: insert_span,
            },
        });

        // Add the constructed `empty_function` to the module items.
        module.items.push(Annotated {
            attribute_list: vec![],
            value: empty_function,
        });

        Ok(result)
    }

    let res = visit_lexed_modules_mut(
        program_info.engines,
        program_info.lexed_program,
        dry_run,
        insert_empty_function_step_impl,
    )?;

    Ok(res.into_iter().flatten().collect())
}
