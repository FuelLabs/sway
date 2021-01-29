#[macro_use]
extern crate pest_derive;

mod ast;
mod error;
mod parser;
pub use error::CompileError;
use parser::HllParser;
use parser::Rule;
use pest::{Parser, Span};

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
    ImportStatement(&'sc str),
    CodeBlock(CodeBlock<'sc>),
}

#[derive(Debug)]
struct CodeBlock<'sc> {
    contents: Vec<AstNode<'sc>>,
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
                let decl = parse_decl_from_pair(pair);
            }
            Rule::use_statement => {}
            a => return Err(CompileError::InvalidTopLevelItem(a, pair.into_span())),
        }
    }
    todo!();

    Ok(ast)
}

struct VariableDeclaration<'sc> {
    name: &'sc str,
    body: AstNode<'sc>, // will be codeblock variant
}

fn parse_decl_from_pair<'sc>(decl: Pair<'sc, Rule>) -> Result<AstNode<'sc>, CompileError<'sc>> {
    let mut pair = decl.into_inner();
    let decl_inner = pair.next().unwrap();
    match decl_inner.as_rule() {
        Rule::fn_decl => {
            let mut fn_parts = decl_inner.into_inner();
            let fn_signature = fn_parts.next().unwrap();
            let fn_body = fn_parts.next().unwrap();

            let fn_body = parse_code_block(fn_body)?;
        }
        Rule::var_decl => {
            let mut var_decl_parts = decl_inner.into_inner();
            let _let_keyword = var_decl_parts.next();
            let var_name: &'sc str = var_decl_parts.next().unwrap().as_str().trim();
            let var_body = var_decl_parts.next().unwrap();
            let var_body = parse_expr_from_pair(var_body)?;
            todo!("return AstNode for VarDecl");
        }
        Rule::trait_decl => (),
        _ => unreachable!("declarations don't have any other sub-types"),
    }
    todo!()
}

fn parse_expr_from_pair<'sc>(
    expr: Pair<'sc, Rule>,
) -> Result<AstNodeContent<'sc>, CompileError<'sc>> {
    todo!()
}

fn parse_code_block<'sc>(block: Pair<'sc, Rule>) -> Result<CodeBlock<'sc>, CompileError<'sc>> {
    let block_inner = block.into_inner();
    let mut block_contents = Vec::new();
    for pair in block_inner {
        match pair.as_rule() {
            Rule::declaration => {
                let decl = parse_decl_from_pair(pair)?;
                block_contents.push(decl);
            }
            a => println!("In code block parsing: {:?} {:?}", a, pair.as_str()),
        }
    }

    todo!()
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
