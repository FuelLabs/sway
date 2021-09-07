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
        config: Option<BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let path = config.map(|c| c.dir_of_code);
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let whole_block_span = span::Span {
            span: block.as_span(),
            path,
        };
        let block_inner = block.into_inner();
        let mut contents = Vec::new();
        for pair in block_inner {
            contents.push(match pair.as_rule() {
                Rule::declaration => AstNode {
                    content: AstNodeContent::Declaration(eval2!(
                        Declaration::parse_from_pair,
                        warnings,
                        errors,
                        pair.clone(),
                        config,
                        continue
                    )),
                    span: span::Span {
                        span: pair.as_span(),
                        path,
                    },
                },
                Rule::expr_statement => {
                    let evaluated_node = eval2!(
                        Expression::parse_from_pair,
                        warnings,
                        errors,
                        pair.clone().into_inner().next().unwrap().clone(),
                        config,
                        continue
                    );
                    AstNode {
                        content: AstNodeContent::Expression(evaluated_node),
                        span: span::Span {
                            span: pair.as_span(),
                            path,
                        },
                    }
                }
                Rule::return_statement => {
                    let evaluated_node = eval2!(
                        ReturnStatement::parse_from_pair,
                        warnings,
                        errors,
                        pair.clone(),
                        config,
                        continue
                    );
                    AstNode {
                        content: AstNodeContent::ReturnStatement(evaluated_node),
                        span: span::Span {
                            span: pair.as_span(),
                            path,
                        },
                    }
                }
                Rule::expr => {
                    let res = eval2!(
                        Expression::parse_from_pair,
                        warnings,
                        errors,
                        pair.clone(),
                        config,
                        continue
                    );
                    AstNode {
                        content: AstNodeContent::ImplicitReturnExpression(res.clone()),
                        span: res.span(),
                    }
                }
                Rule::while_loop => {
                    let res = eval2!(
                        WhileLoop::parse_from_pair,
                        warnings,
                        errors,
                        pair.clone(),
                        config,
                        continue
                    );
                    AstNode {
                        content: AstNodeContent::WhileLoop(res),
                        span: span::Span {
                            span: pair.as_span(),
                            path,
                        },
                    }
                }
                a => {
                    println!("In code block parsing: {:?} {:?}", a, pair.as_str());
                    errors.push(CompileError::UnimplementedRule(
                        a,
                        span::Span {
                            span: pair.as_span(),
                            path,
                        },
                    ));
                    continue;
                }
            })
        }

        ok(
            CodeBlock {  whole_block_span, contents, scope: /* TODO */ HashMap::default()  },
            warnings,
            errors,
        )
    }
}
