#[macro_use]
extern crate pest_derive;
#[macro_use]
pub mod error;

mod asm_generation;
mod asm_lang;
mod build_config;
pub mod constants;
mod control_flow_analysis;
mod ident;
pub mod parse_tree;
mod parser;
pub mod semantic_analysis;

pub use crate::parse_tree::*;
pub use crate::parser::{HllParser, Rule};
use crate::{asm_generation::compile_ast_to_asm, error::*};
pub use asm_generation::{AbstractInstructionSet, FinalizedAsm, HllAsmSet};
pub use build_config::BuildConfig;
use control_flow_analysis::{ControlFlowGraph, Graph};
use pest::iterators::Pair;
use pest::Parser;
use semantic_analysis::{TreeType, TypedParseTree};
pub mod types;
pub(crate) mod utils;
pub use crate::parse_tree::{Declaration, Expression, UseStatement, WhileLoop};

pub use error::{CompileError, CompileResult, CompileWarning};
pub use ident::Ident;
pub use pest::Span;
pub use semantic_analysis::{Namespace, TypedDeclaration, TypedFunctionDeclaration};
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
    pub root_nodes: Vec<AstNode<'sc>>,
    pub span: Span<'sc>,
}

#[derive(Debug, Clone)]
pub struct AstNode<'sc> {
    pub content: AstNodeContent<'sc>,
    pub span: Span<'sc>,
}

#[derive(Debug, Clone)]
pub enum AstNodeContent<'sc> {
    UseStatement(UseStatement<'sc>),
    ReturnStatement(ReturnStatement<'sc>),
    Declaration(Declaration<'sc>),
    Expression(Expression<'sc>),
    ImplicitReturnExpression(Expression<'sc>),
    WhileLoop(WhileLoop<'sc>),
    IncludeStatement(IncludeStatement<'sc>),
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
    Success {
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
pub enum BytecodeCompilationResult<'sc> {
    Success {
        bytes: Vec<u8>,
        warnings: Vec<CompileWarning<'sc>>,
    },
    Library {
        warnings: Vec<CompileWarning<'sc>>,
    },
    Failure {
        warnings: Vec<CompileWarning<'sc>>,
        errors: Vec<CompileError<'sc>>,
    },
}

pub fn extract_keyword(line: &str, rule: Rule) -> Option<&str> {
    if let Ok(pair) = HllParser::parse(rule, line) {
        Some(pair.as_str().trim())
    } else {
        None
    }
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

/// This struct represents the compilation of an internal dependency
/// defined through an include statement (the `dep` keyword).
pub(crate) struct InnerDependencyCompileResult<'sc> {
    library_exports: LibraryExports<'sc>,
}
/// For internal compiler use.
/// Compiles an included file and returns its control flow and dead code graphs.
/// These graphs are merged into the parent program's graphs for accurate analysis.
///
/// TODO -- there is _so_ much duplicated code and messiness in this file around the
/// different types of compilation and stuff. After we get to a good state with the MVP,
/// clean up the types here with the power of hindsight
pub(crate) fn compile_inner_dependency<'sc, 'manifest>(
    input: &'sc str,
    initial_namespace: &Namespace<'sc>,
    build_config: BuildConfig,
    dead_code_graph: &mut ControlFlowGraph<'sc>,
) -> CompileResult<'sc, InnerDependencyCompileResult<'sc>> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let parse_tree = eval!(parse, warnings, errors, input, return err(warnings, errors));
    match (
        parse_tree.script_ast,
        parse_tree.predicate_ast,
        parse_tree.contract_ast,
    ) {
        (None, None, None) => (),
        _ => {
            errors.push(CompileError::ImportMustBeLibrary {
                span: Span::new(input, 0, 0).unwrap(),
            });
            return err(warnings, errors);
        }
    }
    let library_exports: LibraryExports = {
        let res: Vec<_> = parse_tree
            .library_exports
            .into_iter()
            .filter_map(|(name, tree)| {
                let mut checked_tree = TypedParseTree::type_check(
                    tree,
                    initial_namespace.clone(),
                    TreeType::Library,
                    &build_config,
                    dead_code_graph,
                );
                warnings.append(&mut checked_tree.warnings);
                errors.append(&mut checked_tree.errors);
                checked_tree.value.map(|value| (name, value))
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
    // look for return path errors
    for tree in &library_exports.trees {
        let graph = ControlFlowGraph::construct_return_path_graph(tree);
        errors.append(&mut graph.analyze_return_paths());
    }

    for tree in &library_exports.trees {
        // The dead code will be analyzed later wholistically with the rest of the program
        // since we can't tell what is dead and what isn't just from looking at this file
        if let Err(e) =
            ControlFlowGraph::append_to_dead_code_graph(tree, TreeType::Library, dead_code_graph)
        {
            errors.push(e)
        };
    }

    ok(
        InnerDependencyCompileResult { library_exports },
        warnings,
        errors,
    )
}

pub fn compile_to_asm<'sc, 'manifest>(
    input: &'sc str,
    initial_namespace: &Namespace<'sc>,
    build_config: BuildConfig,
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
    let mut dead_code_graph = ControlFlowGraph {
        graph: Graph::new(),
        entry_points: vec![],
        namespace: Default::default(),
    };

    let mut type_check_ast = |ast: Option<_>, tree_type| {
        ast.map(|tree| {
            let mut typed_tree = TypedParseTree::type_check(
                tree,
                initial_namespace.clone(),
                tree_type,
                &build_config,
                &mut dead_code_graph,
            );
            warnings.append(&mut typed_tree.warnings);
            errors.append(&mut typed_tree.errors);
            typed_tree.value
        })
        .flatten()
    };

    let contract_ast = type_check_ast(parse_tree.contract_ast, TreeType::Contract);
    let predicate_ast = type_check_ast(parse_tree.predicate_ast, TreeType::Predicate);
    let script_ast = type_check_ast(parse_tree.script_ast, TreeType::Script);

    let library_exports: LibraryExports = {
        let res: Vec<_> = parse_tree
            .library_exports
            .into_iter()
            .filter_map(|(name, tree)| {
                let mut typed_library = TypedParseTree::type_check(
                    tree,
                    initial_namespace.clone(),
                    TreeType::Library,
                    &build_config,
                    &mut dead_code_graph,
                );
                warnings.append(&mut typed_library.warnings);
                errors.append(&mut typed_library.errors);
                typed_library.value.map(|value| (name, value))
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
        perform_control_flow_analysis(&script_ast, TreeType::Script, &mut dead_code_graph);
    let (contract_warnings, contract_errors) =
        perform_control_flow_analysis(&contract_ast, TreeType::Contract, &mut dead_code_graph);
    let (predicate_warnings, predicate_errors) =
        perform_control_flow_analysis(&predicate_ast, TreeType::Predicate, &mut dead_code_graph);
    let (library_warnings, library_errors) =
        perform_control_flow_analysis_on_library_exports(&library_exports, &mut dead_code_graph);

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
    let predicate_asm = (|| {
        if let Some(tree) = predicate_ast {
            Some(check!(
                compile_ast_to_asm(tree),
                return None,
                warnings,
                errors
            ))
        } else {
            None
        }
    })();

    let contract_asm = (|| {
        if let Some(tree) = contract_ast {
            Some(check!(
                compile_ast_to_asm(tree),
                return None,
                warnings,
                errors
            ))
        } else {
            None
        }
    })();

    let script_asm = (|| {
        if let Some(tree) = script_ast {
            Some(check!(
                compile_ast_to_asm(tree),
                return None,
                warnings,
                errors
            ))
        } else {
            None
        }
    })();

    if errors.is_empty() {
        // TODO move this check earlier and don't compile all of them if there is only one
        match (predicate_asm, contract_asm, script_asm, library_exports) {
            (Some(pred), None, None, o) if o.trees.is_empty() => CompilationResult::Success {
                asm: pred,
                warnings,
            },
            (None, Some(contract), None, o) if o.trees.is_empty() => CompilationResult::Success {
                asm: contract,
                warnings,
            },
            (None, None, Some(script), o) if o.trees.is_empty() => CompilationResult::Success {
                asm: script,
                warnings,
            },
            (None, None, None, o) if !o.trees.is_empty() => CompilationResult::Library {
                warnings,
                exports: o,
            },
            (None, None, None, o) if o.trees.is_empty() => {
                todo!("do we want empty files to be valid programs?")
            }
            // Default to compiling an empty library if there is no code or invalid state
            _ => unimplemented!(
                "Multiple contracts, libraries, scripts, or predicates in a single file are \
                 unsupported."
            ),
        }
    } else {
        CompilationResult::Failure { errors, warnings }
    }
}
pub fn compile_to_bytecode<'sc, 'manifest>(
    input: &'sc str,
    initial_namespace: &Namespace<'sc>,
    build_config: BuildConfig,
) -> BytecodeCompilationResult<'sc> {
    match compile_to_asm(input, initial_namespace, build_config) {
        CompilationResult::Success {
            mut asm,
            mut warnings,
        } => {
            let mut asm_res = asm.to_bytecode();
            warnings.append(&mut asm_res.warnings);
            if asm_res.value.is_none() || !asm_res.errors.is_empty() {
                BytecodeCompilationResult::Failure {
                    warnings,
                    errors: asm_res.errors,
                }
            } else {
                // asm_res is confirmed to be Some(bytes).
                BytecodeCompilationResult::Success {
                    bytes: asm_res.value.unwrap(),
                    warnings,
                }
            }
        }
        CompilationResult::Failure { warnings, errors } => {
            BytecodeCompilationResult::Failure { warnings, errors }
        }
        CompilationResult::Library {
            warnings,
            exports: _exports,
        } => BytecodeCompilationResult::Library { warnings },
    }
}

fn perform_control_flow_analysis<'sc>(
    tree: &Option<TypedParseTree<'sc>>,
    tree_type: TreeType,
    dead_code_graph: &mut ControlFlowGraph<'sc>,
) -> (Vec<CompileWarning<'sc>>, Vec<CompileError<'sc>>) {
    match tree {
        Some(tree) => {
            match ControlFlowGraph::append_to_dead_code_graph(tree, tree_type, dead_code_graph) {
                Ok(_) => (),
                Err(e) => return (vec![], vec![e]),
            }
            let mut warnings = vec![];
            let mut errors = vec![];
            warnings.append(&mut dead_code_graph.find_dead_code());
            let graph = ControlFlowGraph::construct_return_path_graph(tree);
            errors.append(&mut graph.analyze_return_paths());
            (warnings, errors)
        }
        None => (vec![], vec![]),
    }
}
fn perform_control_flow_analysis_on_library_exports<'sc>(
    lib: &LibraryExports<'sc>,
    dead_code_graph: &mut ControlFlowGraph<'sc>,
) -> (Vec<CompileWarning<'sc>>, Vec<CompileError<'sc>>) {
    let mut warnings = vec![];
    let mut errors = vec![];
    for tree in &lib.trees {
        match ControlFlowGraph::append_to_dead_code_graph(tree, TreeType::Library, dead_code_graph)
        {
            Ok(_) => (),
            Err(e) => return (vec![], vec![e]),
        }
        warnings.append(&mut dead_code_graph.find_dead_code());
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
                Rule::include_statement => {
                    // parse the include statement into a reference to a specific file
                    let include_statement = eval!(
                        IncludeStatement::parse_from_pair,
                        warnings,
                        errors,
                        pair,
                        continue
                    );
                    parse_tree.push(AstNode {
                        content: AstNodeContent::IncludeStatement(include_statement),
                        span: pair.as_span(),
                    });
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
                fuel_ast.library_exports.push((
                    library_name.expect(
                        "Safe unwrap, because the core_lang enforces the library keyword is \
                         followed by a name. This is an invariant",
                    ),
                    parse_tree,
                ));
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
        contract;

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
    "#,
    );
    dbg!(&prog);
    prog.unwrap();
}
#[test]
fn test_parenthesized() {
    let prog = parse(
        r#"
        contract;
        pub fn some_abi_func() -> unit {
            let x = (5 + 6 / (1 + (2 / 1) + 4));
            return;
        }
    "#,
    );
    prog.unwrap();
}
