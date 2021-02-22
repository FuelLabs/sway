#![allow(warnings)]
#[macro_use]
extern crate pest_derive;
#[macro_use]
mod error;

mod parse_tree;
mod parser;
mod semantics;
use crate::parse_tree::*;
pub(crate) use crate::parse_tree::{
    Expression, FunctionDeclaration, FunctionParameter, Literal, TypeInfo, UseStatement,
};
use crate::parser::{HllParser, Rule};
use either::{Either, Left, Right};
use pest::iterators::Pair;
use pest::Parser;
use std::collections::HashMap;

pub use error::{CompileWarning, ParseError, ParseResult};
pub use pest::Span;

// todo rename to language name
#[derive(Debug)]
pub struct HllParseTree<'sc> {
    pub contract_ast: Option<ParseTree<'sc>>,
    pub script_ast: Option<ParseTree<'sc>>,
    pub predicate_ast: Option<ParseTree<'sc>>,
}

#[derive(Debug)]
pub struct ParseTree<'sc> {
    /// In a typical program, you might have a single root node for your syntax tree.
    /// In this language however, we want to expose multiple public functions at the root
    /// level so the tree is multi-root.
    root_nodes: Vec<AstNode<'sc>>,
}

#[derive(Debug, Clone)]
struct AstNode<'sc> {
    content: AstNodeContent<'sc>,
    span: Span<'sc>,
}

#[derive(Debug, Clone)]
pub(crate) enum AstNodeContent<'sc> {
    UseStatement(UseStatement<'sc>),
    CodeBlock(CodeBlock<'sc>),
    ReturnStatement(ReturnStatement<'sc>),
    Declaration(Declaration<'sc>),
    Expression(Expression<'sc>),
    TraitDeclaration(TraitDeclaration<'sc>),
}

#[derive(Debug, Clone)]
struct ReturnStatement<'sc> {
    expr: Expression<'sc>,
}

impl<'sc> ReturnStatement<'sc> {
    fn parse_from_pair(pair: Pair<'sc, Rule>) -> ParseResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut inner = pair.into_inner();
        let _ret_keyword = inner.next();
        let expr = inner.next();
        let res = match expr {
            None => ReturnStatement {
                expr: Expression::Unit,
            },
            Some(expr_pair) => {
                let expr = eval!(Expression::parse_from_pair, warnings, expr_pair);
                ReturnStatement { expr }
            }
        };
        Ok((res, warnings))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CodeBlock<'sc> {
    contents: Vec<AstNode<'sc>>,
    scope: HashMap<&'sc str, Declaration<'sc>>,
}

impl<'sc> CodeBlock<'sc> {
    fn parse_from_pair(block: Pair<'sc, Rule>) -> Result<Self, ParseError<'sc>> {
        let mut warnings = Vec::new();
        let block_inner = block.into_inner();
        let mut contents = Vec::new();
        for pair in block_inner {
            contents.push(match pair.as_rule() {
                Rule::declaration => AstNode {
                    content: AstNodeContent::Declaration(eval!(
                        Declaration::parse_from_pair,
                        warnings,
                        pair.clone()
                    )),
                    span: pair.into_span(),
                },
                Rule::expr_statement => {
                    let evaluated_node = eval!(
                        Expression::parse_from_pair,
                        warnings,
                        pair.clone().into_inner().next().unwrap().clone()
                    );
                    AstNode {
                        content: AstNodeContent::Expression(evaluated_node),
                        span: pair.into_span(),
                    }
                }
                Rule::return_statement => {
                    let evaluated_node =
                        eval!(ReturnStatement::parse_from_pair, warnings, pair.clone());
                    AstNode {
                        content: AstNodeContent::ReturnStatement(evaluated_node),
                        span: pair.into_span(),
                    }
                }
                a => {
                    println!("In code block parsing: {:?} {:?}", a, pair.as_str());
                    return Err(ParseError::Unimplemented(a, pair.as_span()));
                }
            })
        }
        if !warnings.is_empty() {
            todo!("This func needs to return warnings");
        }

        Ok(CodeBlock {  contents, scope: /* TODO */ HashMap::default()  })
    }
}

impl ParseTree<'_> {
    pub(crate) fn new() -> Self {
        ParseTree {
            root_nodes: Vec::new(),
        }
    }
}

impl<'sc> ParseTree<'sc> {
    pub(crate) fn push(&mut self, new_node: AstNode<'sc>) {
        self.root_nodes.push(new_node);
    }
}

pub fn parse<'sc>(input: &'sc str) -> ParseResult<'sc, HllParseTree<'sc>> {
    let mut parsed = HllParser::parse(Rule::program, input)?;
    let mut warnings: Vec<CompileWarning> = Vec::new();
    let res = eval!(
        parse_root_from_pairs,
        warnings,
        (parsed.next().unwrap().into_inner())
    );
    Ok((res, warnings))
}

// strategy: parse top level things
// and if we encounter a function body or block, recursively call this function and build
// sub-nodes
fn parse_root_from_pairs<'sc>(
    input: impl Iterator<Item = Pair<'sc, Rule>>,
) -> ParseResult<'sc, HllParseTree<'sc>> {
    let mut warnings = Vec::new();
    let mut fuel_ast = HllParseTree {
        contract_ast: None,
        script_ast: None,
        predicate_ast: None,
    };
    for block in input {
        let mut parse_tree = ParseTree::new();
        let rule = block.as_rule();
        let input = block.clone().into_inner();
        for pair in input {
            match pair.as_rule() {
                Rule::declaration => {
                    let decl = eval!(Declaration::parse_from_pair, warnings, pair.clone());
                    parse_tree.push(AstNode {
                        content: AstNodeContent::Declaration(decl),
                        span: pair.as_span(),
                    });
                }
                Rule::use_statement => {
                    let stmt = UseStatement::parse_from_pair(pair.clone())?;
                    parse_tree.push(AstNode {
                        content: AstNodeContent::UseStatement(stmt),
                        span: pair.as_span(),
                    });
                }
                _ => unreachable!(),
            }
        }
        match rule {
            Rule::contract => {
                if fuel_ast.contract_ast.is_some() {
                    return Err(ParseError::MultipleContracts(block.as_span()));
                }
                fuel_ast.contract_ast = Some(parse_tree);
            }
            Rule::script => {
                if fuel_ast.script_ast.is_some() {
                    return Err(ParseError::MultipleScripts(block.as_span()));
                }
                fuel_ast.script_ast = Some(parse_tree);
            }
            Rule::predicate => {
                if fuel_ast.predicate_ast.is_some() {
                    return Err(ParseError::MultiplePredicates(block.as_span()));
                }
                fuel_ast.predicate_ast = Some(parse_tree);
            }
            Rule::EOI => (),
            a => return Err(ParseError::InvalidTopLevelItem(a, block.into_span())),
        }
    }

    Ok((fuel_ast, warnings))
}

#[test]
fn test_basic_prog() {
    let prog = parse(
        r#"
        contract {

    enum yo
    <T> 
    where 
    T: IsAThing
    {
        x: u32,
        y: MyStruct<u32>
    }

    enum  MyOtherSumType 
    {
        x: u32,
        y: MyStruct<u32>
    }
        struct MyStruct<T> {
            field_name: u64,
            other_field: T,
        }


    fn generic_function
    <T>
    (arg1: u64,
    arg2: T) 
    :
    T 
    where T: Display,
          T: Debug {
          let x: MyStruct = 
          MyStruct
          {
              field_name:
              5 
          };
          return 
          match 
            arg1
          {
               1 
               => true,
               _ => { return false; },
          };
    }

    struct MyStruct {
        test: string,
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
            let mut y = 0b11111111; 
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
    
    }
    
    "#,
    );
    dbg!(&prog);
    prog.unwrap();
}
