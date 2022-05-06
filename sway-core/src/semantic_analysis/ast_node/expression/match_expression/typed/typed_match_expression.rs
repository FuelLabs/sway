use sway_types::Span;

use crate::{
    error::{err, ok},
    semantic_analysis::{
        ast_node::expression::typed_expression::{
            instantiate_if_expression, instantiate_lazy_operator,
        },
        IsConstant, TypeCheckArguments, TypedExpression, TypedExpressionVariant,
    },
    type_engine::{insert_type, TypeId},
    CompileResult, LazyOp, Literal, MatchBranch, NamespaceRef, Scrutinee, TypeInfo,
};

use super::typed_match_branch::TypedMatchBranch;

#[derive(Debug)]
pub(crate) struct TypedMatchExpression {
    #[allow(dead_code)]
    value: TypedExpression,
    branches: Vec<TypedMatchBranch>,
    #[allow(dead_code)]
    span: Span,
}

impl TypedMatchExpression {
    pub(crate) fn type_check(
        arguments: TypeCheckArguments<'_, (TypedExpression, Vec<MatchBranch>)>,
        span: Span,
    ) -> CompileResult<(Self, Vec<Scrutinee>)> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let TypeCheckArguments {
            checkee: (typed_value, branches),
            namespace,
            crate_namespace,
            return_type_annotation,
            self_type,
            build_config,
            dead_code_graph,
            opts,
            help_text,
            mode,
        } = arguments;

        // type check all of the branches
        let mut typed_branches = vec![];
        let mut scrutinees = vec![];
        match branches.split_first() {
            None => unimplemented!(),
            Some((first_branch, rest_of_branches)) => {
                // type check the first branch first, so that we can use the return type
                // from the first branch to type check the other branches
                let (typed_first_branch, scrutinee) = check!(
                    TypedMatchBranch::type_check(TypeCheckArguments {
                        checkee: (&typed_value, first_branch.clone()),
                        namespace,
                        crate_namespace,
                        return_type_annotation,
                        help_text,
                        self_type,
                        build_config,
                        dead_code_graph,
                        mode,
                        opts,
                    }),
                    return err(warnings, errors),
                    warnings,
                    errors
                );
                let new_return_type_annotation = typed_first_branch.result.return_type;
                typed_branches.push(typed_first_branch);
                scrutinees.push(scrutinee);
                // type check the rest of the branches
                for branch in rest_of_branches.iter() {
                    let (typed_branch, scrutinee) = check!(
                        TypedMatchBranch::type_check(TypeCheckArguments {
                            checkee: (&typed_value, branch.clone()),
                            namespace,
                            crate_namespace,
                            return_type_annotation: new_return_type_annotation,
                            help_text:
                                "all branches of a match statement must return the same type",
                            self_type,
                            build_config,
                            dead_code_graph,
                            mode,
                            opts,
                        }),
                        continue,
                        warnings,
                        errors
                    );
                    typed_branches.push(typed_branch);
                    scrutinees.push(scrutinee);
                }
            }
        }

        let exp = TypedMatchExpression {
            value: typed_value,
            branches: typed_branches,
            span,
        };
        ok((exp, scrutinees), warnings, errors)
    }

    pub(crate) fn convert_to_typed_if_expression(
        self,
        namespace: NamespaceRef,
        crate_namespace: NamespaceRef,
        self_type: TypeId,
    ) -> CompileResult<TypedExpression> {
        let mut warnings = vec![];
        let mut errors = vec![];

        let TypedMatchExpression { branches, .. } = self;

        // create the typed if expression object that we will be building on to
        let mut typed_if_exp: Option<TypedExpression> = None;

        // for every branch of the match expression, in reverse
        for TypedMatchBranch {
            conditions, result, ..
        } in branches.into_iter().rev()
        {
            // create the conditional that will act as the conditional for the if statement, in reverse
            let mut conditional: Option<TypedExpression> = None;
            for (left_req, right_req) in conditions.into_iter().rev() {
                let joined_span = Span::join(left_req.span.clone(), right_req.span.clone());
                let new_condition = check!(
                    TypedExpression::core_ops_eq(
                        vec![left_req, right_req],
                        joined_span,
                        namespace,
                        crate_namespace,
                        self_type
                    ),
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
                            insert_type(TypeInfo::Boolean),
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
                    check!(
                        instantiate_if_expression(
                            conditional,
                            result.clone(),
                            Some(result), // TODO: this is a really bad hack and we should not do this
                            result_span,
                            insert_type(TypeInfo::Unknown), // TODO: figure out if this argument matters or not
                            self_type
                        ),
                        continue,
                        warnings,
                        errors
                    )
                }
                (Some(prev_if_exp), None) => {
                    let conditional = TypedExpression {
                        expression: TypedExpressionVariant::Literal(Literal::Boolean(true)),
                        return_type: insert_type(TypeInfo::Boolean),
                        is_constant: IsConstant::No,
                        span: result_span.clone(),
                    };
                    let return_type = prev_if_exp.return_type;
                    check!(
                        instantiate_if_expression(
                            conditional,
                            result,
                            Some(prev_if_exp),
                            result_span,
                            return_type,
                            self_type
                        ),
                        continue,
                        warnings,
                        errors
                    )
                }
                (Some(prev_if_exp), Some(conditional)) => {
                    let return_type = prev_if_exp.return_type;
                    check!(
                        instantiate_if_expression(
                            conditional,
                            result,
                            Some(prev_if_exp),
                            result_span,
                            return_type,
                            self_type
                        ),
                        continue,
                        warnings,
                        errors
                    )
                }
            });
        }

        // return!
        let typed_if_exp = match typed_if_exp {
            None => unimplemented!(),
            Some(typed_if_exp) => typed_if_exp,
        };
        ok(typed_if_exp, warnings, errors)
    }
}
