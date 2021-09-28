use super::WhileLoop;
use crate::build_config::BuildConfig;
use crate::parser::Rule;
use crate::span::Span;
use crate::{
    error::*,
    parse_tree::{Expression, ReturnStatement},
    span, AstNode, AstNodeContent, Declaration,
};
use pest::iterators::Pair;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CodeBlock<'sc> {
    pub contents: Vec<AstNode<'sc>>,
    pub(crate) scope: HashMap<&'sc str, Declaration<'sc>>,
    pub(crate) whole_block_span: Span<'sc>,
}

impl<'sc> CodeBlock<'sc> {
    pub(crate) fn parse_from_pair(
        block: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
        docstrings: &mut HashMap<String, String>,
    ) -> CompileResult<'sc, Self> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let whole_block_span = span::Span {
            span: block.as_span(),
            path: path.clone(),
        };
        let block_inner = block.into_inner();
        let mut unassigned_docstring: String = "".to_string();
        let mut contents = Vec::new();
        for pair in block_inner {
            let content = match pair.as_rule() {
                Rule::declaration => {
                    let mut decl = pair.clone().into_inner();
                    let decl_inner = decl.next().unwrap();
                    match decl_inner.as_rule() {
                        Rule::docstring => {
                            let parts = decl_inner.clone().into_inner();
                            let docstring = parts.as_str().to_string().split_off(3);
                            let docstring = docstring.as_str().trim();
                            unassigned_docstring.push_str("\n");
                            unassigned_docstring.push_str(docstring);
                            None
                        }
                        _ => {
                            let decl_stmt = AstNode {
                                content: AstNodeContent::Declaration(check!(
                                    Declaration::parse_from_pair(
                                        pair.clone(),
                                        config,
                                        unassigned_docstring.clone(),
                                        docstrings
                                    ),
                                    continue,
                                    warnings,
                                    errors
                                )),
                                span: span::Span {
                                    span: pair.as_span(),
                                    path: path.clone(),
                                },
                            };
                            unassigned_docstring = "".to_string();
                            Some(decl_stmt)
                        }
                    }
                }
                Rule::expr_statement => {
                    let evaluated_node = check!(
                        Expression::parse_from_pair(
                            pair.clone().into_inner().next().unwrap().clone(),
                            config,
                            docstrings
                        ),
                        continue,
                        warnings,
                        errors
                    );
                    let expr_stmt = AstNode {
                        content: AstNodeContent::Expression(evaluated_node),
                        span: span::Span {
                            span: pair.as_span(),
                            path: path.clone(),
                        },
                    };
                    unassigned_docstring = "".to_string();
                    Some(expr_stmt)
                }
                Rule::return_statement => {
                    let evaluated_node = check!(
                        ReturnStatement::parse_from_pair(pair.clone(), config, docstrings),
                        continue,
                        warnings,
                        errors
                    );
                    let return_stmt = AstNode {
                        content: AstNodeContent::ReturnStatement(evaluated_node),
                        span: span::Span {
                            span: pair.as_span(),
                            path: path.clone(),
                        },
                    };
                    unassigned_docstring = "".to_string();
                    Some(return_stmt)
                }
                Rule::expr => {
                    let res = check!(
                        Expression::parse_from_pair(pair.clone(), config, docstrings),
                        continue,
                        warnings,
                        errors
                    );
                    let expr = AstNode {
                        content: AstNodeContent::ImplicitReturnExpression(res.clone()),
                        span: res.span(),
                    };
                    unassigned_docstring = "".to_string();
                    Some(expr)
                }
                Rule::while_loop => {
                    let res = check!(
                        WhileLoop::parse_from_pair(pair.clone(), config, docstrings),
                        continue,
                        warnings,
                        errors
                    );
                    let while_stmt = AstNode {
                        content: AstNodeContent::WhileLoop(res),
                        span: span::Span {
                            span: pair.as_span(),
                            path: path.clone(),
                        },
                    };
                    unassigned_docstring = "".to_string();
                    Some(while_stmt)
                }
                a => {
                    println!("In code block parsing: {:?} {:?}", a, pair.as_str());
                    errors.push(CompileError::UnimplementedRule(
                        a,
                        span::Span {
                            span: pair.as_span(),
                            path: path.clone(),
                        },
                    ));
                    continue;
                }
            };
            if let Some(content) = content {
                contents.push(content);
            }
        }

        ok(
            CodeBlock {  whole_block_span, contents, scope: /* TODO */ HashMap::default()  },
            warnings,
            errors,
        )
    }
}
