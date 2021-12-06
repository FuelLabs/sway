use crate::{Op, OpVariant};
use crate::utils::join_spans;
use crate::{Expression, MatchBranch, Span, MatchCondition, VariableDeclaration, Declaration, Ident, TypeInfo, AstNode, AstNodeContent, CodeBlock, Literal, MethodName, CallPath};
use crate::error::{err, ok, CompileResult};
use super::code_block::desugar_code_block;
use super::matcher::matcher;

pub fn desugar_expression<'sc>(exp: Expression<'sc>) -> CompileResult<'sc, Expression<'sc>> {
    let mut warnings = vec!();
    let mut errors = vec!();
    match exp {
        Expression::MatchExpression { primary_expression, branches, span } => desugar_match_expression(&*primary_expression, branches, span),
        Expression::VariableExpression { name, span } => {
            let exp = Expression::VariableExpression { name, span };
            ok(exp, warnings, errors)
        },
        Expression::Literal { value, span } => {
            let exp = Expression::Literal { value, span };
            ok(exp, warnings, errors)
        },
        Expression::IfExp { condition, then, r#else , span } => {
            let condition = check!(
                desugar_expression(*condition),
                return err(warnings, errors),
                warnings,
                errors
            );
            let then = check!(
                desugar_expression(*then),
                return err(warnings, errors),
                warnings,
                errors
            );
            let r#else = match r#else {
                None => None,
                Some(r#else) => Some(Box::new(check!(
                    desugar_expression(*r#else),
                    return err(warnings, errors),
                    warnings,
                    errors
                )))
            };
            let exp = Expression::IfExp {
                condition: Box::new(condition),
                then: Box::new(then),
                r#else,
                span
            };
            ok(exp, warnings, errors)
        },
        Expression::MethodApplication { method_name, arguments, span } => {
            let mut new_arguments = vec![];
            for argument in arguments.into_iter() {
                new_arguments.push(check!(
                    desugar_expression(argument),
                    return err(warnings, errors),
                    warnings,
                    errors
                ));
            }
            let exp = Expression::MethodApplication {
                method_name,
                arguments: new_arguments,
                span
            };
            ok(exp, warnings, errors)
        },
        Expression::CodeBlock { contents, span } => {
            let exp = Expression::CodeBlock {
                contents: check!(
                    desugar_code_block(contents),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
                span
            };
            ok(exp, warnings, errors)
        }
        exp => unimplemented!("{:?}", exp)
    }
}

struct MatchedBranch<'sc> {
    result: Expression<'sc>,
    match_req_map: Vec<(Expression<'sc>, Expression<'sc>)>,
    match_impl_map: Vec<(Ident<'sc>, Expression<'sc>)>,
    branch_span: Span<'sc>
}

pub fn desugar_match_expression<'sc>(primary_expression: &Expression<'sc>, branches: Vec<MatchBranch<'sc>>, _span: Span<'sc>) -> CompileResult<'sc, Expression<'sc>> {
    let mut warnings = vec![];
    let mut errors = vec![];

    // 1. Assemble the "matched branches."
    let mut matched_branches = vec![];
    for MatchBranch { condition, result, span: branch_span } in branches.iter() {
        let matches = match condition {
            MatchCondition::CatchAll(_) => Some((vec![], vec![])),
            MatchCondition::Scrutinee(scrutinee) => matcher(primary_expression, scrutinee)
        };
        match matches {
            Some((match_req_map, match_impl_map)) => {
                matched_branches.push(MatchedBranch {
                    result: check!(
                        desugar_expression(result.to_owned()),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ),
                    match_req_map,
                    match_impl_map,
                    branch_span: branch_span.to_owned()
                });
            }
            None => unimplemented!("implement proper error handling"),
        }
    }

    // 2. Assemble the possibly nested giant if statement using the matched branches.
    let mut if_statement = None;
    for MatchedBranch { result, match_req_map, match_impl_map, branch_span } in matched_branches.iter().rev() {
        // 2a. Assemble the conditional that goes in the if primary expression.
        let mut conditional = None;
        for (left_req, right_req) in match_req_map.iter() {
            let joined_span = join_spans(left_req.clone().span(), right_req.clone().span());
            let condition = Expression::MethodApplication {
                method_name: MethodName::FromType {
                    call_path: CallPath {
                        prefixes: vec![
                            Ident {
                                primary_name: "std",
                                span: joined_span.clone(),
                            },
                            Ident {
                                primary_name: "ops",
                                span: joined_span.clone(),
                            },
                        ],
                        suffix: Op {
                            op_variant: OpVariant::Equals,
                            span: joined_span.clone()
                        }.to_var_name(),
                    },
                    type_name: None,
                    is_absolute: true,
                },
                arguments: vec![left_req.to_owned(), right_req.to_owned()],
                span: joined_span,
            };
            match conditional {
                None => {
                    conditional = Some(condition);
                }
                Some(the_conditional) => {
                    conditional = Some(Expression::LazyOperator {
                        op: crate::LazyOp::And,
                        lhs: Box::new(the_conditional.clone()),
                        rhs: Box::new(condition.clone()),
                        span: join_spans(the_conditional.span(), condition.span())
                    });
                }
            }
        }

        // 2b. Assemble the statements that go inside of the body of the if expression
        let mut code_block_stmts = vec![];
        let mut code_block_stmts_span = None;
        for (left_impl, right_impl) in match_impl_map.iter() {
            let decl = Declaration::VariableDeclaration(VariableDeclaration {
                name: left_impl.clone(),
                is_mutable: false,
                body: right_impl.clone(),
                type_ascription: TypeInfo::Unknown,
                type_ascription_span: None
            });
            let new_span = join_spans(left_impl.span.clone(), right_impl.span());
            code_block_stmts.push(AstNode {
                content: AstNodeContent::Declaration(decl),
                span: new_span.clone()
            });
            code_block_stmts_span = match code_block_stmts_span {
                None => Some(new_span),
                Some(old_span) => Some(join_spans(old_span, new_span))
            };
        }
        match result {
            Expression::CodeBlock {
                contents: CodeBlock {
                    contents,
                    whole_block_span
                },
                span:_
            } => {
                let mut contents = contents.clone();
                code_block_stmts.append(&mut contents);
                code_block_stmts_span = match code_block_stmts_span {
                    None => Some(whole_block_span.clone()),
                    Some(old_span) => Some(join_spans(old_span, whole_block_span.clone()))
                };
            },
            result => {
                code_block_stmts.push(AstNode {
                    content: AstNodeContent::Expression(result.clone()),
                    span: result.span()
                });
                code_block_stmts_span = match code_block_stmts_span {
                    None => Some(result.span()),
                    Some(old_span) => Some(join_spans(old_span, result.span()))
                };
            }
        }
        let code_block_stmts_span = match code_block_stmts_span {
            None => branch_span.clone(),
            Some(span) => span
        };
        let code_block = Expression::CodeBlock {
            contents: CodeBlock {
                contents: code_block_stmts.clone(),
                whole_block_span: code_block_stmts_span.clone()
            },
            span: code_block_stmts_span
        };

        // 2c. Assemble the giant if statement.
        match if_statement {
            None => {
                if_statement = match conditional {
                    None => Some(code_block),
                    Some(conditional) => Some(Expression::IfExp {
                        condition: Box::new(conditional.clone()),
                        then: Box::new(code_block.clone()),
                        r#else: None,
                        span: join_spans(conditional.span(), code_block.span())
                    })
                };
            },
            Some(Expression::CodeBlock {
                contents: right_block,
                span: exp_span
            }) => {
                let right = Expression::CodeBlock {
                    contents: right_block,
                    span: exp_span
                };
                if_statement = match conditional {
                    None => Some(Expression::IfExp {
                        condition: Box::new(Expression::Literal {
                            value: Literal::Boolean(true),
                            span: branch_span.clone()
                        }),
                        then: Box::new(code_block.clone()),
                        r#else: Some(Box::new(right.clone())),
                        span: join_spans(code_block.clone().span(), right.clone().span())
                    }),
                    Some(the_conditional) => Some(Expression::IfExp {
                        condition: Box::new(the_conditional),
                        then: Box::new(code_block.clone()),
                        r#else: Some(Box::new(right.clone())),
                        span: join_spans(code_block.clone().span(), right.clone().span())
                    })
                };
            },
            Some(Expression::IfExp {
                condition,
                then,
                r#else,
                span: exp_span
            }) => {
                if_statement = Some(Expression::IfExp {
                    condition: Box::new(conditional.unwrap()),
                    then: Box::new(code_block.clone()),
                    r#else: Some(Box::new(Expression::IfExp {
                        condition,
                        then,
                        r#else,
                        span: exp_span.clone()
                    })),
                    span: join_spans(code_block.clone().span(), exp_span)
                });
            }
            _ => unimplemented!(),
        }
    }
    
    // 3. Return!
    match if_statement {
        None => err(warnings, errors),
        Some(if_statement) => ok(if_statement, warnings, errors),
    }
}