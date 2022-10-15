use std::{collections::HashMap, sync::Arc};

use sway_types::{integer_bits::IntegerBits, Ident, Span, Spanned};

use crate::{
    language::{parsed, ty, Literal},
    type_system::*,
};

/// Transforms an untyped [Expression](parsed::Expression) into a typeable
/// [TyExpression](ty::TyExpression).
pub(crate) fn transform_to_ty_expression(exp: parsed::Expression) -> ty::TyExpression {
    let parsed::Expression { kind, span } = exp;

    match kind {
        parsed::ExpressionKind::Error(_) => ty::TyExpression::error(span),
        parsed::ExpressionKind::Literal(lit) => transform_to_ty_literal(lit, span),
        parsed::ExpressionKind::FunctionApplication(_) => todo!(),
        parsed::ExpressionKind::LazyOperator(exp) => transform_to_ty_lazy_op(exp, span),
        parsed::ExpressionKind::Variable(name) => transform_to_ty_variable(name, span),
        parsed::ExpressionKind::Tuple(elems) => transform_to_ty_tuple(elems, span),
        parsed::ExpressionKind::TupleIndex(exp) => transform_to_ty_tuple_index(exp, span),
        parsed::ExpressionKind::Array(elems) => transform_to_ty_array(elems, span),
        parsed::ExpressionKind::Struct(_) => todo!(),
        parsed::ExpressionKind::CodeBlock(_) => todo!(),
        parsed::ExpressionKind::If(exp) => transform_to_ty_if(exp, span),
        parsed::ExpressionKind::Match(_) => todo!(),
        parsed::ExpressionKind::Asm(asm) => transform_to_ty_asm(*asm, span),
        parsed::ExpressionKind::MethodApplication(_) => todo!(),
        parsed::ExpressionKind::Subfield(exp) => transform_to_ty_subfield(exp, span),
        parsed::ExpressionKind::DelineatedPath(_) => todo!(),
        parsed::ExpressionKind::AbiCast(_) => todo!(),
        parsed::ExpressionKind::ArrayIndex(exp) => transform_to_ty_array_index(exp, span),
        parsed::ExpressionKind::StorageAccess(exp) => todo!(),
        parsed::ExpressionKind::IntrinsicFunction(_) => todo!(),
        parsed::ExpressionKind::WhileLoop(_) => todo!(),
        parsed::ExpressionKind::Break => ty::TyExpression {
            expression: ty::TyExpressionVariant::Break,
            return_type: insert_type(TypeInfo::Unknown),
            span,
        },
        parsed::ExpressionKind::Continue => ty::TyExpression {
            expression: ty::TyExpressionVariant::Continue,
            return_type: insert_type(TypeInfo::Unknown),
            span,
        },
        parsed::ExpressionKind::Reassignment(_) => todo!(),
        parsed::ExpressionKind::Return(exp) => transform_to_ty_return(*exp, span),
    }
}

/// Transforms a [Literal] into a [TyExpression](ty::TyExpression), where the
/// `return_type` of the [TyExpression](ty::TyExpression) is calculated from the
/// [Literal].
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
        span,
    }
}

/// Transforms a [LazyOperatorExpression](parsed::LazyOperatorExpression) into a
/// [TyExpression](ty::TyExpression) with `return_type` of [TypeInfo::Unknown].
///
/// `return_type` is [TypeInfo::Unknown] because the result of a lazy op
/// expression is dependent upon a type annotation (if there is one), and that
/// is only known during type checking.
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
        span,
    }
}

/// Transforms an [Ident] `name` into a [TyExpression](ty::TyExpression) with
/// `return_type` of [TypeInfo::Unknown].
///
/// `return_type` is [TypeInfo::Unknown] because the type of the value bound to
/// variable `name` is only known during type checking.
fn transform_to_ty_variable(name: Ident, span: Span) -> ty::TyExpression {
    ty::TyExpression {
        expression: ty::TyExpressionVariant::VariableExpression {
            name: name.clone(),
            span: name.span(),
            mutability: ty::VariableMutability::Immutable,
        },
        return_type: insert_type(TypeInfo::Unknown),
        span,
    }
}

/// Transforms a `Vec` of [Expression](parsed::Expression) `elems` into a
/// [TyExpression](ty::TyExpression) with `return_type` of [TypeInfo::Tuple].
///
/// The elements of [TypeInfo::Tuple] are derived from `elems`.
fn transform_to_ty_tuple(elems: Vec<parsed::Expression>, span: Span) -> ty::TyExpression {
    let fields = elems
        .into_iter()
        .map(transform_to_ty_expression)
        .collect::<Vec<_>>();
    let field_types = fields
        .iter()
        .map(|field| TypeArgument {
            type_id: field.return_type,
            initial_type_id: field.return_type,
            span: field.span.clone(),
        })
        .collect::<Vec<_>>();
    ty::TyExpression {
        expression: ty::TyExpressionVariant::Tuple { fields },
        return_type: insert_type(TypeInfo::Tuple(field_types)),
        span,
    }
}

/// Transforms an [TupleIndexExpression](parsed::TupleIndexExpression) `exp`
/// into a [TyExpression](ty::TyExpression) with `return_type` of
/// [TypeInfo::Unknown].
///
/// `return_type` is [TypeInfo::Unknown] because the type located at the given
/// `index` of `exp` can only be determined during type checking.
fn transform_to_ty_tuple_index(exp: parsed::TupleIndexExpression, span: Span) -> ty::TyExpression {
    let parsed::TupleIndexExpression {
        prefix,
        index,
        index_span,
    } = exp;
    let prefix = transform_to_ty_expression(*prefix);
    ty::TyExpression {
        expression: ty::TyExpressionVariant::TupleElemAccess {
            resolved_prefix_type_id: prefix.return_type,
            prefix: Box::new(prefix),
            index,
            index_span,
        },
        return_type: insert_type(TypeInfo::Unknown),
        span,
    }
}

/// Transforms a `Vec` of [Expression](parsed::Expression) `elems` into a
/// [TyExpression](ty::TyExpression) with `return_type` of [TypeInfo::Array].
///
/// The elements of [TypeInfo::Array] are derived from `elems`.
fn transform_to_ty_array(elems: Vec<parsed::Expression>, span: Span) -> ty::TyExpression {
    let contents = elems
        .into_iter()
        .map(transform_to_ty_expression)
        .collect::<Vec<_>>();
    let elem_type = contents
        .get(0)
        .map(|elem| elem.return_type)
        .unwrap_or_else(|| insert_type(TypeInfo::Unknown));
    let array_count = contents.len();
    ty::TyExpression {
        expression: ty::TyExpressionVariant::Array { contents },
        return_type: insert_type(TypeInfo::Array(elem_type, array_count, elem_type)),
        span,
    }
}

/// Transforms an [IfExpression](parsed::IfExpression) `exp` into a
/// [TyExpression](ty::TyExpression) with `return_type` of [TypeInfo::Unknown].
///
/// `return_type` is [TypeInfo::Unknown] because it is dependent upon if the
/// branches of `exp` deterministically abort or not, which is only know during
/// type checking.
fn transform_to_ty_if(exp: parsed::IfExpression, span: Span) -> ty::TyExpression {
    let parsed::IfExpression {
        condition,
        then,
        r#else,
    } = exp;
    let condition = transform_to_ty_expression(*condition);
    let then = transform_to_ty_expression(*then);
    let r#else = r#else.map(|r#else| Box::new(transform_to_ty_expression(*r#else)));
    ty::TyExpression {
        expression: ty::TyExpressionVariant::IfExp {
            condition: Box::new(condition),
            then: Box::new(then),
            r#else,
        },
        return_type: insert_type(TypeInfo::Unknown),
        span,
    }
}

/// Transforms an [Expression](parsed::Expression) into a
/// [TyExpression](ty::TyExpression) with `return_type` of [TypeInfo::Unknown].
///
/// `return_type` is [TypeInfo::Unknown] because return statements do not
/// necessarily follow the type annotation of their immediate surrounding
/// context. Because a return statement is control flow that breaks out to the
/// nearest function, we need to type check it against the surrounding function.
fn transform_to_ty_return(exp: parsed::Expression, span: Span) -> ty::TyExpression {
    let exp = transform_to_ty_expression(exp);
    ty::TyExpression {
        expression: ty::TyExpressionVariant::Return(Box::new(exp)),
        return_type: insert_type(TypeInfo::Unknown),
        span,
    }
}

fn transform_to_ty_asm(asm: parsed::AsmExpression, span: Span) -> ty::TyExpression {
    let parsed::AsmExpression {
        registers,
        body,
        returns,
        return_type,
        whole_block_span,
    } = asm;
    let typed_registers = registers
        .into_iter()
        .map(|register| {
            let parsed::AsmRegisterDeclaration { name, initializer } = register;
            let initializer = initializer.map(transform_to_ty_expression);
            ty::TyAsmRegisterDeclaration { name, initializer }
        })
        .collect();
    ty::TyExpression {
        expression: ty::TyExpressionVariant::AsmExpression {
            whole_block_span,
            body,
            registers: typed_registers,
            returns,
        },
        return_type: insert_type(return_type),
        span,
    }
}

fn transform_to_ty_subfield(exp: parsed::SubfieldExpression, span: Span) -> ty::TyExpression {
    let parsed::SubfieldExpression {
        prefix,
        field_to_access,
    } = exp;
    let prefix = transform_to_ty_expression(*prefix);
    let type_id = insert_type(TypeInfo::Unknown);
    let field_to_access_span = field_to_access.span();
    let field_to_access = ty::TyStructField {
        name: field_to_access,
        type_id,
        initial_type_id: type_id,
        span: field_to_access_span.clone(),
        type_span: field_to_access_span.clone(),
        attributes: Arc::new(HashMap::new()),
    };
    ty::TyExpression {
        expression: ty::TyExpressionVariant::StructFieldAccess {
            resolved_prefix_type_id: prefix.return_type,
            prefix: Box::new(prefix),
            field_to_access,
            field_instantiation_span: field_to_access_span,
        },
        return_type: insert_type(TypeInfo::Unknown),
        span,
    }
}

fn transform_to_ty_array_index(exp: parsed::ArrayIndexExpression, span: Span) -> ty::TyExpression {
    let parsed::ArrayIndexExpression { prefix, index } = exp;
    let prefix = transform_to_ty_expression(*prefix);
    let index = transform_to_ty_expression(*index);
    ty::TyExpression {
        expression: ty::TyExpressionVariant::ArrayIndex {
            prefix: Box::new(prefix),
            index: Box::new(index),
        },
        return_type: insert_type(TypeInfo::Unknown),
        span,
    }
}

fn transform_to_ty_storage_access(
    exp: parsed::StorageAccessExpression,
    span: Span,
) -> ty::TyExpression {
    let parsed::StorageAccessExpression { field_names } = exp;
    ty::TyExpression {
        expression: ty::TyExpressionVariant::StorageAccess(storage_access),
        return_type,
        span: span.clone(),
    }
}
