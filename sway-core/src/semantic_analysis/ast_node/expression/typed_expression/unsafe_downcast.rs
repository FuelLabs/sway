use sway_types::Span;

use crate::{
    semantic_analysis::{
        ast_node::expression::match_expression::MatchReqMap, IsConstant, TypedEnumVariant,
        TypedExpressionVariant,
    },
    type_system::{IntegerBits, TypeEngine},
    Literal, TypeInfo,
};

use super::TypedExpression;

// currently the unsafe downcast expr is only used for enums, so this method is specialized for enums
pub(crate) fn instantiate_unsafe_downcast(
    type_engine: &TypeEngine,
    exp: &TypedExpression,
    variant: TypedEnumVariant,
    span: Span,
) -> (MatchReqMap, TypedExpression) {
    let match_req_map = vec![(
        TypedExpression {
            expression: TypedExpressionVariant::EnumTag {
                exp: Box::new(exp.clone()),
            },
            return_type: type_engine.insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)),
            is_constant: IsConstant::No,
            span: exp.span.clone(),
        },
        TypedExpression {
            expression: TypedExpressionVariant::Literal(Literal::U64(variant.tag as u64)),
            return_type: type_engine.insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)),
            is_constant: IsConstant::No,
            span: exp.span.clone(),
        },
    )];
    let unsafe_downcast = TypedExpression {
        expression: TypedExpressionVariant::UnsafeDowncast {
            exp: Box::new(exp.clone()),
            variant: variant.clone(),
        },
        return_type: variant.type_id,
        is_constant: IsConstant::No,
        span,
    };
    (match_req_map, unsafe_downcast)
}
