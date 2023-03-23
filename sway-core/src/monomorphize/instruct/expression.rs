use sway_error::handler::{ErrorEmitted, Handler};

use crate::{language::ty, monomorphize::priv_prelude::*, type_system::*};

pub(crate) fn instruct_exp(
    ctx: InstructContext,
    handler: &Handler,
    exp: &ty::TyExpression,
) -> Result<(), ErrorEmitted> {
    instruct_exp_inner(ctx, handler, &exp.expression, exp.return_type)
}

pub(crate) fn instruct_exp_inner(
    mut ctx: InstructContext,
    handler: &Handler,
    exp: &ty::TyExpressionVariant,
    _return_type: TypeId,
) -> Result<(), ErrorEmitted> {
    // NOTE: todo here
    match exp {
        ty::TyExpressionVariant::FunctionApplication {
            arguments,
            contract_call_params,
            ..
        } => {
            arguments
                .iter()
                .try_for_each(|(_, arg)| instruct_exp(ctx.by_ref(), handler, arg))?;
            contract_call_params
                .iter()
                .try_for_each(|(_, arg)| instruct_exp(ctx.by_ref(), handler, arg))?;
            // NOTE: todo here
        }
        ty::TyExpressionVariant::LazyOperator { lhs, rhs, .. } => {
            instruct_exp(ctx.by_ref(), handler, lhs)?;
            instruct_exp(ctx.by_ref(), handler, rhs)?;
        }
        ty::TyExpressionVariant::VariableExpression { .. } => {
            // NOTE: may need to do something here later
        }
        ty::TyExpressionVariant::Tuple { fields } => {
            fields
                .iter()
                .try_for_each(|field| instruct_exp(ctx.by_ref(), handler, field))?;
        }
        ty::TyExpressionVariant::Array { contents: _ } => {
            todo!();
            // contents
            //     .iter()
            //     .try_for_each(|elem| instruct_exp(ctx.by_ref(), handler, elem))?;
        }
        ty::TyExpressionVariant::ArrayIndex {
            prefix: _,
            index: _,
        } => {
            todo!();
            // instruct_exp(ctx.by_ref(), handler, prefix)?;
            // instruct_exp(ctx.by_ref(), handler, index)?;
        }
        ty::TyExpressionVariant::StructExpression { .. } => todo!(),
        ty::TyExpressionVariant::CodeBlock(block) => {
            instruct_code_block(ctx, handler, block)?;
        }
        ty::TyExpressionVariant::IfExp { .. } => todo!(),
        ty::TyExpressionVariant::MatchExp { .. } => todo!(),
        ty::TyExpressionVariant::AsmExpression { .. } => todo!(),
        ty::TyExpressionVariant::StructFieldAccess { .. } => todo!(),
        ty::TyExpressionVariant::TupleElemAccess { prefix, .. } => {
            instruct_exp(ctx, handler, prefix)?;
        }
        ty::TyExpressionVariant::EnumInstantiation { .. } => todo!(),
        ty::TyExpressionVariant::AbiCast { .. } => todo!(),
        ty::TyExpressionVariant::StorageAccess(_) => todo!(),
        ty::TyExpressionVariant::IntrinsicFunction(_) => todo!(),
        ty::TyExpressionVariant::AbiName(_) => todo!(),
        ty::TyExpressionVariant::EnumTag { exp } => {
            instruct_exp(ctx.by_ref(), handler, exp)?;
        }
        ty::TyExpressionVariant::UnsafeDowncast { .. } => todo!(),
        ty::TyExpressionVariant::WhileLoop { .. } => todo!(),
        ty::TyExpressionVariant::Reassignment(_) => todo!(),
        ty::TyExpressionVariant::StorageReassignment(_) => todo!(),
        ty::TyExpressionVariant::Return(exp) => {
            instruct_exp(ctx.by_ref(), handler, exp)?;
        }
        ty::TyExpressionVariant::Literal(_) => {}
        ty::TyExpressionVariant::Break => {}
        ty::TyExpressionVariant::Continue => {}
        ty::TyExpressionVariant::FunctionParameter => {}
    }

    Ok(())
}
