use crate::{type_engine::TypeId, Ident, Literal, Span};

use super::{TypedExpression, TypedExpressionVariant, TypedScrutinee, TypedScrutineeVariant};

// if (x == y)
pub type MatchReqMap<'sc> = Vec<(TypedExpression<'sc>, TypedExpression<'sc>)>;
// let z = 4;
pub type MatchImplMap<'sc> = Vec<(Ident<'sc>, TypedExpression<'sc>)>;

pub(crate) fn matcher<'sc>(
    exp: &TypedExpression<'sc>,
    scrutinee: &TypedScrutinee<'sc>,
) -> Option<(MatchReqMap<'sc>, MatchImplMap<'sc>)> {
    let return_type = scrutinee.return_type;
    let span = scrutinee.span.clone();
    let variant = scrutinee.scrutinee.clone();
    match variant {
        TypedScrutineeVariant::Literal { value, .. } => {
            match_literal(exp, value, return_type, span)
        }
        scrutinee => unimplemented!("{:?}", scrutinee),
    }
}

fn match_literal<'sc>(
    exp: &TypedExpression<'sc>,
    scrutinee_value: Literal<'sc>,
    return_type: TypeId,
    span: Span<'sc>,
) -> Option<(MatchReqMap<'sc>, MatchImplMap<'sc>)> {
    let match_req_map = vec![(
        exp.to_owned(),
        TypedExpression {
            expression: TypedExpressionVariant::Literal(scrutinee_value.to_owned()),
            return_type,
            span,
            is_constant: crate::semantic_analysis::ast_node::IsConstant::No,
        },
    )];
    let match_impl_map = vec![];
    Some((match_req_map, match_impl_map))
}
