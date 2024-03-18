use sway_types::Spanned;

use crate::{
    decl_engine::DeclRefConstant,
    language::{ty, SymbolPath},
    semantic_analysis::TypeCheckContext,
    TypeBinding,
};

pub(crate) fn instantiate_constant_expression(
    ctx: TypeCheckContext,
    const_ref: DeclRefConstant,
    symbol_path_binding: TypeBinding<SymbolPath>,
) -> ty::TyExpression {
    let const_decl = (*ctx.engines.de().get_constant(const_ref.id())).clone();
    ty::TyExpression {
        return_type: const_decl.return_type,
        span: symbol_path_binding.span(),
        expression: ty::TyExpressionVariant::ConstantExpression {
            const_decl: Box::new(const_decl),
            span: symbol_path_binding.inner.suffix.span(),
            symbol_path: Some(symbol_path_binding.inner.to_fullpath(ctx.namespace())),
        },
    }
}
