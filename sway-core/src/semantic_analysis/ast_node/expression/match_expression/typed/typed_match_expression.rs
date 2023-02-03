use sway_types::Span;

use crate::{
    error::{err, ok},
    language::{parsed::*, ty, *},
    semantic_analysis::{
        ast_node::expression::typed_expression::{
            instantiate_if_expression, instantiate_lazy_operator,
        },
        TypeCheckContext,
    },
    CompileError, CompileResult, TypeInfo,
};

impl ty::TyMatchExpression {
    pub(crate) fn type_check(
        ctx: TypeCheckContext,
        typed_value: ty::TyExpression,
        branches: Vec<MatchBranch>,
        span: Span,
    ) -> CompileResult<(ty::TyMatchExpression, Vec<ty::TyScrutinee>)> {
        let mut warnings = vec![];
        let mut errors = vec![];

        // type check all of the branches
        let mut typed_branches = vec![];
        let mut typed_scrutinees = vec![];
        let mut ctx =
            ctx.with_help_text("all branches of a match statement must return the same type");
        for branch in branches.into_iter() {
            let (typed_branch, typed_scrutinee) = check!(
                ty::TyMatchBranch::type_check(ctx.by_ref(), &typed_value, branch),
                continue,
                warnings,
                errors
            );
            typed_branches.push(typed_branch);
            typed_scrutinees.push(typed_scrutinee);
        }

        if !errors.is_empty() {
            return err(warnings, errors);
        }

        let typed_exp = ty::TyMatchExpression {
            value_type_id: typed_value.return_type,
            branches: typed_branches,
            return_type_id: ctx.type_annotation(),
            span,
        };
        ok((typed_exp, typed_scrutinees), warnings, errors)
    }

    pub(crate) fn convert_to_typed_if_expression(
        self,
        mut ctx: TypeCheckContext,
    ) -> CompileResult<ty::TyExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let type_engine = ctx.type_engine;
        let decl_engine = ctx.decl_engine;

        // create the typed if expression object that we will be building on to
        let mut typed_if_exp: Option<ty::TyExpression> = None;

        // for every branch of the match expression, in reverse
        for ty::TyMatchBranch {
            conditions, result, ..
        } in self.branches.into_iter().rev()
        {
            // create the conditional that will act as the conditional for the if statement, in reverse
            let mut conditional: Option<ty::TyExpression> = None;
            for (left_req, right_req) in conditions.into_iter().rev() {
                let joined_span = Span::join(left_req.span.clone(), right_req.span.clone());
                let args = vec![left_req, right_req];
                let new_condition = check!(
                    ty::TyExpression::core_ops_eq(ctx.by_ref(), args, joined_span),
                    continue,
                    warnings,
                    errors
                );
                conditional = Some(match conditional {
                    Some(inner_condition) => {
                        let joined_span =
                            Span::join(inner_condition.span.clone(), new_condition.span.clone());
                        instantiate_lazy_operator(
                            LazyOp::And,
                            new_condition,
                            inner_condition,
                            type_engine.insert(decl_engine, TypeInfo::Boolean),
                            joined_span,
                        )
                    }
                    None => new_condition,
                });
            }

            // add to the if expression that we are building using the result component
            // of the match branch and using the conditional that we just built
            let result_span = result.span.clone();
            typed_if_exp = Some(match (typed_if_exp.clone(), conditional) {
                (None, None) => result,
                (None, Some(conditional)) => {
                    // TODO: figure out if this argument matters or not
                    let ctx = ctx.by_ref().with_type_annotation(self.return_type_id);
                    check!(
                        instantiate_if_expression(
                            ctx,
                            conditional,
                            result.clone(),
                            Some(result), // TODO: this is a really bad hack and we should not do this
                            result_span,
                        ),
                        continue,
                        warnings,
                        errors
                    )
                }
                (Some(prev_if_exp), None) => {
                    let ctx = ctx.by_ref().with_type_annotation(self.return_type_id);
                    let conditional = ty::TyExpression {
                        expression: ty::TyExpressionVariant::Literal(Literal::Boolean(true)),
                        return_type: type_engine.insert(decl_engine, TypeInfo::Boolean),
                        span: result_span.clone(),
                    };
                    check!(
                        instantiate_if_expression(
                            ctx,
                            conditional,
                            result,
                            Some(prev_if_exp),
                            result_span,
                        ),
                        continue,
                        warnings,
                        errors
                    )
                }
                (Some(prev_if_exp), Some(conditional)) => {
                    let ctx = ctx.by_ref().with_type_annotation(self.return_type_id);
                    check!(
                        instantiate_if_expression(
                            ctx,
                            conditional,
                            result,
                            Some(prev_if_exp),
                            result_span,
                        ),
                        continue,
                        warnings,
                        errors
                    )
                }
            });
        }

        // return!
        match typed_if_exp {
            None => {
                // If the type that we are matching on does not have a valid
                // constructor, then it is expected that this algorithm finds a
                // "None". This is because the user has not provided any
                // branches in the match expression because the type cannot be
                // constructed or matched upon. In this case, we manually create
                // a typed expression that is equivalent to
                // "if true { implicit_return }" where the implicit_return type is manually set
                // to be the return type of this typed match expression object.
                //
                // NOTE: This manual construction of the expression can (and
                // most likely will) lead to an otherwise improperly typed
                // expression, in most cases.
                if !type_engine.get(self.value_type_id).has_valid_constructor() {
                    let condition = ty::TyExpression {
                        expression: ty::TyExpressionVariant::Literal(Literal::Boolean(true)),
                        return_type: type_engine.insert(decl_engine, TypeInfo::Boolean),
                        span: self.span.clone(),
                    };
                    let then_exp = ty::TyExpression {
                        expression: ty::TyExpressionVariant::Tuple { fields: vec![] },
                        return_type: self.return_type_id,
                        span: self.span.clone(),
                    };
                    let inner_exp = ty::TyExpressionVariant::IfExp {
                        condition: Box::new(condition),
                        then: Box::new(then_exp.clone()),
                        r#else: Option::Some(Box::new(then_exp)),
                    };
                    let typed_if_exp = ty::TyExpression {
                        expression: inner_exp,
                        return_type: self.return_type_id,
                        span: self.span,
                    };
                    return ok(typed_if_exp, warnings, errors);
                }

                errors.push(CompileError::Internal(
                    "unable to convert match exp to if exp",
                    self.span,
                ));
                err(warnings, errors)
            }
            Some(typed_if_exp) => ok(typed_if_exp, warnings, errors),
        }
    }
}
