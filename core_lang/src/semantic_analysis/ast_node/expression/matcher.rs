use crate::Ident;

use super::{TypedExpression, TypedScrutinee};

// if (x == y)
pub type MatchReqMap<'sc> = Vec<(TypedExpression<'sc>, TypedExpression<'sc>)>;
// let z = 4;
pub type MatchImplMap<'sc> = Vec<(Ident<'sc>, TypedExpression<'sc>)>;

pub(crate) fn matcher<'sc>(
    exp: &TypedExpression<'sc>,
    scrutinee: &TypedScrutinee<'sc>,
) -> Option<(MatchReqMap<'sc>, MatchImplMap<'sc>)> {
    unimplemented!()
}
