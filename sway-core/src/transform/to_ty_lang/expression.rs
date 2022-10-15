use sway_types::{integer_bits::IntegerBits, Ident, Span, Spanned};

use crate::{
    language::{parsed, ty, Literal},
    semantic_analysis::IsConstant,
    type_system::*,
};

pub(crate) fn transform_to_ty_expression(exp: parsed::Expression) -> ty::TyExpression {
    let parsed::Expression { kind, span } = exp;

    match kind {
        parsed::ExpressionKind::Error(_) => ty::TyExpression::error(span),
        parsed::ExpressionKind::Literal(lit) => transform_to_ty_literal(lit, span),
        parsed::ExpressionKind::FunctionApplication(_) => todo!(),
        parsed::ExpressionKind::LazyOperator(exp) => transform_to_ty_lazy_op(exp, span),
        parsed::ExpressionKind::Variable(name) => transform_to_ty_variable(name, span),
        parsed::ExpressionKind::Tuple(elems) => todo!(),
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

fn transform_to_ty_literal(lit: Literal, span: Span) -> ty::TyExpression {
    let type_info = match &lit {
        Literal::String(s) => TypeInfo::Str(s.as_str().len() as u64),
        Literal::Numeric(_) => TypeInfo::Numeric,
        Literal::U8(_) => TypeInfo::UnsignedInteger(IntegerBits::Eight),
        Literal::U16(_) => TypeInfo::UnsignedInteger(IntegerBits::Sixteen),
        Literal::U32(_) => TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
        Literal::U64(_) => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
        Literal::Boolean(_) => TypeInfo::Boolean,
        Literal::B256(_) => TypeInfo::B256,
    };
    ty::TyExpression {
        expression: ty::TyExpressionVariant::Literal(lit),
        return_type: insert_type(type_info),
        is_constant: IsConstant::Yes,
        span,
    }
}

fn transform_to_ty_lazy_op(exp: parsed::LazyOperatorExpression, span: Span) -> ty::TyExpression {
    let parsed::LazyOperatorExpression { op, lhs, rhs } = exp;
    let lhs = transform_to_ty_expression(*lhs);
    let rhs = transform_to_ty_expression(*rhs);
    ty::TyExpression {
        expression: ty::TyExpressionVariant::LazyOperator {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        },
        return_type: insert_type(TypeInfo::Unknown),
        is_constant: IsConstant::No,
        span,
    }
}

fn transform_to_ty_variable(name: Ident, span: Span) -> ty::TyExpression {
    ty::TyExpression {
        expression: ty::TyExpressionVariant::VariableExpression {
            name: name.clone(),
            span: name.span(),
            mutability: ty::VariableMutability::Immutable,
        },
        return_type: insert_type(TypeInfo::Unknown),
        is_constant: IsConstant::No,
        span,
    }
}

fn transform_to_ty_tuple(elems: Vec<parsed::Expression>, span: Span) -> ty::TyExpression {
    let fields = elems.into_iter().map(|elem| transform_to_ty_expression(elem)).collect::<Vec<_>>();
    let field_types = fields.iter().map(|field| field.return_type).collect::<Vec<_>>();
    ty::TyExpression {
        expression: ty::TyExpressionVariant::Tuple {
            fields,
        },
        return_type: insert_type(TypeInfo::Tuple(field_types)),
        is_constant,
        span,
    }
}
