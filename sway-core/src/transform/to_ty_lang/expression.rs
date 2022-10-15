use crate::language::{parsed, ty};

pub(crate) fn transform_to_ty_expression(exp: parsed::Expression) -> ty::TyExpression {
    let parsed::Expression { kind, span } = exp;

    match kind {
        parsed::ExpressionKind::Error(_) => todo!(),
        parsed::ExpressionKind::Literal(_) => todo!(),
        parsed::ExpressionKind::FunctionApplication(_) => todo!(),
        parsed::ExpressionKind::LazyOperator(_) => todo!(),
        parsed::ExpressionKind::Variable(_) => todo!(),
        parsed::ExpressionKind::Tuple(_) => todo!(),
        parsed::ExpressionKind::TupleIndex(_) => todo!(),
        parsed::ExpressionKind::Array(_) => todo!(),
        parsed::ExpressionKind::Struct(_) => todo!(),
        parsed::ExpressionKind::CodeBlock(_) => todo!(),
        parsed::ExpressionKind::If(_) => todo!(),
        parsed::ExpressionKind::Match(_) => todo!(),
        parsed::ExpressionKind::Asm(_) => todo!(),
        parsed::ExpressionKind::MethodApplication(_) => todo!(),
        parsed::ExpressionKind::Subfield(_) => todo!(),
        parsed::ExpressionKind::DelineatedPath(_) => todo!(),
        parsed::ExpressionKind::AbiCast(_) => todo!(),
        parsed::ExpressionKind::ArrayIndex(_) => todo!(),
        parsed::ExpressionKind::StorageAccess(_) => todo!(),
        parsed::ExpressionKind::IntrinsicFunction(_) => todo!(),
        parsed::ExpressionKind::WhileLoop(_) => todo!(),
        parsed::ExpressionKind::Break => todo!(),
        parsed::ExpressionKind::Continue => todo!(),
        parsed::ExpressionKind::Reassignment(_) => todo!(),
        parsed::ExpressionKind::Return(_) => todo!(),
    }
}
