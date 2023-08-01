use crate::{
    language::{ty, CallPath, Literal},
    semantic_analysis::{
        ast_node::expression::typed_expression::{
            instantiate_struct_field_access, instantiate_tuple_index_access,
            instantiate_unsafe_downcast,
        },
        TypeCheckContext,
    },
    Ident, TypeId,
};

use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::span::Span;

use itertools::{EitherOrBoth, Itertools};

/// List of requirements that a desugared if expression must include in the conditional in conjunctive normal form.
pub(crate) type MatchReqMap = Vec<Vec<(ty::TyExpression, ty::TyExpression)>>;
/// List of variable declarations that must be placed inside of the body of the if expression.
pub(crate) type MatchDeclMap = Vec<(Ident, ty::TyExpression)>;
/// This is the result type given back by the matcher.
pub(crate) type MatcherResult = (MatchReqMap, MatchDeclMap);

/// This algorithm desugars pattern matching into a [MatcherResult], by creating two lists,
/// the [MatchReqMap] which is a list of requirements that a desugared if expression
/// must include in the conditional in conjunctive normal form.
/// and the [MatchImplMap] which is a list of variable
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
///     Point { x, y: 5 } | Point { x, y: 10 } => { x },
///     Point { x: 10, y: 24 } => { 10 },
///     _ => 0
/// }
/// ```
///
/// The first match arm would create a [MatchReqMap] of roughly:
///
/// ```ignore
/// [
///   [
///     (y, 5) // y must equal 5 to trigger this case
///   ]
/// ]
/// ```
///
/// The first match arm would create a [MatchImplMap] of roughly:
///
/// ```ignore
/// [
///     (x, 42) // add `let x = 42` in the body of the desugared if expression
/// ]
///
/// The second match arm would create a [MatchReqMap] of roughly:
///
/// ```ignore
/// // y must equal 5 or 10 to trigger this case
/// [
///   [
///     (y, 5)
///     (y, 10)
///   ],
/// ]
/// ```
///
/// The second match arm would create a [MatchImplMap] of roughly:
///
/// ```ignore
/// [
///     (x, 42) // add `let x = 42` in the body of the desugared if expression
/// ]
/// ```
///
/// The third match arm would create a [MatchReqMap] of roughly:
///
/// ```ignore
/// // x must equal 10 and y 24 to trigger this case
/// [
///   [
///     (x, 10),
///   ],
///   [
///     (y, 24),
///   ]
/// ]
/// ```
///
/// The third match arm would create a [MatchImplMap] of roughly:
///
/// ```ignore
/// []
/// ```
pub(crate) fn matcher(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    exp: &ty::TyExpression,
    scrutinee: ty::TyScrutinee,
) -> Result<MatcherResult, ErrorEmitted> {
    let ty::TyScrutinee {
        variant,
        type_id,
        span,
    } = scrutinee;

    let type_engine = ctx.engines.te();
    let engines = ctx.engines();

    // unify the type of the scrutinee with the type of the expression
    handler.scope(|h| {
        type_engine.unify(h, engines, type_id, exp.return_type, &span, "", None);
        Ok(())
    })?;

    match variant {
        ty::TyScrutineeVariant::Or(elems) => {
            let mut match_req_map: MatchReqMap = vec![];
            let mut match_decl_map: Option<MatchDeclMap> = None;
            for scrutinee in elems {
                let scrutinee_span = scrutinee.span.clone();

                let (new_req_map, mut new_decl_map) =
                    matcher(handler, ctx.by_ref(), exp, scrutinee)?;

                // check that the bindings are the same between clauses

                new_decl_map.sort_by(|(a, _), (b, _)| a.cmp(b));
                if let Some(match_decl_map) = match_decl_map {
                    for pair in match_decl_map.iter().zip_longest(new_decl_map.iter()) {
                        use EitherOrBoth::*;
                        let missing_var = match pair {
                            Both((l_ident, _), (r_ident, _)) => {
                                if l_ident == r_ident {
                                    None
                                } else {
                                    Some(l_ident)
                                }
                            }
                            Left((ident, _)) => Some(ident),
                            Right((ident, _)) => Some(ident),
                        };
                        if let Some(var) = missing_var {
                            return Err(handler.emit_err(
                                CompileError::MatchVariableNotBoundInAllPatterns {
                                    var: var.clone(),
                                    span: scrutinee_span,
                                },
                            ));
                        }
                    }
                }

                match_decl_map = Some(new_decl_map);
                match_req_map = factor_or_on_cnf(match_req_map, new_req_map);
            }
            Ok((match_req_map, match_decl_map.unwrap_or(vec![])))
        }
        ty::TyScrutineeVariant::CatchAll => Ok((vec![], vec![])),
        ty::TyScrutineeVariant::Literal(value) => Ok(match_literal(exp, value, span)),
        ty::TyScrutineeVariant::Variable(name) => Ok(match_variable(exp, name)),
        ty::TyScrutineeVariant::Constant(name, _, const_decl) => Ok(match_constant(
            ctx,
            exp,
            name,
            const_decl.type_ascription.type_id,
            span,
        )),
        ty::TyScrutineeVariant::StructScrutinee {
            struct_ref: _,
            fields,
            ..
        } => match_struct(handler, ctx, exp, fields),
        ty::TyScrutineeVariant::EnumScrutinee {
            enum_ref: _,
            variant,
            call_path_decl,
            value,
            ..
        } => match_enum(handler, ctx, exp, *variant, call_path_decl, *value, span),
        ty::TyScrutineeVariant::Tuple(elems) => match_tuple(handler, ctx, exp, elems, span),
    }
}

fn factor_or_on_cnf(a: MatchReqMap, b: MatchReqMap) -> MatchReqMap {
    if a.is_empty() {
        return b;
    }
    if b.is_empty() {
        return a;
    }
    let mut res = vec![];
    for a_disj in a.iter() {
        for mut b_disj in b.clone() {
            b_disj.append(&mut a_disj.clone());
            res.push(b_disj);
        }
    }
    res
}

fn match_literal(exp: &ty::TyExpression, scrutinee: Literal, span: Span) -> MatcherResult {
    let match_req_map = vec![vec![(
        exp.to_owned(),
        ty::TyExpression {
            expression: ty::TyExpressionVariant::Literal(scrutinee),
            return_type: exp.return_type,
            span,
        },
    )]];
    let match_decl_map = vec![];
    (match_req_map, match_decl_map)
}

fn match_variable(exp: &ty::TyExpression, scrutinee_name: Ident) -> MatcherResult {
    let match_req_map = vec![vec![]];
    let match_decl_map = vec![(scrutinee_name, exp.to_owned())];

    (match_req_map, match_decl_map)
}

fn match_constant(
    ctx: TypeCheckContext,
    exp: &ty::TyExpression,
    scrutinee_name: Ident,
    scrutinee_type_id: TypeId,
    span: Span,
) -> MatcherResult {
    let match_req_map = vec![vec![(
        exp.to_owned(),
        ty::TyExpression {
            expression: ty::TyExpressionVariant::VariableExpression {
                name: scrutinee_name.clone(),
                span: span.clone(),
                mutability: ty::VariableMutability::Immutable,
                call_path: Some(CallPath::from(scrutinee_name).to_fullpath(ctx.namespace)),
            },
            return_type: scrutinee_type_id,
            span,
        },
    )]];
    let match_decl_map = vec![];

    (match_req_map, match_decl_map)
}

fn match_struct(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    exp: &ty::TyExpression,
    fields: Vec<ty::TyStructScrutineeField>,
) -> Result<MatcherResult, ErrorEmitted> {
    let mut match_req_map = vec![];
    let mut match_decl_map = vec![];
    for ty::TyStructScrutineeField {
        field,
        scrutinee,
        span: field_span,
        field_def_name: _,
    } in fields.into_iter()
    {
        let subfield = instantiate_struct_field_access(
            handler,
            ctx.engines(),
            exp.clone(),
            field.clone(),
            field_span,
        )?;
        match scrutinee {
            // if the scrutinee is simply naming the struct field ...
            None => {
                match_decl_map.push((field, subfield));
            }
            // or if the scrutinee has a more complex agenda
            Some(scrutinee) => {
                let (mut new_match_req_map, mut new_match_decl_map) =
                    matcher(handler, ctx.by_ref(), &subfield, scrutinee)?;
                match_req_map.append(&mut new_match_req_map);
                match_decl_map.append(&mut new_match_decl_map);
            }
        }
    }

    Ok((match_req_map, match_decl_map))
}

fn match_enum(
    handler: &Handler,
    ctx: TypeCheckContext,
    exp: &ty::TyExpression,
    variant: ty::TyEnumVariant,
    call_path_decl: ty::TyDecl,
    scrutinee: ty::TyScrutinee,
    span: Span,
) -> Result<MatcherResult, ErrorEmitted> {
    let (mut match_req_map, unsafe_downcast) =
        instantiate_unsafe_downcast(ctx.engines(), exp, variant, call_path_decl, span);
    let (mut new_match_req_map, match_decl_map) =
        matcher(handler, ctx, &unsafe_downcast, scrutinee)?;
    match_req_map.append(&mut new_match_req_map);
    Ok((match_req_map, match_decl_map))
}

fn match_tuple(
    handler: &Handler,
    mut ctx: TypeCheckContext,
    exp: &ty::TyExpression,
    elems: Vec<ty::TyScrutinee>,
    span: Span,
) -> Result<MatcherResult, ErrorEmitted> {
    let mut match_req_map = vec![];
    let mut match_decl_map = vec![];
    for (pos, elem) in elems.into_iter().enumerate() {
        let tuple_index_access = instantiate_tuple_index_access(
            handler,
            ctx.engines(),
            exp.clone(),
            pos,
            span.clone(),
            span.clone(),
        )?;
        let (mut new_match_req_map, mut new_match_decl_map) =
            matcher(handler, ctx.by_ref(), &tuple_index_access, elem)?;
        match_req_map.append(&mut new_match_req_map);
        match_decl_map.append(&mut new_match_decl_map);
    }
    Ok((match_req_map, match_decl_map))
}
