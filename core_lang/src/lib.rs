#[macro_use]
extern crate pest_derive;
#[macro_use]
pub mod error;

mod asm_generation;
mod asm_lang;
mod build_config;
mod concurrent_slab;
pub mod constants;
mod control_flow_analysis;
mod ident;
pub mod parse_tree;
mod parser;
pub mod semantic_analysis;
mod span;
mod style;
pub(crate) mod type_engine;

use crate::asm_generation::checks::check_invalid_opcodes;
pub use crate::parse_tree::*;
pub use crate::parser::{HllParser, Rule};
use crate::{asm_generation::compile_ast_to_asm, error::*};
pub use asm_generation::{AbstractInstructionSet, FinalizedAsm, HllAsmSet};
pub use build_config::BuildConfig;
use control_flow_analysis::{ControlFlowGraph, Graph};
use pest::iterators::Pair;
use pest::Parser;
use std::collections::{HashMap, HashSet};

pub use semantic_analysis::TreeType;
pub use semantic_analysis::TypedParseTree;
pub mod types;
pub(crate) mod utils;
pub use crate::parse_tree::{Declaration, Expression, UseStatement, WhileLoop};

pub use crate::span::Span;
pub use error::{CompileError, CompileResult, CompileWarning};
pub use ident::Ident;
pub use semantic_analysis::{Namespace, TypedDeclaration, TypedFunctionDeclaration};
pub use type_engine::TypeInfo;

// todo rename to language name
#[derive(Debug)]
pub struct HllParseTree<'sc> {
    pub tree_type: TreeType<'sc>,
    pub tree: ParseTree<'sc>,
}

#[derive(Debug)]
pub struct HllTypedParseTree<'sc> {
    pub library_exports: LibraryExports<'sc>,
}

#[derive(Debug)]
pub struct LibraryExports<'sc> {
    pub namespace: Namespace<'sc>,
    pub trees: Vec<TypedParseTree<'sc>>,
}

#[derive(Debug)]
pub struct ParseTree<'sc> {
    /// In a typical programming language, you might have a single root node for your syntax tree.
    /// In this language however, we want to expose multiple public functions at the root
    /// level so the tree is multi-root.
    pub root_nodes: Vec<AstNode<'sc>>,
    pub span: span::Span<'sc>,
}

#[derive(Debug, Clone)]
pub struct AstNode<'sc> {
    pub content: AstNodeContent<'sc>,
    pub span: span::Span<'sc>,
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
    pub(crate) fn new(span: span::Span<'sc>) -> Self {
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

pub fn parse<'sc>(
    input: &'sc str,
    config: Option<&BuildConfig>,
) -> CompileResult<'sc, HllParseTree<'sc>> {
    let mut warnings: Vec<CompileWarning> = Vec::new();
    let mut errors: Vec<CompileError> = Vec::new();
    let mut parsed = match HllParser::parse(Rule::program, input) {
        Ok(o) => o,
        Err(e) => {
            return err(
                Vec::new(),
                vec![CompileError::ParseFailure {
                    span: span::Span {
                        span: pest::Span::new(input, get_start(&e), get_end(&e)).unwrap(),
                        path: config.map(|config| config.path()),
                    },
                    err: e,
                }],
            )
        }
    };
    let res = check!(
        parse_root_from_pairs(parsed.next().unwrap().into_inner(), config),
        return err(warnings, errors),
        warnings,
        errors
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

pub enum CompileAstResult<'sc> {
    Success {
        parse_tree: TypedParseTree<'sc>,
        tree_type: TreeType<'sc>,
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
pub(crate) fn compile_inner_dependency<'sc>(
    input: &'sc str,
    initial_namespace: &Namespace<'sc>,
    build_config: BuildConfig,
    dead_code_graph: &mut ControlFlowGraph<'sc>,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
) -> CompileResult<'sc, InnerDependencyCompileResult<'sc>> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let parse_tree = check!(
        parse(input, Some(&build_config)),
        return err(warnings, errors),
        warnings,
        errors
    );
    let library_name = match &parse_tree.tree_type {
        TreeType::Library { name } => name,
        TreeType::Contract | TreeType::Script | TreeType::Predicate => {
            errors.push(CompileError::ImportMustBeLibrary {
                span: span::Span {
                    span: pest::Span::new(input, 0, 0).unwrap(),
                    path: Some(build_config.clone().path()),
                },
            });
            return err(warnings, errors);
        }
    };
    let typed_parse_tree = check!(
        TypedParseTree::type_check(
            parse_tree.tree,
            initial_namespace.clone(),
            &parse_tree.tree_type,
            &build_config.clone(),
            dead_code_graph,
            dependency_graph,
        ),
        return err(warnings, errors),
        warnings,
        errors
    );

    // look for return path errors
    let graph = ControlFlowGraph::construct_return_path_graph(&typed_parse_tree);
    errors.append(&mut graph.analyze_return_paths());

    // The dead code will be analyzed later wholistically with the rest of the program
    // since we can't tell what is dead and what isn't just from looking at this file
    if let Err(e) = ControlFlowGraph::append_to_dead_code_graph(
        &typed_parse_tree,
        &parse_tree.tree_type,
        dead_code_graph,
    ) {
        errors.push(e)
    };

    let mut library_exports = LibraryExports {
        namespace: Default::default(),
        trees: vec![],
    };
    library_exports.namespace.insert_module(
        library_name.primary_name.to_string(),
        typed_parse_tree.namespace().clone(),
    );
    library_exports.trees.push(typed_parse_tree);

    ok(
        InnerDependencyCompileResult { library_exports },
        warnings,
        errors,
    )
}

pub fn compile_to_ast<'sc>(
    input: &'sc str,
    initial_namespace: &Namespace<'sc>,
    build_config: &BuildConfig,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
) -> CompileAstResult<'sc> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let parse_tree = check!(
        parse(input, Some(build_config)),
        return CompileAstResult::Failure { errors, warnings },
        warnings,
        errors
    );
    let mut dead_code_graph = ControlFlowGraph {
        graph: Graph::new(),
        entry_points: vec![],
        namespace: Default::default(),
    };

    let typed_parse_tree = check!(
        TypedParseTree::type_check(
            parse_tree.tree,
            initial_namespace.clone(),
            &parse_tree.tree_type,
            &build_config.clone(),
            &mut dead_code_graph,
            dependency_graph,
        ),
        return CompileAstResult::Failure { errors, warnings },
        warnings,
        errors
    );

    let (mut l_warnings, mut l_errors) = perform_control_flow_analysis(
        &typed_parse_tree,
        &parse_tree.tree_type,
        &mut dead_code_graph,
    );

    errors.append(&mut l_errors);
    warnings.append(&mut l_warnings);
    errors = dedup_unsorted(errors);
    warnings = dedup_unsorted(warnings);

    if !errors.is_empty() {
        return CompileAstResult::Failure { errors, warnings };
    }

    CompileAstResult::Success {
        parse_tree: typed_parse_tree,
        tree_type: parse_tree.tree_type,
        warnings,
    }
}

pub fn compile_to_asm<'sc>(
    input: &'sc str,
    initial_namespace: &Namespace<'sc>,
    build_config: BuildConfig,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
) -> CompilationResult<'sc> {
    match compile_to_ast(input, initial_namespace, &build_config, dependency_graph) {
        CompileAstResult::Failure { warnings, errors } => {
            CompilationResult::Failure { warnings, errors }
        }
        CompileAstResult::Success {
            parse_tree,
            tree_type,
            mut warnings,
        } => {
            let mut errors = vec![];
            match tree_type {
                TreeType::Contract | TreeType::Script | TreeType::Predicate => {
                    let asm = check!(
                        compile_ast_to_asm(parse_tree, &build_config),
                        return CompilationResult::Failure { errors, warnings },
                        warnings,
                        errors
                    );
                    if !errors.is_empty() {
                        return CompilationResult::Failure { errors, warnings };
                    }
                    CompilationResult::Success { asm, warnings }
                }
                TreeType::Library { name } => {
                    let mut exports = LibraryExports {
                        namespace: Default::default(),
                        trees: vec![],
                    };
                    exports.namespace.insert_module(
                        name.primary_name.to_string(),
                        parse_tree.namespace().clone(),
                    );
                    exports.trees.push(parse_tree);
                    CompilationResult::Library { warnings, exports }
                }
            }
        }
    }
}

pub fn compile_to_bytecode<'n, 'sc>(
    input: &'sc str,
    initial_namespace: &Namespace<'sc>,
    build_config: BuildConfig,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
) -> BytecodeCompilationResult<'sc> {
    match compile_to_asm(input, initial_namespace, build_config, dependency_graph) {
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
    tree: &TypedParseTree<'sc>,
    tree_type: &TreeType<'sc>,
    dead_code_graph: &mut ControlFlowGraph<'sc>,
) -> (Vec<CompileWarning<'sc>>, Vec<CompileError<'sc>>) {
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

// strategy: parse top level things
// and if we encounter a function body or block, recursively call this function and build
// sub-nodes
fn parse_root_from_pairs<'sc>(
    input: impl Iterator<Item = Pair<'sc, Rule>>,
    config: Option<&BuildConfig>,
) -> CompileResult<'sc, HllParseTree<'sc>> {
    let path = config.map(|config| config.dir_of_code.clone());
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let mut fuel_ast_opt = None;
    for block in input {
        let mut parse_tree = ParseTree::new(span::Span {
            span: block.as_span(),
            path: path.clone(),
        });
        let rule = block.as_rule();
        let input = block.clone().into_inner();
        let mut library_name = None;
        for pair in input {
            match pair.as_rule() {
                Rule::non_var_decl => {
                    let decl = check!(
                        Declaration::parse_non_var_from_pair(pair.clone(), config),
                        continue,
                        warnings,
                        errors
                    );
                    parse_tree.push(AstNode {
                        content: AstNodeContent::Declaration(decl),
                        span: span::Span {
                            span: pair.as_span(),
                            path: path.clone(),
                        },
                    });
                }
                Rule::use_statement => {
                    let stmt = check!(
                        UseStatement::parse_from_pair(pair.clone(), config),
                        continue,
                        warnings,
                        errors
                    );
                    parse_tree.push(AstNode {
                        content: AstNodeContent::UseStatement(stmt),
                        span: span::Span {
                            span: pair.as_span(),
                            path: path.clone(),
                        },
                    });
                }
                Rule::library_name => {
                    let lib_pair = pair.into_inner().next().unwrap();
                    library_name = Some(check!(
                        Ident::parse_from_pair(lib_pair, config),
                        continue,
                        warnings,
                        errors
                    ));
                }
                Rule::include_statement => {
                    // parse the include statement into a reference to a specific file
                    let include_statement = check!(
                        IncludeStatement::parse_from_pair(pair.clone(), config),
                        continue,
                        warnings,
                        errors
                    );
                    parse_tree.push(AstNode {
                        content: AstNodeContent::IncludeStatement(include_statement),
                        span: span::Span {
                            span: pair.as_span(),
                            path: path.clone(),
                        },
                    });
                }
                _ => unreachable!("{:?}", pair.as_str()),
            }
        }
        match rule {
            Rule::contract => {
                fuel_ast_opt = Some(HllParseTree {
                    tree_type: TreeType::Contract,
                    tree: parse_tree,
                });
            }
            Rule::script => {
                fuel_ast_opt = Some(HllParseTree {
                    tree_type: TreeType::Script,
                    tree: parse_tree,
                });
            }
            Rule::predicate => {
                fuel_ast_opt = Some(HllParseTree {
                    tree_type: TreeType::Predicate,
                    tree: parse_tree,
                });
            }
            Rule::library => {
                fuel_ast_opt = Some(HllParseTree {
                    tree_type: TreeType::Library {
                        name: library_name.expect(
                            "Safe unwrap, because the core_lang enforces the library keyword is \
                             followed by a name. This is an invariant",
                        ),
                    },
                    tree: parse_tree,
                });
            }
            Rule::EOI => (),
            a => errors.push(CompileError::InvalidTopLevelItem(
                a,
                span::Span {
                    span: block.as_span(),
                    path: path.clone(),
                },
            )),
        }
    }

    let fuel_ast = fuel_ast_opt.unwrap();
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
        None,
    );
    let mut warnings: Vec<CompileWarning> = Vec::new();
    let mut errors: Vec<CompileError> = Vec::new();
    prog.unwrap(&mut warnings, &mut errors);
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
        None,
    );
    let mut warnings: Vec<CompileWarning> = Vec::new();
    let mut errors: Vec<CompileError> = Vec::new();
    prog.unwrap(&mut warnings, &mut errors);
}

#[test]
fn test_unary_ordering() {
    use crate::parse_tree::declaration::FunctionDeclaration;
    let prog = parse(
        r#"
    script;
    fn main() -> bool {
        let a = true;
        let b = true;
        !a && b;
    }"#,
        None,
    );
    let mut warnings: Vec<CompileWarning> = Vec::new();
    let mut errors: Vec<CompileError> = Vec::new();
    let prog = prog.unwrap(&mut warnings, &mut errors);
    // this should parse as `(!a) && b`, not `!(a && b)`. So, the top level
    // expression should be `&&`
    if let AstNode {
        content:
            AstNodeContent::Declaration(Declaration::FunctionDeclaration(FunctionDeclaration {
                body,
                ..
            })),
        ..
    } = &prog.tree.root_nodes[0]
    {
        if let AstNode {
            content: AstNodeContent::Expression(Expression::LazyOperator { op, .. }),
            ..
        } = &body.contents[2]
        {
            assert_eq!(op, &LazyOp::And)
        } else {
            panic!("Was not lazy operator.")
        }
    } else {
        panic!("Was not ast node")
    };
}

/// We want compile errors and warnings to retain their ordering, since typically
/// they are grouped by relevance. However, we want to deduplicate them.
/// Stdlib dedup in Rust assumes sorted data for efficiency, but we don't want that.
/// A hash set would also mess up the order, so this is just a brute force way of doing it
/// with a vector.
fn dedup_unsorted<T: PartialEq + std::hash::Hash>(mut data: Vec<T>) -> Vec<T> {
    use smallvec::SmallVec;
    use std::collections::hash_map::{DefaultHasher, Entry};
    use std::hash::Hasher;

    let mut write_index = 0;
    let mut indexes: HashMap<u64, SmallVec<[usize; 1]>> = HashMap::with_capacity(data.len());
    for read_index in 0..data.len() {
        let hash = {
            let mut hasher = DefaultHasher::new();
            data[read_index].hash(&mut hasher);
            hasher.finish()
        };
        let index_vec = match indexes.entry(hash) {
            Entry::Occupied(oe) => {
                if oe
                    .get()
                    .iter()
                    .any(|index| data[*index] == data[read_index])
                {
                    continue;
                }
                oe.into_mut()
            }
            Entry::Vacant(ve) => ve.insert(SmallVec::new()),
        };
        data.swap(write_index, read_index);
        index_vec.push(write_index);
        write_index += 1;
    }
    data.truncate(write_index);
    data
}
