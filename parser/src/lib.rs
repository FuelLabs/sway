#[macro_use]
extern crate pest_derive;
#[macro_use]
mod error;

mod asm_generation;
mod asm_lang;
mod control_flow_analysis;
mod ident;
mod parse_tree;
mod parser;
mod semantics;

use crate::parse_tree::*;
use crate::parser::{HllParser, Rule};
use crate::{asm_generation::compile_ast_to_asm, error::*};
pub use asm_generation::{AbstractInstructionSet, FinalizedAsm, HllAsmSet};
use control_flow_analysis::ControlFlowGraph;
use pest::iterators::Pair;
use pest::Parser;
use semantics::{TreeType, TypedParseTree};
use std::collections::HashMap;

pub(crate) mod types;
pub(crate) mod utils;
pub(crate) use crate::parse_tree::{Expression, UseStatement, WhileLoop};

pub use error::{CompileError, CompileResult, CompileWarning};
pub use ident::Ident;
pub use pest::Span;
pub use semantics::{Namespace, TypedDeclaration, TypedFunctionDeclaration};
pub use types::TypeInfo;

// todo rename to language name
#[derive(Debug)]
pub struct HllParseTree<'sc> {
    pub contract_ast: Option<ParseTree<'sc>>,
    pub script_ast: Option<ParseTree<'sc>>,
    pub predicate_ast: Option<ParseTree<'sc>>,
    pub library_exports: Vec<(Ident<'sc>, ParseTree<'sc>)>,
}

#[derive(Debug)]
pub struct HllTypedParseTree<'sc> {
    contract_ast: Option<TypedParseTree<'sc>>,
    script_ast: Option<TypedParseTree<'sc>>,
    predicate_ast: Option<TypedParseTree<'sc>>,
    pub library_exports: LibraryExports<'sc>,
}

#[derive(Debug)]
pub struct LibraryExports<'sc> {
    pub namespace: Namespace<'sc>,
    trees: Vec<TypedParseTree<'sc>>,
}

#[derive(Debug)]
pub struct ParseTree<'sc> {
    /// In a typical programming language, you might have a single root node for your syntax tree.
    /// In this language however, we want to expose multiple public functions at the root
    /// level so the tree is multi-root.
    root_nodes: Vec<AstNode<'sc>>,
    span: Span<'sc>,
}

#[derive(Debug, Clone)]
struct AstNode<'sc> {
    content: AstNodeContent<'sc>,
    span: Span<'sc>,
}

#[derive(Debug, Clone)]
pub(crate) enum AstNodeContent<'sc> {
    UseStatement(UseStatement<'sc>),
    ReturnStatement(ReturnStatement<'sc>),
    Declaration(Declaration<'sc>),
    Expression(Expression<'sc>),
    ImplicitReturnExpression(Expression<'sc>),
    WhileLoop(WhileLoop<'sc>),
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

impl<'sc> ParseTree<'sc> {
    pub(crate) fn new(span: Span<'sc>) -> Self {
        ParseTree {
            root_nodes: Vec::new(),
            span,
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
        Err(e) => {
            return err(
                Vec::new(),
                vec![CompileError::ParseFailure {
                    span: Span::new(input, get_start(&e), get_end(&e)).unwrap(),
                    err: e,
                }],
            )
        }
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

pub enum CompilationResult<'sc> {
    ContractAbi {
        abi: HashMap<usize, FinalizedAsm<'sc>>,
        warnings: Vec<CompileWarning<'sc>>,
    },
    ScriptAsm {
        asm: FinalizedAsm<'sc>,
        warnings: Vec<CompileWarning<'sc>>,
    },
    PredicateAsm {
        asm: FinalizedAsm<'sc>,
        warnings: Vec<CompileWarning<'sc>>,
    },
    Library {
        exports: LibraryExports<'sc>,
        warnings: Vec<CompileWarning<'sc>>,
    },
    Failure {
        warnings: Vec<CompileWarning<'sc>>,
        errors: Vec<CompileError<'sc>>,
    },
}

fn get_start(err: &pest::error::Error<Rule>) -> usize {
    match err.location {
        pest::error::InputLocation::Pos(num) => num,
        pest::error::InputLocation::Span((start, _)) => start,
    }
}

fn get_end(err: &pest::error::Error<Rule>) -> usize {
    match err.location {
        pest::error::InputLocation::Pos(num) => num,
        pest::error::InputLocation::Span((_, end)) => end,
    }
}

pub fn compile<'sc, 'manifest>(
    input: &'sc str,
    initial_namespace: &Namespace<'sc>,
) -> CompilationResult<'sc> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let parse_tree = eval!(
        parse,
        warnings,
        errors,
        input,
        return CompilationResult::Failure { errors, warnings }
    );

    let contract_ast: Option<_> = if let Some(tree) = parse_tree.contract_ast {
        match TypedParseTree::type_check(tree, initial_namespace.clone(), TreeType::Contract) {
            CompileResult::Ok {
                warnings: mut l_w,
                errors: mut l_e,
                value,
            } => {
                warnings.append(&mut l_w);
                errors.append(&mut l_e);
                Some(value)
            }
            CompileResult::Err {
                warnings: mut l_w,
                errors: mut l_e,
            } => {
                warnings.append(&mut l_w);
                errors.append(&mut l_e);
                None
            }
        }
    } else {
        None
    };
    let predicate_ast: Option<_> = if let Some(tree) = parse_tree.predicate_ast {
        match TypedParseTree::type_check(tree, initial_namespace.clone(), TreeType::Predicate) {
            CompileResult::Ok {
                warnings: mut l_w,
                errors: mut l_e,
                value,
            } => {
                warnings.append(&mut l_w);
                errors.append(&mut l_e);
                Some(value)
            }
            CompileResult::Err {
                warnings: mut l_w,
                errors: mut l_e,
            } => {
                warnings.append(&mut l_w);
                errors.append(&mut l_e);
                None
            }
        }
    } else {
        None
    };
    let script_ast: Option<_> = if let Some(tree) = parse_tree.script_ast {
        match TypedParseTree::type_check(tree, initial_namespace.clone(), TreeType::Script) {
            CompileResult::Ok {
                warnings: mut l_w,
                errors: mut l_e,
                value,
            } => {
                warnings.append(&mut l_w);
                errors.append(&mut l_e);
                Some(value)
            }
            CompileResult::Err {
                warnings: mut l_w,
                errors: mut l_e,
            } => {
                warnings.append(&mut l_w);
                errors.append(&mut l_e);
                None
            }
        }
    } else {
        None
    };
    let library_exports: LibraryExports = {
        let res: Vec<_> = parse_tree
            .library_exports
            .into_iter()
            .filter_map(|(name, tree)| {
                match TypedParseTree::type_check(tree, initial_namespace.clone(), TreeType::Library)
                {
                    CompileResult::Ok {
                        warnings: mut l_w,
                        errors: mut l_e,
                        value,
                    } => {
                        warnings.append(&mut l_w);
                        errors.append(&mut l_e);
                        Some((name, value))
                    }
                    CompileResult::Err {
                        warnings: mut l_w,
                        errors: mut l_e,
                    } => {
                        warnings.append(&mut l_w);
                        errors.append(&mut l_e);
                        None
                    }
                }
            })
            .collect();
        let mut exports = LibraryExports {
            namespace: Default::default(),
            trees: vec![],
        };
        for (ref name, parse_tree) in res {
            exports.namespace.insert_module(
                name.primary_name.to_string(),
                parse_tree.namespace().clone(),
            );
            exports.trees.push(parse_tree);
        }
        exports
    };
    // If there are errors, display them now before performing control flow analysis.
    // It is necessary that the syntax tree is well-formed for control flow analysis
    // to be correct.
    if !errors.is_empty() {
        return CompilationResult::Failure { errors, warnings };
    }

    // perform control flow analysis on each branch
    let (script_warnings, script_errors) =
        perform_control_flow_analysis(&script_ast, TreeType::Script);
    let (contract_warnings, contract_errors) =
        perform_control_flow_analysis(&contract_ast, TreeType::Contract);
    let (predicate_warnings, predicate_errors) =
        perform_control_flow_analysis(&predicate_ast, TreeType::Predicate);
    let (library_warnings, library_errors) =
        perform_control_flow_analysis_on_library_exports(&library_exports);

    let mut l_warnings = [
        script_warnings,
        contract_warnings,
        predicate_warnings,
        library_warnings,
    ]
    .concat();
    let mut l_errors = [
        script_errors,
        contract_errors,
        predicate_errors,
        library_errors,
    ]
    .concat();

    errors.append(&mut l_errors);
    warnings.append(&mut l_warnings);
    // for each syntax tree, generate assembly.
    let predicate_asm = if let Some(tree) = predicate_ast {
        Some(compile_ast_to_asm(tree))
    } else {
        None
    };

    let contract_asm = if let Some(tree) = contract_ast {
        Some(compile_ast_to_asm(tree))
    } else {
        None
    };

    let script_asm = if let Some(tree) = script_ast {
        Some(compile_ast_to_asm(tree))
    } else {
        None
    };
    if errors.is_empty() {
        // TODO move this check earlier and don't compile all of them if there is only one
        match (predicate_asm, contract_asm, script_asm, library_exports) {
            (Some(pred), None, None, o) if o.trees.is_empty() => CompilationResult::PredicateAsm {
                asm: pred,
                warnings,
            },
            (None, Some(contract), None, o) if o.trees.is_empty() => {
                CompilationResult::ContractAbi {
                    abi: todo!(),
                    warnings,
                }
            }
            (None, None, Some(script), o) if o.trees.is_empty() => CompilationResult::ScriptAsm {
                asm: script,
                warnings,
            },
            (None, None, None, o) if !o.trees.is_empty() => CompilationResult::Library {
                warnings,
                exports: o,
            },
            _ => todo!(),
        }
    } else {
        CompilationResult::Failure { errors, warnings }
    }
}

fn perform_control_flow_analysis<'sc>(
    tree: &Option<TypedParseTree<'sc>>,
    tree_type: TreeType,
) -> (Vec<CompileWarning<'sc>>, Vec<CompileError<'sc>>) {
    match tree {
        Some(tree) => {
            let graph = ControlFlowGraph::construct_dead_code_graph(tree, tree_type);
            let mut warnings = vec![];
            let mut errors = vec![];
            warnings.append(&mut graph.find_dead_code());
            let graph = ControlFlowGraph::construct_return_path_graph(tree);
            errors.append(&mut graph.analyze_return_paths());
            (warnings, errors)
        }
        None => (vec![], vec![]),
    }
}
fn perform_control_flow_analysis_on_library_exports<'sc>(
    lib: &LibraryExports<'sc>,
) -> (Vec<CompileWarning<'sc>>, Vec<CompileError<'sc>>) {
    let mut warnings = vec![];
    let mut errors = vec![];
    for tree in &lib.trees {
        let graph = ControlFlowGraph::construct_dead_code_graph(tree, TreeType::Library);
        warnings.append(&mut graph.find_dead_code());
        let graph = ControlFlowGraph::construct_return_path_graph(tree);
        errors.append(&mut graph.analyze_return_paths());
    }
    (warnings, errors)
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
        library_exports: vec![],
    };
    for block in input {
        let mut parse_tree = ParseTree::new(block.as_span());
        let rule = block.as_rule();
        let input = block.clone().into_inner();
        let mut library_name = None;
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
                Rule::library_name => {
                    let lib_pair = pair.into_inner().next().unwrap();
                    library_name = Some(eval!(
                        Ident::parse_from_pair,
                        warnings,
                        errors,
                        lib_pair,
                        continue
                    ));
                }
                _ => unreachable!("{:?}", pair.as_str()),
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
            Rule::library => {
                fuel_ast.library_exports.push((library_name.expect("Safe unwrap, because the parser enforces the library keyword is followed by a name. This is an invariant"), parse_tree));
            }
            Rule::EOI => (),
            a => errors.push(CompileError::InvalidTopLevelItem(a, block.as_span())),
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
    ->
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
        fn myfunc(x: int) -> unit;
        } {
        // methods
        fn calls_interface_fn(x: int) -> unit {
            // declare a byte
            let x = 0b10101111;
            let mut y = 0b11111111; 
            self.interface_fn(x);
        }
    }

    pub fn prints_number_five() -> u8 {
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
        pub fn abi_func() -> unit {
            let x = (5 + 6 / (1 + (2 / 1) + 4));
            return;
        }
   } 
    "#,
    );
    prog.unwrap();
}
