use sway_types::Span;

use crate::language::ty;

/// Returns an [ty::TyExpressionVariant::UnsafeDowncast] expression that
/// downcasts the expression `exp`, resulting in enum variant `variant`,
/// to its underlying type.
/// The expression `exp` **must** result in an enum variant `variant`.
/// E.g., for `let a = MyEnum::A(u64, u32)` downcasting `a` to `MyEnum::A`
/// will result in `a as (u64, u32)`.
pub(crate) fn instantiate_enum_unsafe_downcast(
    exp: &ty::TyExpression,
    variant: ty::TyEnumVariant,
    call_path_decl: ty::TyDecl,
    span: Span,
) -> ty::TyExpression {
    ty::TyExpression {
        expression: ty::TyExpressionVariant::UnsafeDowncast {
            exp: Box::new(exp.clone()),
            variant: variant.clone(),
            call_path_decl,
        },
        return_type: variant.type_argument.type_id(),
        span,
    }
}
