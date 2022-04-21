use super::WhileLoop;
use crate::{
    build_config::BuildConfig,
    error::*,
    parse_tree::{Expression, ReturnStatement},
    parser::Rule,
    AstNode, AstNodeContent, Declaration, VariableDeclaration,
};

use sway_types::span::Span;

use pest::iterators::Pair;

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub contents: Vec<AstNode>,
    pub(crate) whole_block_span: Span,
}

impl CodeBlock {
    pub fn span(&self) -> &Span {
        &self.whole_block_span
    }
    pub(crate) fn parse_from_pair(
        block: Pair<Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<Self> {
        let path = config.map(|c| c.path());
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let whole_block_span = Span::from_pest(block.as_span(), path.clone());
        let block_inner = block.into_inner();
        let mut contents = Vec::new();
        for pair in block_inner {
            let span = Span::from_pest(pair.as_span(), path.clone());
            let mut ast_nodes = match pair.as_rule() {
                Rule::declaration => check!(
                    Declaration::parse_from_pair(pair.clone(), config),
                    continue,
                    warnings,
                    errors
                )
                .into_iter()
                .map(|content| AstNode {
                    content: AstNodeContent::Declaration(content),
                    span: Span::from_pest(pair.as_span(), path.clone()),
                })
                .collect::<Vec<_>>(),
                Rule::expr_statement => {
                    let ParserLifter { value, var_decls } = check!(
                        Expression::parse_from_pair(
                            pair.clone().into_inner().next().unwrap().clone(),
                            config
                        ),
                        continue,
                        warnings,
                        errors
                    );
                    let mut ast_node_contents = collect_var_decls(var_decls, span.clone());
                    ast_node_contents.push(AstNode {
                        content: AstNodeContent::Expression(value),
                        span,
                    });
                    ast_node_contents
                }
                Rule::return_statement => {
                    let ParserLifter { value, var_decls } = check!(
                        ReturnStatement::parse_from_pair(pair.clone(), config),
                        continue,
                        warnings,
                        errors
                    );
                    let mut ast_node_contents = collect_var_decls(var_decls, span.clone());
                    ast_node_contents.push(AstNode {
                        content: AstNodeContent::ReturnStatement(value),
                        span,
                    });
                    ast_node_contents
                }
                Rule::expr => {
                    let ParserLifter { value, var_decls } = check!(
                        Expression::parse_from_pair(pair.clone(), config),
                        continue,
                        warnings,
                        errors
                    );
                    let mut ast_node_contents = collect_var_decls(var_decls, span.clone());
                    let expr_span = value.span();
                    ast_node_contents.push(AstNode {
                        content: AstNodeContent::ImplicitReturnExpression(value),
                        span: expr_span,
                    });
                    ast_node_contents
                }
                Rule::while_loop => {
                    let ParserLifter { value, var_decls } = check!(
                        WhileLoop::parse_from_pair(pair.clone(), config),
                        continue,
                        warnings,
                        errors
                    );
                    let mut ast_node_contents = collect_var_decls(var_decls, span.clone());
                    ast_node_contents.push(AstNode {
                        content: AstNodeContent::WhileLoop(value),
                        span,
                    });
                    ast_node_contents
                }
                a => {
                    println!("In code block parsing: {:?} {:?}", a, pair.as_str());
                    errors.push(CompileError::UnimplementedRule(a, span));
                    continue;
                }
            };
            contents.append(&mut ast_nodes);
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

fn collect_var_decls(var_decls: Vec<VariableDeclaration>, span: Span) -> Vec<AstNode> {
    var_decls
        .into_iter()
        .map(|x| AstNode {
            content: AstNodeContent::Declaration(Declaration::VariableDeclaration(x)),
            span: span.clone(),
        })
        .collect::<Vec<_>>()
}
