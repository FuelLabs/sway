use crate::{Expression, Scrutinee, Literal, Span, Ident};

// if (x == y)
pub type MatchReqMap<'sc> = Vec<(Expression<'sc>, Expression<'sc>)>;
// let z = 4;
pub type MatchImplMap<'sc> = Vec<(Ident<'sc>, Expression<'sc>)>;

pub fn matcher<'sc>(exp: &Expression<'sc>, scrutinee: &Scrutinee<'sc>) -> Option<(MatchReqMap<'sc>, MatchImplMap<'sc>)> {
    match scrutinee {
        Scrutinee::Unit { span } => unimplemented!(),
        Scrutinee::Literal { value, span } => match_literal(exp, value, span),
    }
}

fn match_literal<'sc>(exp: &Expression<'sc>, scrutinee: &Literal<'sc>, scrutinee_span: &Span<'sc>) -> Option<(MatchReqMap<'sc>, MatchImplMap<'sc>)> {
    match exp {
        Expression::VariableExpression { name, span } => {
            let match_req_map = vec![];
            let match_impl_map = vec![(
                name.clone(),
                Expression::Literal { value: scrutinee.clone(), span: scrutinee_span.clone() },
            )];
            Some((match_req_map, match_impl_map))
        }
        Expression::Literal { value, span } => {
            let match_req_map = vec![(
                Expression::Literal { value: value.clone(), span: span.clone() },
                Expression::Literal { value: scrutinee.clone(), span: scrutinee_span.clone() },
            )];
            let match_impl_map = vec![];
            Some((match_req_map, match_impl_map))
        },
        exp => unimplemented!("{:?}", exp)
    }
}