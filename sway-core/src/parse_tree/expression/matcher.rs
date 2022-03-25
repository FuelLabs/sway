use crate::{
    error::{err, ok},
    CallPath, CompileError, CompileResult, DelayedResolutionVariant, DelayedStructFieldResolution,
    DelayedTupleVariantResolution, Expression, Ident, Literal, Scrutinee, StructScrutineeField,
};

use sway_types::span::Span;

/// List of requirements that a desugared if expression must include in the conditional.
pub type MatchReqMap = Vec<(Expression, Expression)>;
/// List of variable declarations that must be placed inside of the body of the if expression.
pub type MatchImplMap = Vec<(Ident, Expression)>;
/// This is the result type given back by the matcher.
pub type MatcherResult = Option<(MatchReqMap, MatchImplMap)>;

/// This algorithm desugars pattern matching into a [MatcherResult], by creating two lists,
/// the [MatchReqMap] which is a list of requirements that a desugared if expression
/// must inlcude in the conditional, and the [MatchImplMap] which is a list of variable
/// declarations that must be placed inside the body of the if expression.
///
/// Given the following example
///
/// ```ignore
/// struct Point {
///     x: u64,
///     y: u64
/// }
///
/// let p = Point {
///     x: 42,
///     y: 24
/// };
///
/// match p {
///     Point { x, y: 5 } => { x },
///     Point { x, y: 24 } => { x },
///     _ => 0
/// }
/// ```
///
/// The first match arm would create a [MatchReqMap] of roughly:
///
/// ```ignore
/// [
///     (y, 5) // y must equal 5 to trigger this case
/// ]
/// ```
///
/// The first match arm would create a [MatchImplMap] of roughly:
///
/// ```ignore
/// [
///     (x, 42) // add `let x = 42` in the body of the desugared if expression
/// ]
/// ```
pub fn matcher(exp: &Expression, scrutinee: &Scrutinee) -> CompileResult<MatcherResult> {
    let mut errors = vec![];
    let warnings = vec![];
    match scrutinee {
        Scrutinee::Literal { value, span } => match_literal(exp, value, span),
        Scrutinee::Variable { name, span } => match_variable(exp, name, span),
        Scrutinee::StructScrutinee {
            struct_name,
            fields,
            span,
        } => match_struct(exp, struct_name, fields, span),
        Scrutinee::EnumScrutinee {
            call_path,
            variable_to_assign,
            span,
            ..
        } => match_enum(exp, call_path, variable_to_assign, span),
        Scrutinee::Tuple { elems, span } => match_tuple(exp, elems, span),
        scrutinee => {
            eprintln!("Unimplemented scrutinee: {:?}", scrutinee,);
            errors.push(CompileError::Unimplemented(
                "this match expression scrutinee is not implemented",
                scrutinee.span(),
            ));
            ok(Some((vec![], vec![])), warnings, errors)
        }
    }
}

fn match_literal(
    exp: &Expression,
    scrutinee: &Literal,
    scrutinee_span: &Span,
) -> CompileResult<MatcherResult> {
    let match_req_map = vec![(
        exp.to_owned(),
        Expression::Literal {
            value: scrutinee.clone(),
            span: scrutinee_span.clone(),
        },
    )];
    let match_impl_map = vec![];
    ok(Some((match_req_map, match_impl_map)), vec![], vec![])
}

fn match_variable(
    exp: &Expression,
    scrutinee_name: &Ident,
    _span: &Span,
) -> CompileResult<MatcherResult> {
    let match_req_map = vec![];
    let match_impl_map = vec![(scrutinee_name.to_owned(), exp.to_owned())];
    ok(Some((match_req_map, match_impl_map)), vec![], vec![])
}

fn match_struct(
    exp: &Expression,
    struct_name: &Ident,
    fields: &[StructScrutineeField],
    span: &Span,
) -> CompileResult<MatcherResult> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut match_req_map = vec![];
    let mut match_impl_map = vec![];
    for field in fields.iter() {
        let field_name = field.field.clone();
        let scrutinee = field.scrutinee.clone();
        let delayed_resolution_exp = Expression::DelayedMatchTypeResolution {
            variant: DelayedResolutionVariant::StructField(DelayedStructFieldResolution {
                exp: Box::new(exp.clone()),
                struct_name: struct_name.to_owned(),
                field: field_name.clone(),
            }),
            span: span.clone(),
        };
        match scrutinee {
            // if the scrutinee is simply naming the struct field ...
            None => {
                match_impl_map.push((field_name.clone(), delayed_resolution_exp));
            }
            // or if the scrutinee has a more complex agenda
            Some(scrutinee) => {
                let new_matches = check!(
                    matcher(&delayed_resolution_exp, &scrutinee),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                match new_matches {
                    Some((mut new_match_req_map, mut new_match_impl_map)) => {
                        match_req_map.append(&mut new_match_req_map);
                        match_impl_map.append(&mut new_match_impl_map);
                    }
                    None => return ok(None, warnings, errors),
                }
            }
        }
    }

    ok(Some((match_req_map, match_impl_map)), warnings, errors)
}

fn match_enum(
    _exp: &Expression,
    _call_path: &CallPath,
    _arg: &Ident,
    span: &Span,
) -> CompileResult<MatcherResult> {
    err(
        vec![],
        vec![CompileError::Unimplemented(
            "Matching enums has not yet been implemented.",
            span.clone(),
        )],
    )
}

fn match_tuple(exp: &Expression, elems: &[Scrutinee], span: &Span) -> CompileResult<MatcherResult> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut match_req_map = vec![];
    let mut match_impl_map = vec![];
    for (pos, elem) in elems.iter().enumerate() {
        let delayed_resolution_exp = Expression::DelayedMatchTypeResolution {
            variant: DelayedResolutionVariant::TupleVariant(DelayedTupleVariantResolution {
                exp: Box::new(exp.clone()),
                elem_num: pos,
            }),
            span: span.clone(),
        };
        let new_matches = check!(
            matcher(&delayed_resolution_exp, elem),
            return err(warnings, errors),
            warnings,
            errors
        );
        match new_matches {
            Some((mut new_match_req_map, mut new_match_impl_map)) => {
                match_req_map.append(&mut new_match_req_map);
                match_impl_map.append(&mut new_match_impl_map);
            }
            None => return ok(None, warnings, errors),
        }
    }

    ok(Some((match_req_map, match_impl_map)), warnings, errors)
}
