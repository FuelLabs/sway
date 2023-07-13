use sway_types::Spanned;

use crate::{
    decl_engine::DeclRefConstant,
    error::*,
    language::{ty, CallPath},
    semantic_analysis::TypeCheckContext,
    CompileResult, TypeBinding,
};

pub(crate) fn instantiate_constant_expression(
    ctx: TypeCheckContext,
    const_ref: DeclRefConstant,
    call_path_binding: TypeBinding<CallPath>,
) -> CompileResult<ty::TyExpression> {
    let const_decl = ctx.engines.de().get_constant(const_ref.id());
    ok(
        ty::TyExpression {
            return_type: const_decl.return_type,
            span: call_path_binding.span(),
            expression: ty::TyExpressionVariant::ConstantExpression {
                const_decl: Box::new(const_decl),
                span: call_path_binding.inner.suffix.span(),
                call_path: Some(call_path_binding.inner.to_fullpath(ctx.namespace)),
            },
        },
        vec![],
        vec![],
    )
}
