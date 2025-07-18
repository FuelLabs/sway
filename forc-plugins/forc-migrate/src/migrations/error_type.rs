#![allow(deprecated)]

use std::{sync::Arc, vec};

use crate::{
    internal_error,
    migrations::{MutProgramInfo, Occurrence},
    modifying::*,
    visiting::*,
};
use anyhow::{bail, Ok, Result};
use sway_ast::{
    assignable::ElementAccess, expr, Assignable, Expr, ItemFn, ItemStruct, StatementLet,
};
use sway_core::language::{
    ty::{
        TyExpression, TyExpressionVariant, TyFunctionDecl, TyReassignmentTarget, TyStructDecl,
        TyVariableDecl,
    },
    CallPathType,
};
use sway_types::{Ident, Spanned};

use super::{ContinueMigrationProcess, DryRun, MigrationStep, MigrationStepKind};

// NOTE: We assume idiomatic usage of the identifier `panic`. This means we support
//       its migration only if it is used as a function name, struct field, or variable name.
//       E.g., renaming `panic` in `struct panic { ... }` is not supported,
//       as it is not an idiomatic usage.

// NOTE: We don't have infrastructure in place for searching for usages of a symbol.
//       Ideally, if we had it, we would use such infrastructure to rename symbol usages
//       when its definition get renamed.
//       Luckily, for this particular migration, it is sufficient to visit specific expression,
//       like, e.g., function calls, and rename them.

// NOTE: We don't support renaming modules named `panic`. The reason is that we have the `str`
//       module in the standard library, signaling that using keywords as module names is acceptable.

#[allow(dead_code)]
pub(super) const RENAME_EXISTING_PANIC_IDENTIFIERS_TO_R_PANIC_STEP: MigrationStep = MigrationStep {
    title: "Rename existing `panic` identifiers to `r#panic`",
    duration: 0,
    kind: MigrationStepKind::CodeModification(
        rename_existing_panic_identifiers_to_r_panic_step,
        &[],
        ContinueMigrationProcess::IfNoManualMigrationActionsNeeded,
    ),
    help: &[
        "Migration will rename existing `panic` identifiers in struct fields,",
        "function names and arguments, and variable names to `r#panic`.",
        " ",
        "E.g., `let panic = 42;` will become `let r#panic = 42;`.",
    ],
};

fn rename_existing_panic_identifiers_to_r_panic_step(
    program_info: &mut MutProgramInfo,
    dry_run: DryRun,
) -> Result<Vec<Occurrence>> {
    struct Visitor;
    impl TreesVisitorMut<Occurrence> for Visitor {
        fn visit_fn_decl(
            &mut self,
            ctx: &VisitingContext,
            lexed_fn: &mut ItemFn,
            _ty_fn: Option<Arc<TyFunctionDecl>>,
            output: &mut Vec<Occurrence>,
        ) -> Result<InvalidateTypedElement> {
            // First, let's check the arguments.
            for lexed_arg in lexed_fn.fn_signature.arguments.inner.args_mut() {
                let arg_name = match &mut lexed_arg.pattern {
                    sway_ast::Pattern::Var { name, .. } => name,
                    // A valid identifier in a function argument pattern can only be a variable,
                    // never an enum variant. So we know that this `ident` is a variable.
                    sway_ast::Pattern::AmbiguousSingleIdent(ident) => ident,
                    _ => continue,
                };

                if arg_name.as_raw_ident_str() != "panic" {
                    continue;
                }

                output.push(arg_name.span().into());

                if ctx.dry_run == DryRun::Yes {
                    continue;
                }

                *arg_name = Ident::new_with_raw(arg_name.span(), true);
            }

            // Then, the function name.
            if lexed_fn.fn_signature.name.as_raw_ident_str() != "panic" {
                return Ok(InvalidateTypedElement::No);
            }

            output.push(lexed_fn.fn_signature.name.span().into());

            if ctx.dry_run == DryRun::Yes {
                return Ok(InvalidateTypedElement::No);
            }

            modify(lexed_fn).set_name("r#panic");

            Ok(InvalidateTypedElement::No)
        }

        fn visit_struct_decl(
            &mut self,
            ctx: &VisitingContext,
            lexed_struct: &mut ItemStruct,
            _ty_struct: Option<Arc<TyStructDecl>>,
            output: &mut Vec<Occurrence>,
        ) -> Result<InvalidateTypedElement> {
            for lexed_field in lexed_struct.fields.inner.iter_mut() {
                let field_name = &mut lexed_field.value.name;

                if field_name.as_raw_ident_str() != "panic" {
                    continue;
                }

                output.push(field_name.span().into());

                if ctx.dry_run == DryRun::Yes {
                    continue;
                }

                *field_name = Ident::new_with_raw(field_name.span(), true);
            }

            Ok(InvalidateTypedElement::No)
        }

        fn visit_fn_call(
            &mut self,
            ctx: &VisitingContext,
            lexed_fn_call: &mut Expr,
            ty_fn_call: Option<&TyExpression>,
            output: &mut Vec<Occurrence>,
        ) -> Result<InvalidateTypedElement> {
            // We report the occurrences only if it is not a dry-run.
            if ctx.dry_run == DryRun::Yes {
                return Ok(InvalidateTypedElement::No);
            }

            let Some(ty_fn_call) = ty_fn_call else {
                // Without the typed function call, we cannot proceed
                // because we cannot check if the function is actually defined in the current package.
                return Ok(InvalidateTypedElement::No);
            };

            let Expr::FuncApp { func, args: _ } = lexed_fn_call else {
                bail!(internal_error("`lexed_fn_call` is not an `Expr::FuncApp`."));
            };

            let Expr::Path(path) = &mut **func else {
                // We are interested only in function calls that are paths.
                // Only such calls can be renamed.
                return Ok(InvalidateTypedElement::No);
            };

            let last_segment = path.last_segment_mut();

            if last_segment.name.as_raw_ident_str() != "panic" {
                return Ok(InvalidateTypedElement::No);
            }

            // Check if the function is actually defined in the current package.
            let TyExpressionVariant::FunctionApplication { fn_ref, .. } = &ty_fn_call.expression
            else {
                bail!(internal_error(
                    "`ty_fn_call` is not a `TyExpressionVariant::FunctionApplication`."
                ));
            };

            let ty_fn = ctx.engines.de().get_function(fn_ref.id());
            // We need the full path to the function to ensure it is defined in the current package.
            if ty_fn.call_path.callpath_type != CallPathType::Full {
                return Ok(InvalidateTypedElement::No);
            }

            let Some(fn_pkg_name) = ty_fn.call_path.prefixes.first() else {
                return Ok(InvalidateTypedElement::No);
            };

            if fn_pkg_name.as_str() != ctx.pkg_name {
                return Ok(InvalidateTypedElement::No);
            }

            output.push(last_segment.span().into());

            modify(last_segment).set_name("r#panic");

            Ok(InvalidateTypedElement::No)
        }

        fn visit_method_call(
            &mut self,
            ctx: &VisitingContext,
            lexed_method_call: &mut Expr,
            ty_method_call: Option<&TyExpression>,
            output: &mut Vec<Occurrence>,
        ) -> Result<InvalidateTypedElement> {
            // We report the occurrences only if it is not a dry-run.
            if ctx.dry_run == DryRun::Yes {
                return Ok(InvalidateTypedElement::No);
            }

            let lexed_method_call_info = LexedMethodCallInfoMut::new(lexed_method_call)?;
            let ty_method_call_info = ty_method_call
                .map(|ty_method_call| TyMethodCallInfo::new(ctx.engines.de(), ty_method_call))
                .transpose()?;

            let Some(ty_method_call_info) = ty_method_call_info else {
                // Without the typed method call, we cannot proceed
                // because we cannot check if the method is actually defined in the current package.
                return Ok(InvalidateTypedElement::No);
            };

            if lexed_method_call_info.path_seg.name.as_raw_ident_str() != "panic" {
                return Ok(InvalidateTypedElement::No);
            }

            let ty_method = ty_method_call_info.fn_decl;
            // We need the full path to the function to ensure it is defined in the current package.
            if ty_method.call_path.callpath_type != CallPathType::Full {
                return Ok(InvalidateTypedElement::No);
            }

            let Some(fn_pkg_name) = ty_method.call_path.prefixes.first() else {
                return Ok(InvalidateTypedElement::No);
            };

            if fn_pkg_name.as_str() != ctx.pkg_name {
                return Ok(InvalidateTypedElement::No);
            }

            output.push(lexed_method_call_info.path_seg.span().into());

            modify(lexed_method_call_info.path_seg).set_name("r#panic");

            Ok(InvalidateTypedElement::No)
        }

        fn visit_statement_let(
            &mut self,
            ctx: &VisitingContext,
            lexed_let: &mut StatementLet,
            _ty_var_decl: Option<&TyVariableDecl>,
            output: &mut Vec<Occurrence>,
        ) -> Result<InvalidateTypedElement> {
            let var_name = match &mut lexed_let.pattern {
                sway_ast::Pattern::Var { name, .. } => name,
                // A valid identifier in a variable name pattern can only be a variable,
                // never an enum variant. So we know that this `ident` is a variable.
                sway_ast::Pattern::AmbiguousSingleIdent(ident) => ident,
                _ => {
                    // NOTE: We don't support renaming `panic` in patterns other than variables,
                    //       e.g., in deconstruction patterns.
                    return Ok(InvalidateTypedElement::No);
                }
            };

            if var_name.as_raw_ident_str() != "panic" {
                return Ok(InvalidateTypedElement::No);
            }

            output.push(var_name.span().into());

            if ctx.dry_run == DryRun::Yes {
                return Ok(InvalidateTypedElement::No);
            }

            *var_name = Ident::new_with_raw(var_name.span(), true);

            Ok(InvalidateTypedElement::No)
        }

        fn visit_expr(
            &mut self,
            ctx: &VisitingContext,
            lexed_expr: &mut Expr,
            _ty_expr: Option<&TyExpression>,
            output: &mut Vec<Occurrence>,
        ) -> Result<InvalidateTypedElement> {
            // We report the occurrences only if it is not a dry-run.
            if ctx.dry_run == DryRun::Yes {
                return Ok(InvalidateTypedElement::No);
            }

            let var_names = match lexed_expr {
                Expr::Path(path) if path.suffix.is_empty() => vec![&mut path.prefix.name],
                Expr::Struct { fields, .. } => fields
                    .inner
                    .iter_mut()
                    .map(|field| &mut field.field_name)
                    .collect(),
                Expr::FieldProjection { name, .. } => vec![name],
                _ => vec![],
            };

            for var_name in var_names
                .into_iter()
                .filter(|n| n.as_raw_ident_str() == "panic")
            {
                output.push(var_name.span().into());

                *var_name = Ident::new_with_raw(var_name.span(), true);
            }

            Ok(InvalidateTypedElement::No)
        }

        fn visit_reassignment(
            &mut self,
            ctx: &VisitingContext,
            _lexed_op: &mut expr::ReassignmentOp,
            lexed_lhs: &mut Assignable,
            _ty_lhs: Option<&TyReassignmentTarget>,
            _lexed_rhs: &mut Expr,
            _ty_rhs: Option<&TyExpression>,
            output: &mut Vec<Occurrence>,
        ) -> Result<InvalidateTypedElement> {
            // On the LHS, we support renaming `panic` only in these cases:
            // - Variable names, e.g., `let panic = 42;`
            // - Single field access, e.g., `let x.panic = 42;`
            // But occurrences in, e.g., `foo[panic].x = 42;` will not be renamed.
            // Full traversal of reassignments' LHS will be done as a part of migration
            // infrastructure in the future.

            // We report the occurrences only if it is not a dry-run.
            if ctx.dry_run == DryRun::Yes {
                return Ok(InvalidateTypedElement::No);
            }

            let var_names = match lexed_lhs {
                Assignable::ElementAccess(element_access) => match element_access {
                    ElementAccess::Var(name) => vec![name],
                    ElementAccess::FieldProjection {
                        target: element_access,
                        name,
                        ..
                    } => {
                        let mut names = vec![name];
                        if let ElementAccess::Var(name) = &mut **element_access {
                            names.push(name)
                        };
                        names
                    }
                    ElementAccess::TupleFieldProjection {
                        target: element_access,
                        ..
                    }
                    | ElementAccess::Index {
                        target: element_access,
                        ..
                    } => match &mut **element_access {
                        ElementAccess::Var(name) => vec![name],
                        _ => vec![],
                    },
                    _ => vec![],
                },
                Assignable::Deref { .. } => vec![],
            };

            for var_name in var_names
                .into_iter()
                .filter(|n| n.as_raw_ident_str() == "panic")
            {
                output.push(var_name.span().into());

                *var_name = Ident::new_with_raw(var_name.span(), true);
            }

            Ok(InvalidateTypedElement::No)
        }
    }

    ProgramVisitorMut::visit_program(program_info, dry_run, &mut Visitor {})
}
