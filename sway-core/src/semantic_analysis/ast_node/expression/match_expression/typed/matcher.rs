use crate::{
    error::{err, ok},
    language::{ty, Literal},
    semantic_analysis::{
        ast_node::expression::typed_expression::{
            instantiate_struct_field_access, instantiate_tuple_index_access,
            instantiate_unsafe_downcast,
        },
        TypeCheckContext,
    },
    CompileResult, Ident, TypeId,
};

use sway_types::span::Span;

/// List of requirements that a desugared if expression must include in the conditional.
pub(crate) type MatchReqMap = Vec<(ty::TyExpression, ty::TyExpression)>;
/// List of variable declarations that must be placed inside of the body of the if expression.
pub(crate) type MatchDeclMap = Vec<(Ident, ty::TyExpression)>;
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
    ctx: TypeCheckContext,
    exp: &ty::TyExpression,
    scrutinee: ty::TyScrutinee,
) -> CompileResult<MatcherResult> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let ty::TyScrutinee {
        variant,
        type_id,
        span,
    } = scrutinee;

    let type_engine = ctx.type_engine;
    let declaration_engine = ctx.declaration_engine;

    // unify the type of the scrutinee with the type of the expression
    append!(
        type_engine.unify(declaration_engine, type_id, exp.return_type, &span, ""),
        warnings,
        errors
    );

    if !errors.is_empty() {
        return err(warnings, errors);
    }

    match variant {
        ty::TyScrutineeVariant::CatchAll => ok((vec![], vec![]), warnings, errors),
        ty::TyScrutineeVariant::Literal(value) => match_literal(exp, value, span),
        ty::TyScrutineeVariant::Variable(name) => match_variable(exp, name),
        ty::TyScrutineeVariant::Constant(name, _, type_id) => {
            match_constant(exp, name, type_id, span)
        }
        ty::TyScrutineeVariant::StructScrutinee(_, fields) => match_struct(ctx, exp, fields),
        ty::TyScrutineeVariant::EnumScrutinee { value, variant, .. } => {
            match_enum(ctx, exp, variant, *value, span)
        }
        ty::TyScrutineeVariant::Tuple(elems) => match_tuple(ctx, exp, elems, span),
    }
}

fn match_literal(
    exp: &ty::TyExpression,
    scrutinee: Literal,
    span: Span,
) -> CompileResult<MatcherResult> {
    let match_req_map = vec![(
        exp.to_owned(),
        ty::TyExpression {
            expression: ty::TyExpressionVariant::Literal(scrutinee),
            return_type: exp.return_type,
            span,
        },
    )];
    let match_decl_map = vec![];
    ok((match_req_map, match_decl_map), vec![], vec![])
}

fn match_variable(exp: &ty::TyExpression, scrutinee_name: Ident) -> CompileResult<MatcherResult> {
    let match_req_map = vec![];
    let match_decl_map = vec![(scrutinee_name, exp.to_owned())];

    ok((match_req_map, match_decl_map), vec![], vec![])
}

fn match_constant(
    exp: &ty::TyExpression,
    scrutinee_name: Ident,
    scrutinee_type_id: TypeId,
    span: Span,
) -> CompileResult<MatcherResult> {
    let match_req_map = vec![(
        exp.to_owned(),
        ty::TyExpression {
            expression: ty::TyExpressionVariant::VariableExpression {
                name: scrutinee_name,
                span: span.clone(),
                mutability: ty::VariableMutability::Immutable,
            },
            return_type: scrutinee_type_id,
            span,
        },
    )];
    let match_decl_map = vec![];

    ok((match_req_map, match_decl_map), vec![], vec![])
}

fn match_struct(
    mut ctx: TypeCheckContext,
    exp: &ty::TyExpression,
    fields: Vec<ty::TyStructScrutineeField>,
) -> CompileResult<MatcherResult> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut match_req_map = vec![];
    let mut match_decl_map = vec![];
    for ty::TyStructScrutineeField {
        field,
        scrutinee,
        span: field_span,
    } in fields.into_iter()
    {
        let subfield = check!(
            instantiate_struct_field_access(
                ctx.type_engine,
                exp.clone(),
                field.clone(),
                field_span
            ),
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
                    matcher(ctx.by_ref(), &subfield, scrutinee),
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
    ctx: TypeCheckContext,
    exp: &ty::TyExpression,
    variant: ty::TyEnumVariant,
    scrutinee: ty::TyScrutinee,
    span: Span,
) -> CompileResult<MatcherResult> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let (mut match_req_map, unsafe_downcast) =
        instantiate_unsafe_downcast(ctx.type_engine, exp, variant, span);
    let (mut new_match_req_map, match_decl_map) = check!(
        matcher(ctx, &unsafe_downcast, scrutinee),
        return err(warnings, errors),
        warnings,
        errors
    );
    match_req_map.append(&mut new_match_req_map);
    ok((match_req_map, match_decl_map), warnings, errors)
}

fn match_tuple(
    mut ctx: TypeCheckContext,
    exp: &ty::TyExpression,
    elems: Vec<ty::TyScrutinee>,
    span: Span,
) -> CompileResult<MatcherResult> {
    let mut warnings = vec![];
    let mut errors = vec![];
    let mut match_req_map = vec![];
    let mut match_decl_map = vec![];
    for (pos, elem) in elems.into_iter().enumerate() {
        let tuple_index_access = check!(
            instantiate_tuple_index_access(
                ctx.type_engine,
                exp.clone(),
                pos,
                span.clone(),
                span.clone()
            ),
            return err(warnings, errors),
            warnings,
            errors
        );
        let (mut new_match_req_map, mut new_match_decl_map) = check!(
            matcher(ctx.by_ref(), &tuple_index_access, elem),
            return err(warnings, errors),
            warnings,
            errors
        );
        match_req_map.append(&mut new_match_req_map);
        match_decl_map.append(&mut new_match_decl_map);
    }
    ok((match_req_map, match_decl_map), warnings, errors)
}
