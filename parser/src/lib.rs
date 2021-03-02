#![allow(warnings)]
#[macro_use]
extern crate pest_derive;
#[macro_use]
mod error;

mod parse_tree;
mod parser;
mod semantics;
pub(crate) mod types;
use crate::error::*;
use crate::parse_tree::*;
pub(crate) use crate::parse_tree::{
    Expression, FunctionDeclaration, FunctionParameter, Literal, UseStatement,
};
use crate::parser::{HllParser, Rule};
use either::{Either, Left, Right};
use pest::iterators::Pair;
use pest::Parser;
use semantics::TypedParseTree;
use std::collections::HashMap;
use types::TypeInfo;

pub use error::{CompileError, CompileResult, CompileWarning};
pub use pest::Span;

// todo rename to language name
#[derive(Debug)]
pub struct HllParseTree<'sc> {
    pub contract_ast: Option<ParseTree<'sc>>,
    pub script_ast: Option<ParseTree<'sc>>,
    pub predicate_ast: Option<ParseTree<'sc>>,
}

#[derive(Debug)]
pub struct HllTypedParseTree<'sc> {
    contract_ast: Option<TypedParseTree<'sc>>,
    script_ast: Option<TypedParseTree<'sc>>,
    predicate_ast: Option<TypedParseTree<'sc>>,
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
    ImplicitReturnExpression(Expression<'sc>),
}

#[derive(Debug, Clone)]
struct ReturnStatement<'sc> {
    expr: Expression<'sc>,
}

impl<'sc> ReturnStatement<'sc> {
    fn parse_from_pair(pair: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let span = pair.as_span();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut inner = pair.into_inner();
        let _ret_keyword = inner.next();
        let expr = inner.next();
        let res = match expr {
            None => ReturnStatement {
                expr: Expression::Unit { span },
            },
            Some(expr_pair) => {
                let expr = eval!(
                    Expression::parse_from_pair,
                    warnings,
                    errors,
                    expr_pair,
                    Expression::Unit { span }
                );
                ReturnStatement { expr }
            }
        };
        ok(res, warnings, errors)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CodeBlock<'sc> {
    contents: Vec<AstNode<'sc>>,
    scope: HashMap<&'sc str, Declaration<'sc>>,
}

impl<'sc> CodeBlock<'sc> {
    fn parse_from_pair(block: Pair<'sc, Rule>) -> CompileResult<'sc, Self> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
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
                    span: pair.into_span(),
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
                        span: pair.into_span(),
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
                        content: AstNodeContent::ImplicitReturnExpression(res),
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
            CodeBlock {  contents, scope: /* TODO */ HashMap::default()  },
            warnings,
            errors,
        )
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

pub fn parse<'sc>(input: &'sc str) -> CompileResult<'sc, HllParseTree<'sc>> {
    let mut warnings: Vec<CompileWarning> = Vec::new();
    let mut errors: Vec<CompileError> = Vec::new();
    let mut parsed = match HllParser::parse(Rule::program, input) {
        Ok(o) => o,
        Err(e) => return err(Vec::new(), vec![e.into()]),
    };
    let res = eval!(
        parse_root_from_pairs,
        warnings,
        errors,
        (parsed.next().unwrap().into_inner()),
        return err(warnings, errors)
    );
    ok(res, warnings, errors)
}

pub fn compile<'sc>(
    input: &'sc str,
) -> Result<
    (HllTypedParseTree<'sc>, Vec<CompileWarning<'sc>>),
    (Vec<CompileError<'sc>>, Vec<CompileWarning<'sc>>),
> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let parse_tree = eval!(
        parse,
        warnings,
        errors,
        input,
        return Err((errors, warnings))
    );

    let maybe_contract_tree: Option<Result<_, _>> = parse_tree
        .contract_ast
        .map(|tree| semantics::type_check_tree(tree));
    let maybe_predicate_tree: Option<Result<_, _>> = parse_tree
        .predicate_ast
        .map(|tree| semantics::type_check_tree(tree));
    let maybe_script_tree: Option<Result<_, _>> = parse_tree
        .script_ast
        .map(|tree| semantics::type_check_tree(tree));

    let contract_ast = match maybe_contract_tree {
        Some(Ok((tree, mut l_warnings))) => {
            warnings.append(&mut l_warnings);
            Some(tree)
        }
        Some(Err(mut errs)) => {
            errors.append(&mut errs);
            None
        }
        None => None,
    };
    let predicate_ast = match maybe_predicate_tree {
        Some(Ok((tree, mut l_warnings))) => {
            warnings.append(&mut l_warnings);
            Some(tree)
        }
        Some(Err(mut errs)) => {
            errors.append(&mut errs);
            None
        }
        None => None,
    };
    let script_ast = match maybe_script_tree {
        Some(Ok((tree, mut l_warnings))) => {
            warnings.append(&mut l_warnings);
            Some(tree)
        }
        Some(Err(mut errs)) => {
            errors.append(&mut errs);
            None
        }
        None => None,
    };
    if errors.is_empty() {
        Ok((
            HllTypedParseTree {
                contract_ast,
                script_ast,
                predicate_ast,
            },
            warnings,
        ))
    } else {
        Err((errors, warnings))
    }
}

// strategy: parse top level things
// and if we encounter a function body or block, recursively call this function and build
// sub-nodes
fn parse_root_from_pairs<'sc>(
    input: impl Iterator<Item = Pair<'sc, Rule>>,
) -> CompileResult<'sc, HllParseTree<'sc>> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
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
                    let decl = eval!(
                        Declaration::parse_from_pair,
                        warnings,
                        errors,
                        pair.clone(),
                        continue
                    );
                    parse_tree.push(AstNode {
                        content: AstNodeContent::Declaration(decl),
                        span: pair.as_span(),
                    });
                }
                Rule::use_statement => {
                    let stmt = eval!(
                        UseStatement::parse_from_pair,
                        warnings,
                        errors,
                        pair.clone(),
                        continue
                    );
                    parse_tree.push(AstNode {
                        content: AstNodeContent::UseStatement(stmt),
                        span: pair.as_span(),
                    });
                }
                a => unreachable!("{:?}", pair.as_str()),
            }
        }
        match rule {
            Rule::contract => {
                if fuel_ast.contract_ast.is_some() {
                    errors.push(CompileError::MultipleContracts(block.as_span()));
                } else {
                    fuel_ast.contract_ast = Some(parse_tree);
                }
            }
            Rule::script => {
                if fuel_ast.script_ast.is_some() {
                    errors.push(CompileError::MultipleScripts(block.as_span()));
                } else {
                    fuel_ast.script_ast = Some(parse_tree);
                }
            }
            Rule::predicate => {
                if fuel_ast.predicate_ast.is_some() {
                    errors.push(CompileError::MultiplePredicates(block.as_span()));
                } else {
                    fuel_ast.predicate_ast = Some(parse_tree);
                }
            }
            Rule::EOI => (),
            a => errors.push(CompileError::InvalidTopLevelItem(a, block.into_span())),
        }
    }

    ok(fuel_ast, warnings, errors)
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

    pub fn prints_number_five(): u8 {
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
#[test]
fn test_parenthesized() {
    let prog = parse(
        r#"
        contract {
        fn abi_func(): unit {
            let x = (5 + 6 / (1 + (2 / 1) + 4));
            return;
        }
   } 
    "#,
    );
    dbg!(&prog);
    prog.unwrap();
}
