use sway_types::Span;

use crate::{
    semantic_analysis::{
        ast_node::expression::match_expression::MatchReqMap, IsConstant, TyEnumVariant,
        TyExpressionVariant,
    },
    type_system::{insert_type, IntegerBits},
    Literal, TypeInfo,
};

use super::TyExpression;

// currently the unsafe downcast expr is only used for enums, so this method is specialized for enums
pub(crate) fn instantiate_unsafe_downcast(
    exp: &TyExpression,
    variant: TyEnumVariant,
    span: Span,
) -> (MatchReqMap, TyExpression) {
    let match_req_map = vec![(
        TyExpression {
            expression: TyExpressionVariant::EnumTag {
                exp: Box::new(exp.clone()),
            },
            return_type: insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)),
            is_constant: IsConstant::No,
            span: exp.span.clone(),
        },
        TyExpression {
            expression: TyExpressionVariant::Literal(Literal::U64(variant.tag as u64)),
            return_type: insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)),
            is_constant: IsConstant::No,
            span: exp.span.clone(),
        },
    )];
    let unsafe_downcast = TyExpression {
        expression: TyExpressionVariant::UnsafeDowncast {
            exp: Box::new(exp.clone()),
            variant: variant.clone(),
        },
        return_type: variant.type_id,
        is_constant: IsConstant::No,
        span,
    };
    (match_req_map, unsafe_downcast)
}
