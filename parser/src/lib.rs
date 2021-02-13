#![allow(warnings)]
#[macro_use]
extern crate pest_derive;

mod ast;
mod error;
mod parser;
use crate::ast::*;
use crate::parser::{HllParser, Rule};
use either::{Either, Left, Right};
pub use error::CompileError;
use pest::{Parser, Span};
use std::collections::HashMap;

use crate::ast::{
    Expression, FunctionDeclaration, FunctionParameter, Literal, TypeInfo, UseStatement,
};
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
    UseStatement(UseStatement<'sc>),
    CodeBlock(CodeBlock<'sc>),
    ReturnStatement(ReturnStatement<'sc>),
    Declaration(Declaration<'sc>),
    Expression(Expression<'sc>),
    TraitDeclaration(TraitDeclaration<'sc>),
}

#[derive(Debug)]
struct ReturnStatement<'sc> {
    expr: Expression<'sc>,
}

impl<'sc> ReturnStatement<'sc> {
    fn parse_from_pair(pair: Pair<'sc, Rule>) -> Result<Self, CompileError> {
        let mut inner = pair.into_inner();
        let _ret_keyword = inner.next();
        let expr = inner.next();
        Ok(match expr {
            None => ReturnStatement {
                expr: Expression::Unit,
            },
            Some(expr_pair) => ReturnStatement {
                expr: Expression::parse_from_pair(expr_pair)?,
            },
        })
    }
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
                Rule::expr_statement => AstNode {
                    content: AstNodeContent::Expression(Expression::parse_from_pair(
                        pair.clone().into_inner().next().unwrap().clone(),
                    )?),
                    span: pair.into_span(),
                },
                Rule::return_statement => {
                    println!("parsing ret statement");
                    AstNode {
                        content: AstNodeContent::ReturnStatement(ReturnStatement::parse_from_pair(
                            pair.clone(),
                        )?),
                        span: pair.into_span(),
                    }
                }
                a => {
                    println!("In code block parsing: {:?} {:?}", a, pair.as_str());
                    return Err(CompileError::Unimplemented(a, pair.as_span()));
                }
            })
        }

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
                ast.push(AstNode {
                    content: AstNodeContent::Declaration(decl),
                    span: pair.as_span(),
                });
            }
            Rule::use_statement => {
                let stmt = UseStatement::parse_from_pair(pair.clone())?;
                ast.push(AstNode {
                    content: AstNodeContent::UseStatement(stmt),
                    span: pair.as_span(),
                });
            }
            Rule::EOI => (),
            a => return Err(CompileError::InvalidTopLevelItem(a, pair.into_span())),
        }
    }

    Ok(ast)
}

#[test]
fn test_basic_prog() {
    let prog = parse(
        r#"
        struct MyStruct {
            field_name: u64
        }


    fn generic_function
    <T>
    (arg1: u64,
    arg2: T) 
    :
    T 
    where T: Display,
          T: Debug {
          return 
          match 
            arg1
          {
               1 
               => {
               true
               },
               _ => { false },
          };
    }

    use stdlib::println;

    trait MyTrait {
        // interface points
        fn myfunc(x: int): unit
    } {
        // methods
        fn calls_interface_fn(x: int): unit {
            // declare a byte
            let x = 0b10101111;
            self.interface_fn(x);
        }
    }

    fn prints_number_five(): u8 {
        let x: u8 = 5;
        let reference_to_x = ref x;
        let second_value_of_x = deref x; // u8 is `Copy` so this clones
        println(x);
         x.to_string();
         let some_list = [
         5,
         10 + 3 / 2,
         func_app(my_args, (so_many_args))];
        return 5;
    }
    
    
    "#,
    );
    dbg!(&prog);
    prog.unwrap();
}
