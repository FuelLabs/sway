use super::WhileLoop;
use crate::parser::Rule;
use crate::{
    error::*,
    parse_tree::{Expression, ReturnStatement},
    AstNode, AstNodeContent, Declaration,
};
use pest::iterators::Pair;
use pest::Span;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct CodeBlock<'sc> {
    pub(crate) contents: Vec<AstNode<'sc>>,
    pub(crate) scope: HashMap<&'sc str, Declaration<'sc>>,
    pub(crate) whole_block_span: Span<'sc>,
}

impl<'sc> CodeBlock<'sc> {
    pub(crate) fn parse_from_pair(block: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let whole_block_span = block.as_span();
        let block_inner = block.into_inner();
        let mut contents = Vec::new();
        for pair in block_inner {
            contents.push(match pair.as_rule() {
                Rule::declaration => AstNode {
                    content: AstNodeContent::Declaration(eval!(
                        Declaration::parse_from_pair,
                        warnings,
                        errors,
                        pair.clone(),
                        continue
                    )),
                    span: pair.as_span(),
                },
                Rule::expr_statement => {
                    let evaluated_node = eval!(
                        Expression::parse_from_pair,
                        warnings,
                        errors,
                        pair.clone().into_inner().next().unwrap().clone(),
                        continue
                    );
                    AstNode {
                        content: AstNodeContent::Expression(evaluated_node),
                        span: pair.as_span(),
                    }
                }
                Rule::return_statement => {
                    let evaluated_node = eval!(
                        ReturnStatement::parse_from_pair,
                        warnings,
                        errors,
                        pair.clone(),
                        continue
                    );
                    AstNode {
                        content: AstNodeContent::ReturnStatement(evaluated_node),
                        span: pair.as_span(),
                    }
                }
                Rule::expr => {
                    let res = eval!(
                        Expression::parse_from_pair,
                        warnings,
                        errors,
                        pair.clone(),
                        continue
                    );
                    AstNode {
                        content: AstNodeContent::ImplicitReturnExpression(res.clone()),
                        span: res.span(),
                    }
                }
                Rule::while_loop => {
                    let res = eval!(
                        WhileLoop::parse_from_pair,
                        warnings,
                        errors,
                        pair.clone(),
                        continue
                    );
                    AstNode {
                        content: AstNodeContent::WhileLoop(res),
                        span: pair.as_span(),
                    }
                }
                a => {
                    println!("In code block parsing: {:?} {:?}", a, pair.as_str());
                    errors.push(CompileError::UnimplementedRule(a, pair.as_span()));
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
