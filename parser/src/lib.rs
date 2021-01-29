#[macro_use]
extern crate pest_derive;

mod ast;
mod error;
mod parser;
pub use error::CompileError;
use parser::HllParser;
use parser::Rule;
use pest::{Span, Parser};

use pest::iterators::Pair;

#[derive(Debug)]
pub struct Ast<'sc> {
    /// In a typical program, you might have a single root node for your syntax tree.
    /// In this language however, we want to expose multiple public functions at the root
    /// level so the tree is multi-root.
    root_nodes: Vec<AstNode<'sc>>
}

#[derive(Debug)]
struct AstNode<'sc> {
    content: AstNodeContent<'sc>,
    span: Span<'sc>
}

#[derive(Debug)]
enum AstNodeContent<'sc> {
    ImportStatement(&'sc str),
}

impl Ast<'_> {
    pub(crate) fn new() -> Self {
        Ast {
            root_nodes: Vec::new() 
        }
    }
}


pub fn parse(input: &str) -> Result<Ast, CompileError> {
    let parsed = HllParser::parse(Rule::program, input)?;
    parse_from_pairs(parsed)
}

// strategy: parse top level things
// and if we encounter a function body or block, recursively call this function and build
// sub-nodes 
fn parse_from_pairs<'sc> (input: impl Iterator<Item = Pair<'sc, Rule>>) -> Result<Ast<'sc>, CompileError> {
    let mut ast = Ast::new();
    for pair in input {
        match pair.as_rule() {
            Rule::use_statement=> {
                //   ast.push(Print(Box::new(build_ast_from_expr(pair))));
            }
        _ => todo!()
        }
    }

    Ok(ast)
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
