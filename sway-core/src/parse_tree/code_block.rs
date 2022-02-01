use super::WhileLoop;
use crate::{
    build_config::BuildConfig,
    error::*,
    parse_tree::{Expression, ReturnStatement},
    parser::Rule,
    AstNode, AstNodeContent, Declaration,
};

use sway_types::span;

use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub contents: Vec<AstNode>,
    pub(crate) whole_block_span: span::Span,
}

impl CodeBlock {
    pub(crate) fn parse_from_pair(
        block: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let whole_block_span = span::Span {
            span: block.as_span(),
            path: path.clone(),
        };
        let block_inner = block.into_inner();
        let mut contents = Vec::new();
        for pair in block_inner {
            let mut pair_contents: Vec<AstNode> = match pair.as_rule() {
                Rule::declaration => {
                    let span = span::Span {
                        span: pair.as_span(),
                        path: path.clone(),
                    };
                    check!(
                        Declaration::parse_from_pair(pair.clone(), config),
                        continue,
                        warnings,
                        errors
                    )
                    .into_iter()
                    .map(|x| AstNode {
                        content: AstNodeContent::Declaration(x),
                        span: span.clone(),
                    })
                    .collect::<Vec<_>>()
                }
                Rule::expr_statement => {
                    let evaluated_node_result = check!(
                        Expression::parse_from_pair(
                            pair.clone().into_inner().next().unwrap().clone(),
                            config
                        ),
                        continue,
                        warnings,
                        errors
                    );
                    let mut ast_node_contents = evaluated_node_result
                        .var_decls
                        .into_iter()
                        .map(|x| AstNode {
                            content: AstNodeContent::Declaration(Declaration::VariableDeclaration(
                                x,
                            )),
                            span: span::Span {
                                span: pair.as_span(),
                                path: path.clone(),
                            },
                        })
                        .collect::<Vec<_>>();
                    ast_node_contents.push(AstNode {
                        content: AstNodeContent::Expression(evaluated_node_result.value),
                        span: span::Span {
                            span: pair.as_span(),
                            path: path.clone(),
                        },
                    });
                    ast_node_contents
                }
                Rule::return_statement => {
                    let res_result = check!(
                        ReturnStatement::parse_from_pair(pair.clone(), config),
                        continue,
                        warnings,
                        errors
                    );
                    let mut ast_node_contents = res_result
                        .var_decls
                        .into_iter()
                        .map(|x| AstNode {
                            content: AstNodeContent::Declaration(Declaration::VariableDeclaration(
                                x,
                            )),
                            span: span::Span {
                                span: pair.as_span(),
                                path: path.clone(),
                            },
                        })
                        .collect::<Vec<_>>();
                    ast_node_contents.push(AstNode {
                        content: AstNodeContent::ReturnStatement(res_result.value),
                        span: span::Span {
                            span: pair.as_span(),
                            path: path.clone(),
                        },
                    });
                    ast_node_contents
                }
                Rule::expr => {
                    let res_result = check!(
                        Expression::parse_from_pair(pair.clone(), config),
                        continue,
                        warnings,
                        errors
                    );
                    let mut ast_node_contents = res_result
                        .var_decls
                        .into_iter()
                        .map(|x| AstNode {
                            content: AstNodeContent::Declaration(Declaration::VariableDeclaration(
                                x,
                            )),
                            span: span::Span {
                                span: pair.as_span(),
                                path: path.clone(),
                            },
                        })
                        .collect::<Vec<_>>();
                    ast_node_contents.push(AstNode {
                        content: AstNodeContent::ImplicitReturnExpression(res_result.value.clone()),
                        span: res_result.value.span(),
                    });
                    ast_node_contents
                }
                Rule::while_loop => {
                    let res_result = check!(
                        WhileLoop::parse_from_pair(pair.clone(), config),
                        continue,
                        warnings,
                        errors
                    );
                    let mut ast_node_contents = res_result
                        .var_decls
                        .into_iter()
                        .map(|x| AstNode {
                            content: AstNodeContent::Declaration(Declaration::VariableDeclaration(
                                x,
                            )),
                            span: span::Span {
                                span: pair.as_span(),
                                path: path.clone(),
                            },
                        })
                        .collect::<Vec<_>>();
                    ast_node_contents.push(AstNode {
                        content: AstNodeContent::WhileLoop(res_result.value),
                        span: span::Span {
                            span: pair.as_span(),
                            path: path.clone(),
                        },
                    });
                    ast_node_contents
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
            contents.append(&mut pair_contents);
        }

        ok(
            CodeBlock {
                whole_block_span,
                contents,
            },
            warnings,
            errors,
        )
    }
}
