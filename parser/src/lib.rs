#![allow(warnings)]
#[macro_use]
extern crate pest_derive;

mod ast;
mod error;
mod parser;
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
struct TraitDeclaration<'sc> {
    name: &'sc str,
    interface_surface: Vec<TraitFn<'sc>>,
    methods: Vec<FunctionDeclaration<'sc>>,
}

impl<'sc> TraitDeclaration<'sc> {
    pub(crate) fn parse_from_pair(pair: Pair<'sc, Rule>) -> Result<Self, CompileError<'sc>> {
        let mut trait_parts = pair.into_inner();
        let _trait_keyword = trait_parts.next();
        let name = trait_parts.next().unwrap().as_str();
        let methods_and_interface = trait_parts
            .next()
            .map(|if_some: Pair<'sc, Rule>| -> Result<_, CompileError> {
                if_some
                    .into_inner()
                    .map(
                        |fn_sig_or_decl| -> Result<
                            Either<TraitFn<'sc>, FunctionDeclaration<'sc>>,
                            CompileError,
                        > {
                            Ok(match fn_sig_or_decl.as_rule() {
                                Rule::fn_signature => {
                                    Left(TraitFn::parse_from_pair(fn_sig_or_decl)?)
                                }
                                Rule::fn_decl => {
                                    Right(FunctionDeclaration::parse_from_pair(fn_sig_or_decl)?)
                                }
                                _ => unreachable!(),
                            })
                        },
                    )
                    .collect::<Result<Vec<_>, CompileError>>()
            })
            .unwrap_or_else(|| Ok(Vec::new()))?;

        let mut interface_surface = Vec::new();
        let mut methods = Vec::new();
        methods_and_interface.into_iter().for_each(|x| match x {
            Left(x) => interface_surface.push(x),
            Right(x) => methods.push(x),
        });

        Ok(TraitDeclaration {
            name,
            interface_surface,
            methods,
        })
    }
}

#[derive(Debug)]
struct TraitFn<'sc> {
    pub(crate) name: &'sc str,
    pub(crate) parameters: Vec<FunctionParameter<'sc>>,
    pub(crate) return_type: TypeInfo<'sc>,
}

impl<'sc> TraitFn<'sc> {
    fn parse_from_pair(pair: Pair<'sc, Rule>) -> Result<Self, CompileError<'sc>> {
        let mut signature = pair.clone().into_inner();
        let _fn_keyword = signature.next().unwrap();
        let name = signature.next().unwrap().as_str();
        let parameters = signature.next().unwrap();
        let parameters = FunctionParameter::list_from_pairs(parameters.into_inner())?;
        let return_type_signal = signature.next();
        let return_type = match return_type_signal {
            Some(_) => TypeInfo::parse_from_pair(signature.next().unwrap())?,
            None => TypeInfo::Unit,
        };

        Ok(TraitFn {
            name,
            parameters,
            return_type,
        })
    }
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

#[derive(Debug)]
struct VariableDeclaration<'sc> {
    name: &'sc str,
    type_ascription: Option<TypeInfo<'sc>>,
    body: Expression<'sc>, // will be codeblock variant
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
                Declaration::FunctionDeclaration(FunctionDeclaration::parse_from_pair(decl_inner)?)
            }
            Rule::var_decl => {
                let mut var_decl_parts = decl_inner.into_inner();
                let _let_keyword = var_decl_parts.next();
                let name: &'sc str = var_decl_parts.next().unwrap().as_str().trim();
                let mut maybe_body = var_decl_parts.next().unwrap();
                let type_ascription = match maybe_body.as_rule() {
                    Rule::type_ascription => {
                        let type_asc = maybe_body.clone();
                        maybe_body = var_decl_parts.next().unwrap();
                        Some(type_asc)
                    }
                    _ => None,
                };
                let type_ascription =
                    invert(type_ascription.map(|x| TypeInfo::parse_from_pair(x)))?;
                let body = Expression::parse_from_pair(maybe_body)?;
                Declaration::VariableDeclaration(VariableDeclaration {
                    name,
                    body,
                    type_ascription,
                })
            }
            Rule::trait_decl => {
                Declaration::TraitDeclaration(TraitDeclaration::parse_from_pair(decl_inner)?)
            }
            _ => unreachable!("declarations don't have any other sub-types"),
        };
        Ok(parsed_declaration)
    }
}

#[test]
fn test_basic_prog() {
    let prog = parse(
        r#"


    fn generic_function
    <T>
    (arg1: u64,
    arg2: T) 
    :
    T 
    where T: Display,
          T: Debug {
        return arg2;
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
// option res to res option helper
fn invert<T, E>(x: Option<Result<T, E>>) -> Result<Option<T>, E> {
    x.map_or(Ok(None), |v| v.map(Some))
}
