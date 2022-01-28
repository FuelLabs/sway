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
mod optimize;
pub mod parse_tree;
mod parser;
pub mod semantic_analysis;
pub mod source_map;
mod style;
pub mod type_engine;

pub use crate::parser::{Rule, SwayParser};
use crate::{
    asm_generation::{checks, compile_ast_to_asm},
    error::*,
    source_map::SourceMap,
};
pub use asm_generation::{AbstractInstructionSet, FinalizedAsm, SwayAsmSet};
pub use build_config::BuildConfig;
use control_flow_analysis::{ControlFlowGraph, Graph};
use pest::iterators::Pair;
use pest::Parser;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub use semantic_analysis::{
    create_module, retrieve_module, Namespace, NamespaceRef, NamespaceWrapper, TreeType,
    TypedDeclaration, TypedFunctionDeclaration, TypedParseTree,
};
pub mod types;
pub use crate::parse_tree::{Declaration, Expression, UseStatement, WhileLoop, *};

pub use error::{CompileError, CompileResult, CompileWarning};
use sway_types::{ident::Ident, span};
pub use type_engine::TypeInfo;

/// Represents a parsed, but not yet type-checked, Sway program.
/// A Sway program can be either a contract, script, predicate, or
/// it can be a library to be imported into one of the aforementioned
/// program types.
#[derive(Debug)]
pub struct SwayParseTree {
    pub tree_type: TreeType,
    pub tree: ParseTree,
}

/// Represents some exportable information that results from compiling some
/// Sway source code.
#[derive(Debug)]
pub struct ParseTree {
    /// The untyped AST nodes that constitute this tree's root nodes.
    pub root_nodes: Vec<AstNode>,
    /// The [span::Span] of the entire tree.
    pub span: span::Span,
}

/// A single [AstNode] represents a node in the parse tree. Note that [AstNode]
/// is a recursive type and can contain other [AstNode], thus populating the tree.
#[derive(Debug, Clone)]
pub struct AstNode {
    /// The content of this ast node, which could be any control flow structure or other
    /// basic organizational component.
    pub content: AstNodeContent,
    /// The [span::Span] representing this entire [AstNode].
    pub span: span::Span,
}

/// Represents the various structures that constitute a Sway program.
#[derive(Debug, Clone)]
pub enum AstNodeContent {
    /// A statement of the form `use foo::bar;` or `use ::foo::bar;`
    UseStatement(UseStatement),
    /// A statement of the form `return foo;`
    ReturnStatement(ReturnStatement),
    /// Any type of declaration, of which there are quite a few. See [Declaration] for more details
    /// on the possible variants.
    Declaration(Declaration),
    /// Any type of expression, of which there are quite a few. See [Expression] for more details.
    Expression(Expression),
    /// An implicit return expression is different from a [AstNodeContent::ReturnStatement] because
    /// it is not a control flow item. Therefore it is a different variant.
    ///
    /// An implicit return expression is an [Expression] at the end of a code block which has no
    /// semicolon, denoting that it is the [Expression] to be returned from that block.
    ImplicitReturnExpression(Expression),
    /// A control flow element which loops continually until some boolean expression evaluates as
    /// `false`.
    WhileLoop(WhileLoop),
    /// A statement of the form `dep foo::bar;` which imports/includes another source file.
    IncludeStatement(IncludeStatement),
}

impl ParseTree {
    /// Create a new, empty, [ParseTree] from a span which represents the source code that it will
    /// cover.
    pub(crate) fn new(span: span::Span) -> Self {
        ParseTree {
            root_nodes: Vec::new(),
            span,
        }
    }

    /// Push a new [AstNode] on to the end of a [ParseTree]'s root nodes.
    pub(crate) fn push(&mut self, new_node: AstNode) {
        self.root_nodes.push(new_node);
    }
}

/// Given an input `Arc<str>` and an optional [BuildConfig], parse the input into a [SwayParseTree].
///
/// # Example
/// ```
/// # use sway_core::parse;
/// # fn main() {
///     let input = "script; fn main() -> bool { true }";
///     let result = parse(input.into(), Default::default());
/// # }
/// ```
///
/// # Panics
/// Panics if the generated parser from Pest panics.
pub fn parse(input: Arc<str>, config: Option<&BuildConfig>) -> CompileResult<SwayParseTree> {
    let mut warnings: Vec<CompileWarning> = Vec::new();
    let mut errors: Vec<CompileError> = Vec::new();
    let mut parsed = match SwayParser::parse(Rule::program, input.clone()) {
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
    let parsed_root = check!(
        parse_root_from_pairs(parsed.next().unwrap().into_inner(), config),
        return err(warnings, errors),
        warnings,
        errors
    );
    ok(parsed_root, warnings, errors)
}

/// Represents the result of compiling Sway code via [compile_to_asm].
/// Contains the compiled assets or resulting errors, and any warnings generated.
pub enum CompilationResult {
    Success {
        asm: FinalizedAsm,
        warnings: Vec<CompileWarning>,
    },
    Library {
        name: Ident,
        namespace: NamespaceRef,
        warnings: Vec<CompileWarning>,
    },
    Failure {
        warnings: Vec<CompileWarning>,
        errors: Vec<CompileError>,
    },
}

pub enum CompileAstResult {
    Success {
        parse_tree: Box<TypedParseTree>,
        tree_type: TreeType,
        warnings: Vec<CompileWarning>,
    },
    Failure {
        warnings: Vec<CompileWarning>,
        errors: Vec<CompileError>,
    },
}

/// Represents the result of compiling Sway code via [compile_to_bytecode].
/// Contains the compiled bytecode in byte form, or resulting errors, and any warnings generated.
pub enum BytecodeCompilationResult {
    Success {
        bytes: Vec<u8>,
        warnings: Vec<CompileWarning>,
    },
    Library {
        warnings: Vec<CompileWarning>,
    },
    Failure {
        warnings: Vec<CompileWarning>,
        errors: Vec<CompileError>,
    },
}

/// If a given [Rule] exists in the input text, return
/// that string trimmed. Otherwise, return `None`. This is typically used to find keywords.
pub fn extract_keyword(line: &str, rule: Rule) -> Option<String> {
    if let Ok(pair) = SwayParser::parse(rule, Arc::from(line)) {
        Some(pair.as_str().trim().to_string())
    } else {
        None
    }
}

/// Takes a parse failure as input and returns either the index of the positional pest parse error, or the start position of the span of text that the error occurs.
fn get_start(err: &pest::error::Error<Rule>) -> usize {
    match err.location {
        pest::error::InputLocation::Pos(num) => num,
        pest::error::InputLocation::Span((start, _)) => start,
    }
}

/// Takes a parse failure as input and returns either the index of the positional pest parse error, or the end position of the span of text that the error occurs.
fn get_end(err: &pest::error::Error<Rule>) -> usize {
    match err.location {
        pest::error::InputLocation::Pos(num) => num,
        pest::error::InputLocation::Span((_, end)) => end,
    }
}

/// This struct represents the compilation of an internal dependency
/// defined through an include statement (the `dep` keyword).
pub(crate) struct InnerDependencyCompileResult {
    name: Ident,
    namespace: Namespace,
}
/// For internal compiler use.
/// Compiles an included file and returns its control flow and dead code graphs.
/// These graphs are merged into the parent program's graphs for accurate analysis.
///
/// TODO -- there is _so_ much duplicated code and messiness in this file around the
/// different types of compilation and stuff. After we get to a good state with the MVP,
/// clean up the types here with the power of hindsight
pub(crate) fn compile_inner_dependency(
    input: Arc<str>,
    initial_namespace: NamespaceRef,
    build_config: BuildConfig,
    dead_code_graph: &mut ControlFlowGraph,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
) -> CompileResult<InnerDependencyCompileResult> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let parse_tree = check!(
        parse(input.clone(), Some(&build_config)),
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
                    path: Some(build_config.path()),
                },
            });
            return err(warnings, errors);
        }
    };
    let typed_parse_tree = check!(
        TypedParseTree::type_check(
            parse_tree.tree,
            initial_namespace,
            initial_namespace,
            &parse_tree.tree_type,
            &build_config,
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

    ok(
        InnerDependencyCompileResult {
            name: library_name.clone(),
            namespace: typed_parse_tree.into_namespace(),
        },
        warnings,
        errors,
    )
}

pub fn compile_to_ast(
    input: Arc<str>,
    initial_namespace: crate::semantic_analysis::NamespaceRef,
    build_config: &BuildConfig,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
) -> CompileAstResult {
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
            initial_namespace,
            initial_namespace,
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
        parse_tree: Box::new(typed_parse_tree),
        tree_type: parse_tree.tree_type,
        warnings,
    }
}

/// Given input Sway source code, compile to a [CompilationResult] which contains the asm in opcode
/// form (not raw bytes/bytecode).
pub fn compile_to_asm(
    input: Arc<str>,
    initial_namespace: crate::semantic_analysis::NamespaceRef,
    build_config: BuildConfig,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
) -> CompilationResult {
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
                        if build_config.use_ir {
                            compile_ast_to_ir_to_asm(*parse_tree, tree_type, &build_config)
                        } else {
                            compile_ast_to_asm(*parse_tree, &build_config)
                        },
                        return CompilationResult::Failure { errors, warnings },
                        warnings,
                        errors
                    );
                    if !errors.is_empty() {
                        return CompilationResult::Failure { errors, warnings };
                    }
                    CompilationResult::Success { asm, warnings }
                }
                TreeType::Library { name } => CompilationResult::Library {
                    warnings,
                    name,
                    namespace: parse_tree.get_namespace_ref(),
                },
            }
        }
    }
}

use sway_ir::{context::Context, function::Function};

pub(crate) fn compile_ast_to_ir_to_asm(
    ast: TypedParseTree,
    tree_type: TreeType,
    build_config: &BuildConfig,
) -> CompileResult<FinalizedAsm> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    let mut ir = match optimize::compile_ast(ast) {
        Ok(ir) => ir,
        Err(msg) => {
            errors.push(CompileError::InternalOwned(
                msg,
                span::Span {
                    span: pest::Span::new(" ".into(), 0, 0).unwrap(),
                    path: None,
                },
            ));
            return err(warnings, errors);
        }
    };

    // Inline function calls since we don't support them yet.  For scripts and predicates we inline
    // into main(), and for contracts we inline into ABI impls, which are found due to them having
    // a selector.
    let mut functions_to_inline_to = Vec::new();
    for (idx, fc) in &ir.functions {
        if (matches!(tree_type, TreeType::Script | TreeType::Predicate) && fc.name == "main")
            || (tree_type == TreeType::Contract && fc.selector.is_some())
        {
            functions_to_inline_to.push(::sway_ir::function::Function(idx));
        }
    }
    check!(
        inline_function_calls(&mut ir, &functions_to_inline_to),
        return err(warnings, errors),
        warnings,
        errors
    );

    // The only other optimisation we have at the moment is constant combining.  In lieu of a
    // forthcoming pass manager we can just call it here now.  We can re-use the inline functions
    // list.
    check!(
        combine_constants(&mut ir, &functions_to_inline_to),
        return err(warnings, errors),
        warnings,
        errors
    );

    if build_config.print_ir {
        println!("{}", ir);
    }

    crate::asm_generation::from_ir::compile_ir_to_asm(&ir, build_config)
}

fn inline_function_calls(ir: &mut Context, functions: &[Function]) -> CompileResult<()> {
    for function in functions {
        if let Err(msg) = sway_ir::optimize::inline_all_function_calls(ir, function) {
            return err(
                Vec::new(),
                vec![CompileError::InternalOwned(
                    msg,
                    span::Span {
                        span: pest::Span::new("".into(), 0, 0).unwrap(),
                        path: None,
                    },
                )],
            );
        }
    }
    ok((), Vec::new(), Vec::new())
}

fn combine_constants(ir: &mut Context, functions: &[Function]) -> CompileResult<()> {
    for function in functions {
        if let Err(msg) = sway_ir::optimize::combine_constants(ir, function) {
            return err(
                Vec::new(),
                vec![CompileError::InternalOwned(
                    msg,
                    span::Span {
                        span: pest::Span::new("".into(), 0, 0).unwrap(),
                        path: None,
                    },
                )],
            );
        }
    }
    ok((), Vec::new(), Vec::new())
}

/// Given input Sway source code, compile to a [BytecodeCompilationResult] which contains the asm in
/// bytecode form.
pub fn compile_to_bytecode(
    input: Arc<str>,
    initial_namespace: crate::semantic_analysis::NamespaceRef,
    build_config: BuildConfig,
    dependency_graph: &mut HashMap<String, HashSet<String>>,
    source_map: &mut SourceMap,
) -> BytecodeCompilationResult {
    match compile_to_asm(input, initial_namespace, build_config, dependency_graph) {
        CompilationResult::Success {
            mut asm,
            mut warnings,
        } => {
            let mut asm_res = asm.to_bytecode_mut(source_map);
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
        CompilationResult::Library { warnings, .. } => {
            BytecodeCompilationResult::Library { warnings }
        }
    }
}

/// Given a [TypedParseTree], which is type-checked Sway source, construct a graph to analyze
/// control flow and determine if it is valid.
fn perform_control_flow_analysis(
    tree: &TypedParseTree,
    tree_type: &TreeType,
    dead_code_graph: &mut ControlFlowGraph,
) -> (Vec<CompileWarning>, Vec<CompileError>) {
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

/// The basic recursive parser which handles the top-level parsing given the output of the
/// pest-generated parser.
fn parse_root_from_pairs(
    input: impl Iterator<Item = Pair<Rule>>,
    config: Option<&BuildConfig>,
) -> CompileResult<SwayParseTree> {
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
                    for entry in stmt {
                        parse_tree.push(AstNode {
                            content: AstNodeContent::UseStatement(entry.clone()),
                            span: span::Span {
                                span: pair.as_span(),
                                path: path.clone(),
                            },
                        });
                    }
                }
                Rule::library_name => {
                    let lib_pair = pair.into_inner().next().unwrap();
                    library_name = Some(check!(
                        parse_tree::ident::parse_from_pair(lib_pair, config),
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
                fuel_ast_opt = Some(SwayParseTree {
                    tree_type: TreeType::Contract,
                    tree: parse_tree,
                });
            }
            Rule::script => {
                fuel_ast_opt = Some(SwayParseTree {
                    tree_type: TreeType::Script,
                    tree: parse_tree,
                });
            }
            Rule::predicate => {
                fuel_ast_opt = Some(SwayParseTree {
                    tree_type: TreeType::Predicate,
                    tree: parse_tree,
                });
            }
            Rule::library => {
                fuel_ast_opt = Some(SwayParseTree {
                    tree_type: TreeType::Library {
                        name: library_name.expect(
                            "Safe unwrap, because the sway-core enforces the library keyword is \
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
    "#
        .into(),
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
    "#
        .into(),
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
    }"#
        .into(),
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
