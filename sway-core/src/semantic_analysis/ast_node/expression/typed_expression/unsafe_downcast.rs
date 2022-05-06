use sway_types::Span;

use crate::{
    semantic_analysis::{
        ast_node::expression::match_expression::MatchReqMap, IsConstant, TypedEnumVariant,
        TypedExpressionVariant,
    },
    type_engine::{insert_type, IntegerBits},
    Literal, TypeInfo,
};

use super::TypedExpression;

// currently the unsafe downcast expr is only used for enums, so this method is specialized for enums
pub(crate) fn instantiate_unsafe_downcast(
    exp: &TypedExpression,
    variant: TypedEnumVariant,
    span: Span,
) -> (MatchReqMap, TypedExpression) {
    let match_req_map = vec![(
        TypedExpression {
            expression: TypedExpressionVariant::EnumTag {
                exp: Box::new(exp.clone()),
            },
            return_type: insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)),
            is_constant: IsConstant::No,
            span: exp.span.clone(),
        },
        TypedExpression {
            expression: TypedExpressionVariant::Literal(Literal::U64(variant.tag as u64)),
            return_type: insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)),
            is_constant: IsConstant::No,
            span: variant.span.clone(),
        },
    )];
    let unsafe_downcast = TypedExpression {
        expression: TypedExpressionVariant::UnsafeDowncast {
            exp: Box::new(exp.clone()),
            variant_tag: variant.tag,
        },
        return_type: variant.r#type,
        is_constant: IsConstant::No,
        span,
    };
    (match_req_map, unsafe_downcast)
}
