use sway_types::{Span, Spanned};

use crate::{
    error::*,
    language::ty::{self, TyConstantDeclaration},
    CompileResult,
};

pub(crate) fn instantiate_constant_decl(
    const_decl: TyConstantDeclaration,
    span: Span,
) -> CompileResult<ty::TyExpression> {
    ok(
        ty::TyExpression {
            expression: ty::TyExpressionVariant::VariableExpression {
                name: const_decl.name.clone(),
                span: const_decl.name.span(),
                mutability: ty::VariableMutability::Immutable,
            },
            return_type: const_decl.value.return_type,
            span,
        },
        vec![],
        vec![],
    )
}
