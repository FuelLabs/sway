use crate::{
    migrations::MutProgramInfo,
    modifying::*,
    visiting::{
        InvalidateTypedElement, LexedFnCallInfoMut, ProgramVisitorMut, TreesVisitorMut,
        TyFnCallInfo, VisitingContext,
    },
};
use anyhow::{bail, Ok, Result};
use sway_ast::Expr;
use sway_core::language::{ty::TyExpression, CallPath};
use sway_types::{Span, Spanned};

use super::{ContinueMigrationProcess, DryRun, MigrationStep, MigrationStepKind};

// NOTE: We do not fully support cases when `b256::from` is nested within another `b256::from`.
//       E.g.: `b256::from(Bytes::from(b256::from(nested_bytes)))`.
//       In such cases, only the outermost `b256::from` will be migrated.
//       In practice, this does not happen.

#[allow(dead_code)]
pub(super) const REPLACE_B256_FROM_BYTES_TO_TRY_FROM_BYTES_STEP: MigrationStep = MigrationStep {
    title: "Replace calls to `b256::from(<bytes>)` with `b256::try_from(<bytes>).unwrap()`",
    duration: 0,
    kind: MigrationStepKind::CodeModification(
        replace_b256_from_bytes_to_try_from_bytes_step,
        &[],
        ContinueMigrationProcess::IfNoManualMigrationActionsNeeded,
    ),
    help: &[
        "Migration will replace all the calls to `b256::from(<bytes>)` with",
        "`b256::try_from(<bytes>).unwrap()`.",
        " ",
        "E.g.:",
        "  let result = b256::from(some_bytes);",
        "will become:",
        "  let result = b256::try_from(some_bytes).unwrap();",
    ],
};

fn replace_b256_from_bytes_to_try_from_bytes_step(
    program_info: &mut MutProgramInfo,
    dry_run: DryRun,
) -> Result<Vec<Span>> {
    struct Visitor;
    impl TreesVisitorMut<Span> for Visitor {
        fn visit_fn_call(
            &mut self,
            ctx: &VisitingContext,
            lexed_fn_call: &mut Expr,
            ty_fn_call: Option<&TyExpression>,
            output: &mut Vec<Span>,
        ) -> Result<InvalidateTypedElement> {
            let lexed_fn_call_info = LexedFnCallInfoMut::new(lexed_fn_call)?;
            let ty_fn_call_info = ty_fn_call
                .map(|ty_fn_call| TyFnCallInfo::new(ctx.engines.de(), ty_fn_call))
                .transpose()?;

            // We need the typed info in order to ensure that the `from` function
            // is really the `b256::from(Bytes)` function.
            let Some(ty_fn_call_info) = ty_fn_call_info else {
                return Ok(InvalidateTypedElement::No);
            };

            // Note that neither the implementing for type not the trait are a
            // part of the `from` function call path. All associated `from` functions
            // in the `std::bytes` will have the same call path.
            // We will filter further below to target exactly the `<From<Bytes> for b256>::from`.
            let from_call_path = CallPath::fullpath(&["std", "bytes", "from"]);

            let Some(implementing_for_type_id) = ty_fn_call_info.fn_decl.implementing_for_typeid
            else {
                return Ok(InvalidateTypedElement::No);
            };

            // This check is sufficient. The only `from` in `std::bytes` that
            // satisfies it is the `<From<Bytes> for b256>::from`.
            if !(ty_fn_call_info.fn_decl.call_path == from_call_path
                && implementing_for_type_id == ctx.engines.te().id_of_b256())
            {
                return Ok(InvalidateTypedElement::No);
            }

            // We have found a `b256::from(Bytes)` call.
            output.push(lexed_fn_call_info.func.span());

            if ctx.dry_run == DryRun::Yes {
                return Ok(InvalidateTypedElement::No);
            }

            let lexed_from_call_path = match lexed_fn_call {
                Expr::FuncApp { func, args: _ } => match func.as_mut() {
                    Expr::Path(path_expr) => path_expr,
                    _ => bail!("`func` of the `lexed_fn_call` must be of the variant `Expr::Path`."),
                },
                _ => bail!("`lexed_fn_call` must be of the variant `Expr::FuncApp`."),
            };

            // Rename the call to `from` to `try_from`.
            let from_ident = lexed_from_call_path.last_segment_mut();
            modify(from_ident).set_name("try_from");

            // The call to `try_from` becomes the target of the `unwrap` method call.
            let target = lexed_fn_call.clone();
            let insert_span = Span::empty_at_end(&target.span());
            *lexed_fn_call = New::method_call(insert_span, target, "unwrap");

            Ok(InvalidateTypedElement::Yes)
        }
    }

    ProgramVisitorMut::visit_program(program_info, dry_run, &mut Visitor {})
}
