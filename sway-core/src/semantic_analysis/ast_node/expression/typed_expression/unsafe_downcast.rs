use sway_types::{integer_bits::IntegerBits, Span};

use crate::{
    language::{ty, Literal},
    semantic_analysis::{
        ast_node::expression::match_expression::MatchReqMap, IsConstant, TyEnumVariant,
    },
    type_system::insert_type,
    TypeInfo,
};
// currently the unsafe downcast expr is only used for enums, so this method is specialized for enums
pub(crate) fn instantiate_unsafe_downcast(
    exp: &ty::TyExpression,
    variant: TyEnumVariant,
    span: Span,
) -> (MatchReqMap, ty::TyExpression) {
    let match_req_map = vec![(
        ty::TyExpression {
            expression: ty::TyExpressionVariant::EnumTag {
                exp: Box::new(exp.clone()),
            },
            return_type: insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)),
            is_constant: IsConstant::No,
            span: exp.span.clone(),
        },
        ty::TyExpression {
            expression: ty::TyExpressionVariant::Literal(Literal::U64(variant.tag as u64)),
            return_type: insert_type(TypeInfo::UnsignedInteger(IntegerBits::SixtyFour)),
            is_constant: IsConstant::No,
            span: exp.span.clone(),
        },
    )];
    let unsafe_downcast = ty::TyExpression {
        expression: ty::TyExpressionVariant::UnsafeDowncast {
            exp: Box::new(exp.clone()),
            variant: variant.clone(),
        },
        return_type: variant.type_id,
        is_constant: IsConstant::No,
        span,
    };
    (match_req_map, unsafe_downcast)
}
