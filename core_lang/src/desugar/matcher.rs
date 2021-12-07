use crate::{Expression, Ident, Literal, Scrutinee, Span};

// if (x == y)
pub type MatchReqMap<'sc> = Vec<(Expression<'sc>, Expression<'sc>)>;
// let z = 4;
pub type MatchImplMap<'sc> = Vec<(Ident<'sc>, Expression<'sc>)>;

pub fn matcher<'sc>(
    exp: &Expression<'sc>,
    scrutinee: &Scrutinee<'sc>,
) -> Option<(MatchReqMap<'sc>, MatchImplMap<'sc>)> {
    match scrutinee {
        Scrutinee::Unit { span: _ } => unimplemented!(),
        Scrutinee::Literal { value, span } => match_literal(exp, value, span),
        Scrutinee::Variable { name, span } => match_variable(exp, name, span),
    }
}

fn match_literal<'sc>(
    exp: &Expression<'sc>,
    scrutinee: &Literal<'sc>,
    scrutinee_span: &Span<'sc>,
) -> Option<(MatchReqMap<'sc>, MatchImplMap<'sc>)> {
    let match_req_map = vec![(
        exp.to_owned(),
        Expression::Literal {
            value: scrutinee.clone(),
            span: scrutinee_span.clone(),
        },
    )];
    let match_impl_map = vec![];
    Some((match_req_map, match_impl_map))
}

fn match_variable<'sc>(
    exp: &Expression<'sc>,
    scrutinee_name: &Ident<'sc>,
    _span: &Span<'sc>,
) -> Option<(MatchReqMap<'sc>, MatchImplMap<'sc>)> {
    let match_req_map = vec![];
    let match_impl_map = vec![(scrutinee_name.to_owned(), exp.to_owned())];
    Some((match_req_map, match_impl_map))
}
