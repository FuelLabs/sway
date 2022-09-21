use crate::{
    error::{err, ok},
    semantic_analysis::{
        ast_node::expression::typed_expression::{
            instantiate_struct_field_access, instantiate_tuple_index_access,
            instantiate_unsafe_downcast,
        },
        namespace::Namespace,
        IsConstant, TypedEnumVariant, TypedExpression, TypedExpressionVariant, VariableMutability,
    },
    type_system::unify,
    CompileResult, Ident, Literal, TypeId,
};

use sway_types::span::Span;

use super::typed_scrutinee::{TypedScrutinee, TypedScrutineeVariant, TypedStructScrutineeField};

/// List of requirements that a desugared if expression must include in the conditional.
pub(crate) type MatchReqMap = Vec<(TypedExpression, TypedExpression)>;
/// List of variable declarations that must be placed inside of the body of the if expression.
pub(crate) type MatchDeclMap = Vec<(Ident, TypedExpression)>;
/// This is the result type given back by the matcher.
pub(crate) type MatcherResult = (MatchReqMap, MatchDeclMap);

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
pub(crate) fn matcher(
    exp: &TypedExpression,
    scrutinee: TypedScrutinee,
    namespace: &mut Namespace,
) -> CompileResult<MatcherResult> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let TypedScrutinee {
        variant,
        type_id,
        span,
    } = scrutinee;

    // unify the type of the scrutinee with the type of the expression
    let (mut new_warnings, new_errors) = unify(type_id, exp.return_type, &span, "");
    warnings.append(&mut new_warnings);
    errors.append(&mut new_errors.into_iter().map(|x| x.into()).collect());

    if !errors.is_empty() {
        return err(warnings, errors);
    }

    match variant {
        TypedScrutineeVariant::CatchAll => ok((vec![], vec![]), warnings, errors),
        TypedScrutineeVariant::Literal(value) => match_literal(exp, value, span),
        TypedScrutineeVariant::Variable(name) => match_variable(exp, name),
        TypedScrutineeVariant::Constant(name, _, type_id) => {
            match_constant(exp, name, type_id, span)
        }
        TypedScrutineeVariant::StructScrutinee(_, fields) => match_struct(exp, fields, namespace),
        TypedScrutineeVariant::EnumScrutinee { value, variant, .. } => {
            match_enum(exp, variant, *value, span, namespace)
        }
        TypedScrutineeVariant::Tuple(elems) => match_tuple(exp, elems, span, namespace),
    }
}

fn match_literal(
    exp: &TypedExpression,
    scrutinee: Literal,
    span: Span,
) -> CompileResult<MatcherResult> {
    let match_req_map = vec![(
        exp.to_owned(),
        TypedExpression {
            expression: TypedExpressionVariant::Literal(scrutinee),
            return_type: exp.return_type,
            is_constant: IsConstant::No,
            span,
        },
    )];
    let match_decl_map = vec![];
    ok((match_req_map, match_decl_map), vec![], vec![])
}

fn match_variable(exp: &TypedExpression, scrutinee_name: Ident) -> CompileResult<MatcherResult> {
    let match_req_map = vec![];
    let match_decl_map = vec![(scrutinee_name, exp.to_owned())];

    ok((match_req_map, match_decl_map), vec![], vec![])
}

fn match_constant(
    exp: &TypedExpression,
    scrutinee_name: Ident,
    scrutinee_type_id: TypeId,
    span: Span,
) -> CompileResult<MatcherResult> {
    let match_req_map = vec![(
        exp.to_owned(),
        TypedExpression {
            expression: TypedExpressionVariant::VariableExpression {
                name: scrutinee_name,
                span: span.clone(),
                mutability: VariableMutability::Immutable,
            },
            return_type: scrutinee_type_id,
            is_constant: IsConstant::Yes,
            span,
        },
    )];
    let match_decl_map = vec![];

    ok((match_req_map, match_decl_map), vec![], vec![])
}

fn match_struct(
    exp: &TypedExpression,
    fields: Vec<TypedStructScrutineeField>,
    namespace: &mut Namespace,
) -> CompileResult<MatcherResult> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut match_req_map = vec![];
    let mut match_decl_map = vec![];
    for TypedStructScrutineeField {
        field,
        scrutinee,
        span: field_span,
    } in fields.into_iter()
    {
        let subfield = check!(
            instantiate_struct_field_access(exp.clone(), field.clone(), field_span),
            return err(warnings, errors),
            warnings,
            errors
        );
        match scrutinee {
            // if the scrutinee is simply naming the struct field ...
            None => {
                match_decl_map.push((field, subfield));
            }
            // or if the scrutinee has a more complex agenda
            Some(scrutinee) => {
                let (mut new_match_req_map, mut new_match_decl_map) = check!(
                    matcher(&subfield, scrutinee, namespace),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                match_req_map.append(&mut new_match_req_map);
                match_decl_map.append(&mut new_match_decl_map);
            }
        }
    }

    ok((match_req_map, match_decl_map), warnings, errors)
}

fn match_enum(
    exp: &TypedExpression,
    variant: TypedEnumVariant,
    scrutinee: TypedScrutinee,
    span: Span,
    namespace: &mut Namespace,
) -> CompileResult<MatcherResult> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let (mut match_req_map, unsafe_downcast) = instantiate_unsafe_downcast(exp, variant, span);
    let (mut new_match_req_map, match_decl_map) = check!(
        matcher(&unsafe_downcast, scrutinee, namespace),
        return err(warnings, errors),
        warnings,
        errors
    );
    match_req_map.append(&mut new_match_req_map);
    ok((match_req_map, match_decl_map), warnings, errors)
}

fn match_tuple(
    exp: &TypedExpression,
    elems: Vec<TypedScrutinee>,
    span: Span,
    namespace: &mut Namespace,
) -> CompileResult<MatcherResult> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut match_req_map = vec![];
    let mut match_decl_map = vec![];
    for (pos, elem) in elems.into_iter().enumerate() {
        let tuple_index_access = check!(
            instantiate_tuple_index_access(exp.clone(), pos, span.clone(), span.clone()),
            return err(warnings, errors),
            warnings,
            errors
        );
        let (mut new_match_req_map, mut new_match_decl_map) = check!(
            matcher(&tuple_index_access, elem, namespace),
            return err(warnings, errors),
            warnings,
            errors
        );
        match_req_map.append(&mut new_match_req_map);
        match_decl_map.append(&mut new_match_decl_map);
    }
    ok((match_req_map, match_decl_map), warnings, errors)
}
