#[macro_use]
pub mod error;

mod asm_generation;
mod asm_lang;
mod build_config;
mod concurrent_slab;
pub mod constants;
mod control_flow_analysis;
mod convert_parse_tree;
mod optimize;
pub mod parse_tree;
pub mod semantic_analysis;
pub mod source_map;
mod style;
pub mod type_engine;

use crate::{
    asm_generation::{checks, compile_ast_to_asm},
    error::*,
    source_map::SourceMap,
};
pub use asm_generation::{AbstractInstructionSet, FinalizedAsm, SwayAsmSet};
pub use build_config::BuildConfig;
use control_flow_analysis::{ControlFlowGraph, Graph};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub use semantic_analysis::{
    namespace::{self, Namespace},
    TypedDeclaration, TypedFunctionDeclaration, TypedParseTree, TypedProgram, TypedProgramKind,
};
pub mod types;
pub use crate::parse_tree::{
    Declaration, Expression, ParseModule, ParseProgram, TreeType, UseStatement, WhileLoop, *,
};

pub use error::{CompileError, CompileResult, CompileWarning};
use sway_types::{ident::Ident, span, Spanned};
pub use type_engine::TypeInfo;

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
/// Panics if the parser panics.
pub fn parse(input: Arc<str>, config: Option<&BuildConfig>) -> CompileResult<ParseProgram> {
    match config {
        None => parse_in_memory(input),
        Some(config) => parse_files(input, config),
    }
}

/// When no `BuildConfig` is given, we're assumed to be parsing in-memory with no submodules.
fn parse_in_memory(src: Arc<str>) -> CompileResult<ParseProgram> {
    let module = match sway_parse::parse_file(src, None) {
        Ok(module) => module,
        Err(error) => return err(vec![], parse_file_error_to_compile_errors(error)),
    };
    convert_parse_tree::convert_parse_tree(module).flat_map(|(kind, tree)| {
        let submodules = Default::default();
        let root = ParseModule { tree, submodules };
        let program = ParseProgram { kind, root };
        ok(program, vec![], vec![])
    })
}

/// When a `BuildConfig` is given, the module source may declare `dep`s that must be parsed from
/// other files.
fn parse_files(src: Arc<str>, config: &BuildConfig) -> CompileResult<ParseProgram> {
    parse_module_tree(src, config.path()).flat_map(|(kind, root)| {
        let program = ParseProgram { kind, root };
        ok(program, vec![], vec![])
    })
}

/// Given the source of the module along with its path, parse this module including all of its
/// submodules.
fn parse_module_tree(src: Arc<str>, path: Arc<PathBuf>) -> CompileResult<(TreeType, ParseModule)> {
    // Parse this module first.
    let module = match sway_parse::parse_file(src, Some(path.clone())) {
        Ok(module) => module,
        Err(error) => return err(vec![], parse_file_error_to_compile_errors(error)),
    };
    let module_dir = path.parent().expect("module file has no parent directory");

    // Parse all submodules before converting to the `ParseTree`.
    let init_res = ok(vec![], vec![], vec![]);
    let submodules_res = module.dependencies.iter().fold(init_res, |res, dep| {
        let dep_path = Arc::new(module_path(module_dir, dep));
        let dep_str: Arc<str> = match std::fs::read_to_string(&*dep_path) {
            Ok(s) => Arc::from(s),
            Err(e) => {
                let error = CompileError::FileCouldNotBeRead {
                    span: dep.path.span(),
                    file_path: dep_path.to_string_lossy().to_string(),
                    stringified_error: e.to_string(),
                };
                return res.flat_map(|_| err(vec![], vec![error]));
            }
        };
        parse_module_tree(dep_str.clone(), dep_path.clone()).flat_map(|(kind, module)| {
            let library_name = match kind {
                TreeType::Library { name } => name,
                _ => {
                    let span = span::Span::new(dep_str, 0, 0, Some(dep_path)).unwrap();
                    let error = CompileError::ImportMustBeLibrary { span };
                    return err(vec![], vec![error]);
                }
            };
            let submodule = ParseSubmodule {
                library_name: library_name.to_string(),
                module,
            };
            // NOTE: Typed `IncludStatement`'s include an `alias` field, however its only
            // constructor site is always `None`. If we introduce dep aliases in the future, this
            // is where we should use it.
            let dep_alias = None;
            let dep_name = match dep_alias {
                None => library_name.clone(),
                Some(alias) => alias,
            };
            res.flat_map(|mut submods| {
                submods.push((dep_name, submodule));
                ok(submods, vec![], vec![])
            })
        })
    });

    // Convert from the raw parsed module to the `ParseTree` ready for type-check.
    convert_parse_tree::convert_parse_tree(module).flat_map(|(prog_kind, tree)| {
        submodules_res.flat_map(|submodules| {
            let parse_module = ParseModule { tree, submodules };
            ok((prog_kind, parse_module), vec![], vec![])
        })
    })
}

fn module_path(parent_module_dir: &Path, dep: &sway_parse::Dependency) -> PathBuf {
    parent_module_dir
        .iter()
        .chain(dep.path.span().as_str().split('/').map(AsRef::as_ref))
        .collect()
}

fn parse_file_error_to_compile_errors(error: sway_parse::ParseFileError) -> Vec<CompileError> {
    match error {
        sway_parse::ParseFileError::Lex(error) => vec![CompileError::Lex { error }],
        sway_parse::ParseFileError::Parse(errors) => errors
            .into_iter()
            .map(|error| CompileError::Parse { error })
            .collect(),
    }
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
        namespace: Box<namespace::Root>,
        warnings: Vec<CompileWarning>,
    },
    Failure {
        warnings: Vec<CompileWarning>,
        errors: Vec<CompileError>,
    },
}

pub enum CompileAstResult {
    Success {
        typed_program: Box<TypedProgram>,
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

/// For internal compiler use.
/// Compiles an included file and creates its control flow and dead code graphs.
/// These graphs are merged into the parent program's graphs for accurate analysis.
///
/// TODO -- there is _so_ much duplicated code and messiness in this file around the
/// different types of compilation and stuff. After we get to a good state with the MVP,
/// clean up the types here with the power of hindsight
pub(crate) fn compile_inner_dependency(
    input: Arc<str>,
    alias: Option<&Ident>,
    dep_build_config: BuildConfig,
    parent_namespace: &mut Namespace,
    dead_code_graph: &mut ControlFlowGraph,
) -> CompileResult<()> {
    let mut warnings = vec![];
    let mut errors = vec![];

    // Parse the file.
    let parse_program = check!(
        parse(input.clone(), Some(&dep_build_config)),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Check we have a library, and retrieve the name.
    let library_name = match &parse_program.kind {
        TreeType::Library { name } => name.clone(),
        TreeType::Contract | TreeType::Script | TreeType::Predicate => {
            errors.push(CompileError::ImportMustBeLibrary {
                span: span::Span::new(input, 0, 0, Some(dep_build_config.path())).unwrap(),
            });
            return err(warnings, errors);
        }
    };

    // Fetch the name for the dep library module.
    let dep_name = match alias {
        Some(alias) => alias.clone(),
        None => library_name,
    };

    let mut dep_namespace = parent_namespace.enter_submodule(dep_name);

    // Type-check the module, populating its namespace.
    let typed_parse_tree = check!(
        TypedParseTree::type_check(
            parse_program.root.tree,
            &mut dep_namespace,
            &parse_program.kind,
            &dep_build_config,
            dead_code_graph,
        ),
        return err(warnings, errors),
        warnings,
        errors
    );

    // look for return path errors
    let graph = ControlFlowGraph::construct_return_path_graph(typed_parse_tree.all_nodes());
    errors.append(&mut graph.analyze_return_paths());

    // The dead code will be analyzed later wholistically with the rest of the program
    // since we can't tell what is dead and what isn't just from looking at this file
    if let Err(e) = ControlFlowGraph::append_module_to_dead_code_graph(
        typed_parse_tree.all_nodes(),
        &parse_program.kind,
        dead_code_graph,
    ) {
        errors.push(e)
    };

    ok((), warnings, errors)
}

pub fn compile_to_ast(
    input: Arc<str>,
    initial_namespace: namespace::Module,
    build_config: &BuildConfig,
) -> CompileAstResult {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    let CompileResult {
        value: parse_program_opt,
        warnings: new_warnings,
        errors: new_errors,
    } = parse(input, Some(build_config));
    warnings.extend(new_warnings);
    errors.extend(new_errors);
    let parse_program = match parse_program_opt {
        Some(parse_program) => parse_program,
        None => {
            errors = dedup_unsorted(errors);
            warnings = dedup_unsorted(warnings);
            return CompileAstResult::Failure { errors, warnings };
        }
    };

    let mut dead_code_graph = ControlFlowGraph {
        graph: Graph::new(),
        entry_points: vec![],
        namespace: Default::default(),
    };

    let CompileResult {
        value: typed_program_result,
        warnings: new_warnings,
        errors: new_errors,
    } = TypedProgram::type_check(
        parse_program,
        initial_namespace,
        &build_config.clone(),
        &mut dead_code_graph,
    );
    warnings.extend(new_warnings);
    errors.extend(new_errors);
    let typed_program = match typed_program_result {
        Some(typed_program) => typed_program,
        None => {
            errors = dedup_unsorted(errors);
            warnings = dedup_unsorted(warnings);
            return CompileAstResult::Failure { errors, warnings };
        }
    };

    let (mut l_warnings, mut l_errors) =
        perform_control_flow_analysis(&typed_program, &mut dead_code_graph);

    errors.append(&mut l_errors);
    warnings.append(&mut l_warnings);
    errors = dedup_unsorted(errors);
    warnings = dedup_unsorted(warnings);
    if !errors.is_empty() {
        return CompileAstResult::Failure { errors, warnings };
    }

    CompileAstResult::Success {
        typed_program: Box::new(typed_program),
        warnings,
    }
}

/// Given input Sway source code, compile to a [CompilationResult] which contains the asm in opcode
/// form (not raw bytes/bytecode).
pub fn compile_to_asm(
    input: Arc<str>,
    initial_namespace: namespace::Module,
    build_config: BuildConfig,
) -> CompilationResult {
    let ast_res = compile_to_ast(input, initial_namespace, &build_config);
    ast_to_asm(ast_res, &build_config)
}

/// Given an AST compilation result, compile to a [CompilationResult] which contains the asm in
/// opcode form (not raw bytes/bytecode).
pub fn ast_to_asm(ast_res: CompileAstResult, build_config: &BuildConfig) -> CompilationResult {
    match ast_res {
        CompileAstResult::Failure { warnings, errors } => {
            CompilationResult::Failure { warnings, errors }
        }
        CompileAstResult::Success {
            typed_program,
            mut warnings,
        } => {
            let mut errors = vec![];
            let tree_type = typed_program.kind.tree_type();
            match tree_type {
                TreeType::Contract | TreeType::Script | TreeType::Predicate => {
                    let asm = check!(
                        if build_config.use_orig_asm {
                            compile_ast_to_asm(*typed_program, build_config)
                        } else {
                            compile_ast_to_ir_to_asm(*typed_program, build_config)
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
                    namespace: Box::new(typed_program.root.namespace.into()),
                },
            }
        }
    }
}

use sway_ir::{context::Context, function::Function};

pub(crate) fn compile_ast_to_ir_to_asm(
    program: TypedProgram,
    build_config: &BuildConfig,
) -> CompileResult<FinalizedAsm> {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    // the IR pipeline relies on type information being fully resolved.
    // If type information is found to still be generic or unresolved inside of
    // IR, this is considered an internal compiler error. To resolve this situation,
    // we need to explicitly ensure all types are resolved before going into IR.
    //
    // We _could_ introduce a new type here that uses TypeInfo instead of TypeId and throw away
    // the engine, since we don't need inference for IR. That'd be a _lot_ of copy-pasted code,
    // though, so instead, we are just going to do a pass and throw any unresolved generics as
    // errors and then hold as a runtime invariant that none of the types will be unresolved in the
    // IR phase.

    check!(
        program.finalize_types(),
        return err(warnings, errors),
        warnings,
        errors
    );

    let tree_type = program.kind.tree_type();
    let mut ir = match optimize::compile_program(program) {
        Ok(ir) => ir,
        Err(e) => {
            errors.push(e);
            return err(warnings, errors);
        }
    };

    // Inline function calls since we don't support them yet.  For scripts and predicates we inline
    // into main(), and for contracts we inline into ABI impls, which are found due to them having
    // a selector.
    let mut functions_to_inline_to = Vec::new();
    for (idx, fc) in &ir.functions {
        if (matches!(tree_type, TreeType::Script | TreeType::Predicate)
            && fc.name == crate::constants::DEFAULT_ENTRY_POINT_FN_NAME)
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
        tracing::info!("{}", ir);
    }

    crate::asm_generation::from_ir::compile_ir_to_asm(&ir, build_config)
}

fn inline_function_calls(ir: &mut Context, functions: &[Function]) -> CompileResult<()> {
    for function in functions {
        if let Err(ir_error) = sway_ir::optimize::inline_all_function_calls(ir, function) {
            return err(
                Vec::new(),
                vec![CompileError::InternalOwned(
                    ir_error.to_string(),
                    span::Span::new("".into(), 0, 0, None).unwrap(),
                )],
            );
        }
    }
    ok((), Vec::new(), Vec::new())
}

fn combine_constants(ir: &mut Context, functions: &[Function]) -> CompileResult<()> {
    for function in functions {
        if let Err(ir_error) = sway_ir::optimize::combine_constants(ir, function) {
            return err(
                Vec::new(),
                vec![CompileError::InternalOwned(
                    ir_error.to_string(),
                    span::Span::new("".into(), 0, 0, None).unwrap(),
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
    initial_namespace: namespace::Module,
    build_config: BuildConfig,
    source_map: &mut SourceMap,
) -> BytecodeCompilationResult {
    let asm_res = compile_to_asm(input, initial_namespace, build_config);
    asm_to_bytecode(asm_res, source_map)
}

/// Given a [CompilationResult] containing the assembly (opcodes), compile to a
/// [BytecodeCompilationResult] which contains the asm in bytecode form.
pub fn asm_to_bytecode(
    asm_res: CompilationResult,
    source_map: &mut SourceMap,
) -> BytecodeCompilationResult {
    match asm_res {
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

/// Given a [TypedProgram], which is type-checked Sway source, construct a graph to analyze
/// control flow and determine if it is valid.
fn perform_control_flow_analysis(
    program: &TypedProgram,
    dead_code_graph: &mut ControlFlowGraph,
) -> (Vec<CompileWarning>, Vec<CompileError>) {
    // TODO: Append all submodules, not just root.
    let tree_type = program.kind.tree_type();

    match ControlFlowGraph::append_module_to_dead_code_graph(
        &program.root.all_nodes,
        &tree_type,
        dead_code_graph,
    ) {
        Ok(_) => (),
        Err(e) => return (vec![], vec![e]),
    }
    let mut warnings = vec![];
    let mut errors = vec![];
    warnings.append(&mut dead_code_graph.find_dead_code());
    let graph = ControlFlowGraph::construct_return_path_graph(&program.root.all_nodes);
    errors.append(&mut graph.analyze_return_paths());
    (warnings, errors)
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
