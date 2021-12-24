use super::WhileLoop;
use crate::build_config::BuildConfig;
use crate::parser::Rule;
use crate::span::Span;
use crate::{
    error::*,
    parse_tree::{Expression, ReturnStatement},
    span, AstNode, AstNodeContent, Declaration,
};
use fuel_pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct CodeBlock<'sc> {
    pub contents: Vec<AstNode<'sc>>,
    pub(crate) whole_block_span: Span<'sc>,
}

impl<'sc> CodeBlock<'sc> {
    pub(crate) fn parse_from_pair(
        block: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
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
            contents.push(match pair.as_rule() {
                Rule::declaration => AstNode {
                    content: AstNodeContent::Declaration(check!(
                        Declaration::parse_from_pair(pair.clone(), config),
                        continue,
                        warnings,
                        errors
                    )),
                    span: span::Span {
                        span: pair.as_span(),
                        path: path.clone(),
                    },
                },
                Rule::expr_statement => {
                    let evaluated_node = check!(
                        Expression::parse_from_pair(
                            pair.clone().into_inner().next().unwrap().clone(),
                            config
                        ),
                        continue,
                        warnings,
                        errors
                    );
                    AstNode {
                        content: AstNodeContent::Expression(evaluated_node),
                        span: span::Span {
                            span: pair.as_span(),
                            path: path.clone(),
                        },
                    }
                }
                Rule::return_statement => {
                    let evaluated_node = check!(
                        ReturnStatement::parse_from_pair(pair.clone(), config),
                        continue,
                        warnings,
                        errors
                    );
                    AstNode {
                        content: AstNodeContent::ReturnStatement(evaluated_node),
                        span: span::Span {
                            span: pair.as_span(),
                            path: path.clone(),
                        },
                    }
                }
                Rule::expr => {
                    let res = check!(
                        Expression::parse_from_pair(pair.clone(), config),
                        continue,
                        warnings,
                        errors
                    );
                    AstNode {
                        content: AstNodeContent::ImplicitReturnExpression(res.clone()),
                        span: res.span(),
                    }
                }
                Rule::while_loop => {
                    let res = check!(
                        WhileLoop::parse_from_pair(pair.clone(), config),
                        continue,
                        warnings,
                        errors
                    );
                    AstNode {
                        content: AstNodeContent::WhileLoop(res),
                        span: span::Span {
                            span: pair.as_span(),
                            path: path.clone(),
                        },
                    }
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
            })
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
