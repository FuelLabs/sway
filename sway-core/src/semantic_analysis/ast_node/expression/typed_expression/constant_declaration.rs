use sway_types::Spanned;

use crate::{
    error::*,
    language::{
        ty::{self, TyConstantDeclaration},
        CallPath,
    },
    CompileResult, TypeBinding,
};

pub(crate) fn instantiate_constant_decl(
    const_decl: TyConstantDeclaration,
    call_path_binding: TypeBinding<CallPath>,
) -> CompileResult<ty::TyExpression> {
    ok(
        ty::TyExpression {
            return_type: const_decl.type_ascription.type_id,
            span: call_path_binding.span(),
            expression: ty::TyExpressionVariant::ConstantExpression {
                const_decl: Box::new(const_decl),
            },
        },
        vec![],
        vec![],
    )
}
