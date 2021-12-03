use crate::{Expression, MatchBranch, Span, MatchCondition, VariableDeclaration, Declaration, Ident, TypeInfo, AstNode, AstNodeContent, CodeBlock};
use crate::error::{err, ok, CompileResult};
use super::matcher::matcher;

pub fn desugar_expression<'sc>(exp: Expression<'sc>) -> CompileResult<'sc, Expression<'sc>> {
    match exp {
        Expression::MatchExpression { primary_expression, branches, span } => desugar_match_expression(&*primary_expression, branches, span),
        exp => unimplemented!("{:?}", exp)
    }
}

struct MatchedBranch<'sc> {
    result: Expression<'sc>,
    match_req_map: Vec<(Expression<'sc>, Expression<'sc>)>,
    match_impl_map: Vec<(Ident<'sc>, Expression<'sc>)>,
    span: Span<'sc>
}

pub fn desugar_match_expression<'sc>(primary_expression: &Expression<'sc>, branches: Vec<MatchBranch<'sc>>, span: Span<'sc>) -> CompileResult<'sc, Expression<'sc>> {
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
                    result: result.to_owned(),
                    match_req_map,
                    match_impl_map,
                    span: branch_span.to_owned()
                });
            }
            None => unimplemented!("implement proper error handling"),
        }
    }

    // 2. Assemble the possibly nested giant if statement using the matched branches.
    let mut if_statement = None;
    for MatchedBranch { result, match_req_map, match_impl_map, span: branch_span } in matched_branches.iter() {
        // 2a. Assemble the conditional that goes in the if primary expression.
        let mut conditional = None;
        for (left_req, right_req) in match_req_map.iter() {
            let condition = Expression::LazyOperator {
                op: crate::LazyOp::And,
                lhs: Box::new(left_req.clone()),
                rhs: Box::new(right_req.clone()),
                span: branch_span.to_owned(),
            };
            match conditional {
                None => {
                    conditional = Some(condition);
                }
                Some(the_conditional) => {
                    conditional = Some(Expression::LazyOperator {
                        op: crate::LazyOp::And,
                        lhs: Box::new(the_conditional),
                        rhs: Box::new(condition),
                        span: branch_span.to_owned()
                    });
                }
            }
        }
        println!("{:?}", conditional);

        // 2b. Assemble the statements that go inside of the body of the if expression
        let mut code_block_stmts = vec![];
        for (left_impl, right_impl) in match_impl_map.iter() {
            let decl = Declaration::VariableDeclaration(VariableDeclaration {
                name: left_impl.to_owned(),
                is_mutable: false,
                body: right_impl.clone(),
                type_ascription: TypeInfo::Unknown,
                type_ascription_span: None
            });
            code_block_stmts.push(AstNode {
                content: AstNodeContent::Declaration(decl),
                span: branch_span.clone()
            });
        }
        code_block_stmts.push(AstNode {
            content: AstNodeContent::Expression(result.to_owned()),
            span: branch_span.clone()
        });
        println!("{:#?}", code_block_stmts);

        // 2c. Assemble the giant if statement.
        match if_statement {
            None => unimplemented!(),
            Some(Expression::CodeBlock {
                contents: CodeBlock {
                    contents,
                    whole_block_span
                },
                span: exp_span
            }) => unimplemented!(),
            Some(Expression::IfExp {
                condition,
                then,
                r#else,
                span: exp_span
            }) => unimplemented!(),
            _ => unimplemented!(),
        }
    }
    
    // 3. Return!
    match if_statement {
        None => err(warnings, errors),
        Some(if_statement) => ok(if_statement, warnings, errors),
    }
}