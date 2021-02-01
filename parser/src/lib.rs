#[macro_use]
extern crate pest_derive;

mod ast;
mod error;
mod parser;
pub use error::CompileError;
use parser::{HllParser, Rule};
use pest::{Parser, Span};
use std::collections::HashMap;

use crate::ast::{Expression, FunctionDeclaration, FunctionParameter, ImportStatement, Literal};
use pest::iterators::Pair;

#[derive(Debug)]
pub struct Ast<'sc> {
    /// In a typical program, you might have a single root node for your syntax tree.
    /// In this language however, we want to expose multiple public functions at the root
    /// level so the tree is multi-root.
    root_nodes: Vec<AstNode<'sc>>,
}

#[derive(Debug)]
struct AstNode<'sc> {
    content: AstNodeContent<'sc>,
    span: Span<'sc>,
}

#[derive(Debug)]
enum AstNodeContent<'sc> {
    ImportStatement(ImportStatement<'sc>),
    CodeBlock(CodeBlock<'sc>),
    Declaration(Declaration<'sc>),
    Expression(Expression<'sc>),
}

#[derive(Debug)]
pub(crate) struct CodeBlock<'sc> {
    contents: Vec<AstNode<'sc>>,
    scope: HashMap<&'sc str, Declaration<'sc>>,
}

impl<'sc> CodeBlock<'sc> {
    fn parse_from_pair(block: Pair<'sc, Rule>) -> Result<Self, CompileError<'sc>> {
        let block_inner = block.into_inner();
        let mut contents = Vec::new();
        for pair in block_inner {
            contents.push(match pair.as_rule() {
                Rule::declaration => AstNode {
                    content: AstNodeContent::Declaration(Declaration::parse_from_pair(
                        pair.clone(),
                    )?),
                    span: pair.into_span(),
                },
                Rule::expr => AstNode {
                    content: AstNodeContent::Expression(parse_expr_from_pair(pair.clone())?),
                    span: pair.into_span(),
                },
                a => {
                    println!("In code block parsing: {:?} {:?}", a, pair.as_str());
                    return Err(CompileError::Unimplemented(a, pair.as_span()));
                }
            })
        }

        println!("contents are {:?}", contents);

        Ok(CodeBlock {  contents, scope: /* TODO */ HashMap::default()  })
    }
}

impl Ast<'_> {
    pub(crate) fn new() -> Self {
        Ast {
            root_nodes: Vec::new(),
        }
    }
}

impl<'sc> Ast<'sc> {
    pub(crate) fn push(&mut self, new_node: AstNode<'sc>) {
        self.root_nodes.push(new_node);
    }
}

pub fn parse(input: &str) -> Result<Ast, CompileError> {
    let mut parsed = HllParser::parse(Rule::program, input)?;
    parse_root_from_pairs(parsed.next().unwrap().into_inner())
}

// strategy: parse top level things
// and if we encounter a function body or block, recursively call this function and build
// sub-nodes
fn parse_root_from_pairs<'sc>(
    input: impl Iterator<Item = Pair<'sc, Rule>>,
) -> Result<Ast<'sc>, CompileError<'sc>> {
    let mut ast = Ast::new();
    for pair in input {
        match pair.as_rule() {
            Rule::declaration => {
                let decl = Declaration::parse_from_pair(pair.clone())?;
                ast.push(AstNode { content: AstNodeContent::Declaration(decl), span: pair.as_span()});
            }
            Rule::use_statement => {
                return Err(CompileError::Unimplemented(
                    Rule::use_statement,
                    pair.as_span(),
                ))
            }
            Rule::EOI => (),
            a => return Err(CompileError::InvalidTopLevelItem(a, pair.into_span())),
        }
    }

    Ok(ast)
}

#[derive(Debug)]
struct VariableDeclaration<'sc> {
    name: &'sc str,
    body: Expression<'sc>, // will be codeblock variant
}

#[derive(Debug)]
struct TraitDeclaration<'sc> {
    tmp: &'sc str,
}

#[derive(Debug)]
enum Declaration<'sc> {
    VariableDeclaration(VariableDeclaration<'sc>),
    FunctionDeclaration(FunctionDeclaration<'sc>),
    TraitDeclaration(TraitDeclaration<'sc>),
}

impl<'sc> Declaration<'sc> {
    fn parse_from_pair(decl: Pair<'sc, Rule>) -> Result<Self, CompileError<'sc>> {
        let mut pair = decl.clone().into_inner();
        let decl_inner = pair.next().unwrap();
        let parsed_declaration = match decl_inner.as_rule() {
            Rule::fn_decl => {
                let mut parts = decl_inner.clone().into_inner();
                let mut signature = parts.next().unwrap().into_inner();
                let _fn_keyword = signature.next().unwrap();
                let name = signature.next().unwrap().as_str();
                let parameters = signature.next().unwrap();
                let parameters = FunctionParameter::list_from_pairs(parameters.into_inner())?;
                let body = parts.next().unwrap();
                let body = CodeBlock::parse_from_pair(body)?;
                Declaration::FunctionDeclaration(FunctionDeclaration {
                    name,
                    parameters,
                    body,
                    span: decl_inner.as_span(),
                })
            }
            Rule::var_decl => {
                let mut var_decl_parts = decl_inner.into_inner();
                let _let_keyword = var_decl_parts.next();
                let name: &'sc str = var_decl_parts.next().unwrap().as_str().trim();
                let body = var_decl_parts.next().unwrap();
                let body = parse_expr_from_pair(body)?;
                Declaration::VariableDeclaration(VariableDeclaration { name, body })
            }
            Rule::trait_decl => {
                eprintln!("Unimplemented feature: trait decl");
                return Err(CompileError::Unimplemented(
                    Rule::trait_decl,
                    decl.as_span(),
                ));
                //Declaration::TraitDeclaration(todo!()),
            }
            _ => unreachable!("declarations don't have any other sub-types"),
        };
        Ok(parsed_declaration)
    }
}

fn parse_expr_without_getting_inner<'sc>(
    expr: Pair<'sc, Rule>,
) -> Result<Expression<'sc>, CompileError<'sc>> {
    let parsed = match expr.as_rule() {
        Rule::literal_value => Expression::Literal(Literal::parse_from_pair(expr)?),
        Rule::func_app => {
            let mut func_app_parts = expr.into_inner();
            let name = func_app_parts.next().unwrap().as_str();
            let arguments = func_app_parts.next();
            let arguments = arguments.map(|x| {
                x.into_inner()
                    .map(|x| parse_expr_without_getting_inner(x))
                    .collect::<Result<Vec<_>, _>>()
            });
            let arguments = arguments.unwrap_or_else(|| Ok(Vec::new()))?;

            Expression::FunctionApplication { name, arguments }
        }
        Rule::var_exp => {
            let mut var_exp_parts = expr.into_inner();
            Expression::VariableExpression {
                name: var_exp_parts.next().unwrap().as_str(),
            }
        }
        a => {
            return Err(CompileError::Unimplemented(a, expr.as_span()));
            eprintln!(
                "Unimplemented expr: {:?} ({:?}) ({:?})",
                a,
                expr.as_str(),
                expr.as_span()
            );
        }
    };
    Ok(parsed)
}

fn parse_expr_from_pair<'sc>(expr: Pair<'sc, Rule>) -> Result<Expression<'sc>, CompileError<'sc>> {
    let mut expr_iter = expr.into_inner();
    let expr = expr_iter.next().unwrap();
    if expr_iter.next().is_some() {
        return Err(CompileError::Internal(
            "Expression parsed with non-unary cardinality.",
            expr.into_span(),
        ));
    }
    parse_expr_without_getting_inner(expr)
}

#[test]
fn test_basic_prog() {
    let prog = parse(
        r#"
    use stdlib::println

    fn prints_number_five() {
        let x = 5
        println(x)
x.to_string()
    }"#,
    );
    dbg!(&prog);
}
