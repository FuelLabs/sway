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
            return_type: const_decl.value.return_type,
            span: call_path_binding.span(),
            expression: ty::TyExpressionVariant::VariableExpression {
                name: const_decl.name.clone(),
                span: const_decl.name.span(),
                mutability: ty::VariableMutability::Immutable,
                call_path: Some(call_path_binding.inner),
            },
        },
        vec![],
        vec![],
    )
}
