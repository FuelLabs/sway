use crate::{Expression, Ident, Literal, Scrutinee, Span, StructScrutineeField};

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
        Scrutinee::StructScrutinee {
            struct_name,
            fields,
            span,
        } => match_struct(exp, struct_name, fields, span),
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

fn match_struct<'sc>(
    exp: &Expression<'sc>,
    struct_name: &Ident<'sc>,
    fields: &Vec<StructScrutineeField<'sc>>,
    span: &Span<'sc>,
) -> Option<(MatchReqMap<'sc>, MatchImplMap<'sc>)> {
    let mut match_req_map = vec![];
    let mut match_impl_map = vec![];
    for field in fields.into_iter() {
        let field_name = field.field.clone();
        let scrutinee = field.scrutinee.clone();
        let delayed_resolution_exp = Expression::DelayedStructFieldResolution {
            exp: Box::new(exp.clone()),
            struct_name: struct_name.to_owned(),
            field: field_name.primary_name,
            span: span.clone(),
        };
        match scrutinee {
            // if the scrutinee is simply naming the struct field ...
            None => {
                match_impl_map.push((field_name.clone(), delayed_resolution_exp));
            }
            // or if the scrutinee has a more complex agenda
            Some(scrutinee) => match matcher(&delayed_resolution_exp, &scrutinee) {
                Some((mut new_match_req_map, mut new_match_impl_map)) => {
                    match_req_map.append(&mut new_match_req_map);
                    match_impl_map.append(&mut new_match_impl_map);
                }
                None => return None,
            },
        }
    }
    Some((match_req_map, match_impl_map))
}
