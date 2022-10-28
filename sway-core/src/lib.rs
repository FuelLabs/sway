#[macro_use]
pub mod error;

mod asm_generation;
mod asm_lang;
mod build_config;
mod concurrent_slab;
mod control_flow_analysis;
pub mod declaration_engine;
pub mod ir_generation;
pub mod language;
mod metadata;
pub mod semantic_analysis;
pub mod source_map;
pub mod transform;
pub mod type_system;

use crate::ir_generation::check_function_purity;
use crate::{error::*, source_map::SourceMap};
pub use asm_generation::from_ir::compile_ir_to_asm;
use asm_generation::FinalizedAsm;
pub use build_config::BuildConfig;
use control_flow_analysis::ControlFlowGraph;
use metadata::MetadataManager;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use sway_ast::Dependency;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_ir::{Context, Function, Instruction, Kind, Module, Value};

pub use semantic_analysis::namespace::{self, Namespace};
pub mod types;

pub use error::CompileResult;
use sway_error::error::CompileError;
use sway_error::warning::CompileWarning;
use sway_types::{ident::Ident, span, Spanned};
pub use type_system::*;

use language::{parsed, ty};
use transform::to_parsed_lang;

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
pub fn parse(input: Arc<str>, config: Option<&BuildConfig>) -> CompileResult<parsed::ParseProgram> {
    CompileResult::with_handler(|h| match config {
        None => parse_in_memory(h, input),
        // When a `BuildConfig` is given,
        // the module source may declare `dep`s that must be parsed from other files.
        Some(config) => parse_module_tree(
            h,
            input,
            config.canonical_root_module(),
            config.include_tests,
        )
        .map(|(kind, root)| parsed::ParseProgram { kind, root }),
    })
}

/// When no `BuildConfig` is given, we're assumed to be parsing in-memory with no submodules.
fn parse_in_memory(handler: &Handler, src: Arc<str>) -> Result<parsed::ParseProgram, ErrorEmitted> {
    // Omit tests by default so that a test that fails to typecheck doesn't block the rest of the
    // program (similar behaviour to rust). To include test functions, specify the `include_tests`
    // flag in the `BuildConfig` passed to `parse`.
    let include_test_fns = false;
    let module = sway_parse::parse_file(handler, src, None)?;
    let (kind, tree) = to_parsed_lang::convert_parse_tree(handler, module, include_test_fns)?;
    let submodules = Default::default();
    let root = parsed::ParseModule { tree, submodules };
    Ok(parsed::ParseProgram { kind, root })
}

/// Parse all dependencies `deps` as submodules.
fn parse_submodules(
    handler: &Handler,
    deps: &[Dependency],
    module_dir: &Path,
    include_test_fns: bool,
) -> Vec<(Ident, parsed::ParseSubmodule)> {
    // Assume the happy path, so there'll be as many submodules as dependencies, but no more.
    let mut submods = Vec::with_capacity(deps.len());

    deps.iter().for_each(|dep| {
        // Read the source code from the dependency.
        // If we cannot, record as an error, but continue with other files.
        let dep_path = Arc::new(module_path(module_dir, dep));
        let dep_str: Arc<str> = match std::fs::read_to_string(&*dep_path) {
            Ok(s) => Arc::from(s),
            Err(e) => {
                handler.emit_err(CompileError::FileCouldNotBeRead {
                    span: dep.path.span(),
                    file_path: dep_path.to_string_lossy().to_string(),
                    stringified_error: e.to_string(),
                });
                return;
            }
        };

        if let Ok((kind, module)) =
            parse_module_tree(handler, dep_str.clone(), dep_path.clone(), include_test_fns)
        {
            let library_name = match kind {
                parsed::TreeType::Library { name } => name,
                _ => {
                    let span = span::Span::new(dep_str, 0, 0, Some(dep_path)).unwrap();
                    handler.emit_err(CompileError::ImportMustBeLibrary { span });
                    return;
                }
            };
            // NOTE: Typed `IncludStatement`'s include an `alias` field, however its only
            // constructor site is always `None`. If we introduce dep aliases in the future, this
            // is where we should use it.
            let dep_alias = None;
            let dep_name = dep_alias.unwrap_or_else(|| library_name.clone());
            let submodule = parsed::ParseSubmodule {
                library_name,
                module,
            };
            submods.push((dep_name, submodule));
        }
    });

    submods
}

/// Given the source of the module along with its path,
/// parse this module including all of its submodules.
fn parse_module_tree(
    handler: &Handler,
    src: Arc<str>,
    path: Arc<PathBuf>,
    include_test_fns: bool,
) -> Result<(parsed::TreeType, parsed::ParseModule), ErrorEmitted> {
    // Parse this module first.
    let module_dir = path.parent().expect("module file has no parent directory");
    let module = sway_parse::parse_file(handler, src, Some(path.clone()))?;

    // Parse all submodules before converting to the `ParseTree`.
    // This always recovers on parse errors for the file itself by skipping that file.
    let submodules = parse_submodules(handler, &module.dependencies, module_dir, include_test_fns);

    // Convert from the raw parsed module to the `ParseTree` ready for type-check.
    let (kind, tree) = to_parsed_lang::convert_parse_tree(handler, module, include_test_fns)?;

    Ok((kind, parsed::ParseModule { tree, submodules }))
}

fn module_path(parent_module_dir: &Path, dep: &sway_ast::Dependency) -> PathBuf {
    parent_module_dir
        .iter()
        .chain(dep.path.span().as_str().split('/').map(AsRef::as_ref))
        .collect::<PathBuf>()
        .with_extension(sway_types::constants::DEFAULT_FILE_EXTENSION)
}

pub struct CompiledAsm(pub FinalizedAsm);

/// The bytecode for a sway program.
pub struct CompiledBytecode(pub Vec<u8>);

pub fn parsed_to_ast(
    parse_program: &parsed::ParseProgram,
    initial_namespace: namespace::Module,
) -> CompileResult<ty::TyProgram> {
    // Type check the program.
    let CompileResult {
        value: typed_program_opt,
        mut warnings,
        mut errors,
    } = ty::TyProgram::type_check(parse_program, initial_namespace);
    let mut typed_program = match typed_program_opt {
        Some(typed_program) => typed_program,
        None => return err(warnings, errors),
    };

    // Collect information about the types used in this program
    let CompileResult {
        value: types_metadata_result,
        warnings: new_warnings,
        errors: new_errors,
    } = typed_program.collect_types_metadata(&mut CollectTypesMetadataContext::new());
    warnings.extend(new_warnings);
    errors.extend(new_errors);
    let types_metadata = match types_metadata_result {
        Some(types_metadata) => types_metadata,
        None => return deduped_err(warnings, errors),
    };

    typed_program
        .logged_types
        .extend(types_metadata.iter().filter_map(|m| match m {
            TypeMetadata::LoggedType(log_id, type_id) => Some((*log_id, *type_id)),
            _ => None,
        }));

    // Perform control flow analysis and extend with any errors.
    let cfa_res = perform_control_flow_analysis(&typed_program);
    errors.extend(cfa_res.errors);
    warnings.extend(cfa_res.warnings);

    // Evaluate const declarations,
    // to allow storage slots initializion with consts.
    let mut ctx = Context::default();
    let mut md_mgr = MetadataManager::default();
    let module = Module::new(&mut ctx, Kind::Contract);
    if let Err(e) = ir_generation::compile::compile_constants(
        &mut ctx,
        &mut md_mgr,
        module,
        &typed_program.root.namespace,
    ) {
        errors.push(e);
    }

    // Check that all storage initializers can be evaluated at compile time.
    let typed_wiss_res = typed_program.get_typed_program_with_initialized_storage_slots(
        &mut ctx,
        &mut md_mgr,
        module,
    );
    warnings.extend(typed_wiss_res.warnings);
    errors.extend(typed_wiss_res.errors);
    let typed_program_with_storage_slots = match typed_wiss_res.value {
        Some(typed_program_with_storage_slots) => typed_program_with_storage_slots,
        None => return deduped_err(warnings, errors),
    };

    // All unresolved types lead to compile errors.
    errors.extend(types_metadata.iter().filter_map(|m| match m {
        TypeMetadata::UnresolvedType(name) => Some(CompileError::UnableToInferGeneric {
            ty: name.as_str().to_string(),
            span: name.span(),
        }),
        _ => None,
    }));

    // CEI pattern analysis
    let cei_analysis_warnings =
        semantic_analysis::cei_pattern_analysis::analyze_program(&typed_program);
    warnings.extend(cei_analysis_warnings);

    ok(
        typed_program_with_storage_slots,
        dedup_unsorted(warnings),
        dedup_unsorted(errors),
    )
}

pub fn compile_to_ast(
    input: Arc<str>,
    initial_namespace: namespace::Module,
    build_config: Option<&BuildConfig>,
) -> CompileResult<ty::TyProgram> {
    // Parse the program to a concrete syntax tree (CST).
    let CompileResult {
        value: parse_program_opt,
        mut warnings,
        mut errors,
    } = parse(input, build_config);
    let parse_program = match parse_program_opt {
        Some(parse_program) => parse_program,
        None => return deduped_err(warnings, errors),
    };

    // Type check (+ other static analysis) the CST to a typed AST.
    let typed_res = parsed_to_ast(&parse_program, initial_namespace);
    errors.extend(typed_res.errors);
    warnings.extend(typed_res.warnings);
    let typed_program = match typed_res.value {
        Some(tp) => tp,
        None => return deduped_err(warnings, errors),
    };

    ok(
        typed_program,
        dedup_unsorted(warnings),
        dedup_unsorted(errors),
    )
}

/// Given input Sway source code,
/// try compiling to a `CompiledAsm`,
/// containing the asm in opcode form (not raw bytes/bytecode).
pub fn compile_to_asm(
    input: Arc<str>,
    initial_namespace: namespace::Module,
    build_config: BuildConfig,
) -> CompileResult<CompiledAsm> {
    let ast_res = compile_to_ast(input, initial_namespace, Some(&build_config));
    ast_to_asm(ast_res, &build_config)
}

/// Given an AST compilation result,
/// try compiling to a `CompiledAsm`,
/// containing the asm in opcode form (not raw bytes/bytecode).
pub fn ast_to_asm(
    ast_res: CompileResult<ty::TyProgram>,
    build_config: &BuildConfig,
) -> CompileResult<CompiledAsm> {
    match ast_res.value {
        None => err(ast_res.warnings, ast_res.errors),
        Some(typed_program) => {
            let mut errors = ast_res.errors;
            let mut warnings = ast_res.warnings;
            let asm = check!(
                compile_ast_to_ir_to_asm(typed_program, build_config),
                return deduped_err(warnings, errors),
                warnings,
                errors
            );
            ok(CompiledAsm(asm), warnings, errors)
        }
    }
}

pub(crate) fn compile_ast_to_ir_to_asm(
    program: ty::TyProgram,
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

    let tree_type = program.kind.tree_type();
    let mut ir = match ir_generation::compile_program(program) {
        Ok(ir) => ir,
        Err(e) => return err(warnings, vec![e]),
    };

    // Find all the entry points.  This is main for scripts and predicates, or ABI methods for
    // contracts, identified by them having a selector.
    let entry_point_functions: Vec<::sway_ir::Function> = ir
        .module_iter()
        .flat_map(|module| module.function_iter(&ir))
        .filter(|func| {
            let is_script_or_predicate = matches!(
                tree_type,
                parsed::TreeType::Script | parsed::TreeType::Predicate
            );
            let is_contract = tree_type == parsed::TreeType::Contract;
            let has_entry_name =
                func.get_name(&ir) == sway_types::constants::DEFAULT_ENTRY_POINT_FN_NAME;

            (is_script_or_predicate && has_entry_name) || (is_contract && func.has_selector(&ir))
        })
        .collect();

    // Do a purity check on the _unoptimised_ IR.
    {
        let handler = Handler::default();
        let mut env = ir_generation::PurityEnv::default();
        let mut md_mgr = metadata::MetadataManager::default();
        for entry_point in &entry_point_functions {
            check_function_purity(&handler, &mut env, &ir, &mut md_mgr, entry_point);
        }
        let (e, w) = handler.consume();
        warnings.extend(w);
        errors.extend(e);
    }

    // Now we're working with all functions in the module.
    let all_functions = ir
        .module_iter()
        .flat_map(|module| module.function_iter(&ir))
        .collect::<Vec<_>>();

    // Promote local values to registers.
    check!(
        promote_to_registers(&mut ir, &all_functions),
        return err(warnings, errors),
        warnings,
        errors
    );

    // Inline function calls.
    check!(
        inline_function_calls(&mut ir, &all_functions, &tree_type),
        return err(warnings, errors),
        warnings,
        errors
    );

    let res = CompileResult::with_handler(|handler| {
        // TODO: Experiment with putting combine-constants and simplify-cfg
        // in a loop, but per function.
        combine_constants(handler, &mut ir, &all_functions)?;
        simplify_cfg(handler, &mut ir, &all_functions)?;
        // Simplify-CFG helps combine constants.
        combine_constants(handler, &mut ir, &all_functions)?;
        // And that in-turn enables more simplify-cfg.
        simplify_cfg(handler, &mut ir, &all_functions)?;

        // Remove dead definitions based on the entry points root set.
        dce(handler, &mut ir, &entry_point_functions)?;
        Ok(())
    });
    check!(res, return err(warnings, errors), warnings, errors);

    if build_config.print_ir {
        tracing::info!("{}", ir);
    }

    let final_asm = check!(
        compile_ir_to_asm(&ir, Some(build_config)),
        return err(warnings, errors),
        warnings,
        errors
    );

    ok(final_asm, warnings, errors)
}

fn promote_to_registers(ir: &mut Context, functions: &[Function]) -> CompileResult<()> {
    for function in functions {
        if let Err(ir_error) = sway_ir::optimize::promote_to_registers(ir, function) {
            return err(
                Vec::new(),
                vec![CompileError::InternalOwned(
                    ir_error.to_string(),
                    span::Span::dummy(),
                )],
            );
        }
    }
    ok((), Vec::new(), Vec::new())
}

// Inline function calls based on two conditions:
// 1. The program we're compiling is a "predicate". Predicates cannot jump backwards which means
//    that supporting function calls (i.e. without inlining) is not possible. This is a protocl
//    restriction and not a heuristic.
// 2. If the program is not a "predicate" then, we rely on some heuristic which is described below
//    in the `inline_heuristc` closure.
//
fn inline_function_calls(
    ir: &mut Context,
    functions: &[Function],
    tree_type: &parsed::TreeType,
) -> CompileResult<()> {
    // Inspect ALL calls and count how often each function is called.
    // This is not required for predicates because we don't inline their function calls
    let call_counts: HashMap<Function, u64> = match tree_type {
        parsed::TreeType::Predicate => HashMap::new(),
        _ => functions.iter().fold(HashMap::new(), |mut counts, func| {
            for (_block, ins) in func.instruction_iter(ir) {
                if let Some(Instruction::Call(callee, _args)) = ins.get_instruction(ir) {
                    counts
                        .entry(*callee)
                        .and_modify(|count| *count += 1)
                        .or_insert(1);
                }
            }
            counts
        }),
    };

    let inline_heuristic = |ctx: &Context, func: &Function, _call_site: &Value| {
        // For now, pending improvements to ASMgen for calls, we must inline any function which has
        // too many args.
        if func.args_iter(ctx).count() as u8
            > crate::asm_generation::compiler_constants::NUM_ARG_REGISTERS
        {
            return true;
        }

        // If the function is called only once then definitely inline it.
        if call_counts.get(func).copied().unwrap_or(0) == 1 {
            return true;
        }

        // If the function is (still) small then also inline it.
        const MAX_INLINE_INSTRS_COUNT: usize = 4;
        if func.num_instructions(ctx) <= MAX_INLINE_INSTRS_COUNT {
            return true;
        }

        // As per https://github.com/FuelLabs/sway/issues/2819 we can hit problems if a function
        // argument is used as a pointer (probably because it has a ref type) although it actually
        // isn't one.  Ref type args which aren't pointers need to be inlined.
        if func.args_iter(ctx).any(|(_name, arg_val)| {
            arg_val
                .get_type(ctx)
                .map(|ty| !ty.is_copy_type())
                .unwrap_or(false)
        }) {
            return true;
        }

        false
    };

    for function in functions {
        if let Err(ir_error) = match tree_type {
            parsed::TreeType::Predicate => {
                // Inline everything for predicates
                sway_ir::optimize::inline_all_function_calls(ir, function)
            }
            _ => sway_ir::optimize::inline_some_function_calls(ir, function, inline_heuristic),
        } {
            return err(
                Vec::new(),
                vec![CompileError::InternalOwned(
                    ir_error.to_string(),
                    span::Span::dummy(),
                )],
            );
        }
    }
    ok((), Vec::new(), Vec::new())
}

fn combine_constants(
    handler: &Handler,
    ir: &mut Context,
    functions: &[Function],
) -> Result<(), ErrorEmitted> {
    for function in functions {
        if let Err(ir_error) = sway_ir::optimize::combine_constants(ir, function) {
            return Err(handler.emit_err(CompileError::InternalOwned(
                ir_error.to_string(),
                span::Span::dummy(),
            )));
        }
    }
    Ok(())
}

fn dce(
    handler: &Handler,
    ir: &mut Context,
    entry_functions: &[Function],
) -> Result<(), ErrorEmitted> {
    // Remove entire dead functions first.
    for module in ir.module_iter() {
        sway_ir::optimize::func_dce(ir, &module, entry_functions);
    }

    // Then DCE all the remaining functions.
    for module in ir.module_iter() {
        for function in module.function_iter(ir) {
            if let Err(ir_error) = sway_ir::optimize::dce(ir, &function) {
                return Err(handler.emit_err(CompileError::InternalOwned(
                    ir_error.to_string(),
                    span::Span::dummy(),
                )));
            }
        }
    }
    Ok(())
}

fn simplify_cfg(
    handler: &Handler,
    ir: &mut Context,
    functions: &[Function],
) -> Result<(), ErrorEmitted> {
    for function in functions {
        if let Err(ir_error) = sway_ir::optimize::simplify_cfg(ir, function) {
            return Err(handler.emit_err(CompileError::InternalOwned(
                ir_error.to_string(),
                span::Span::dummy(),
            )));
        }
    }
    Ok(())
}

/// Given input Sway source code, compile to [CompiledBytecode], containing the asm in bytecode form.
pub fn compile_to_bytecode(
    input: Arc<str>,
    initial_namespace: namespace::Module,
    build_config: BuildConfig,
    source_map: &mut SourceMap,
) -> CompileResult<CompiledBytecode> {
    let asm_res = compile_to_asm(input, initial_namespace, build_config);
    let result = asm_to_bytecode(asm_res, source_map);
    clear_lazy_statics();
    result
}

/// Given the assembly (opcodes), compile to [CompiledBytecode], containing the asm in bytecode form.
pub fn asm_to_bytecode(
    CompileResult {
        value,
        mut warnings,
        mut errors,
    }: CompileResult<CompiledAsm>,
    source_map: &mut SourceMap,
) -> CompileResult<CompiledBytecode> {
    match value {
        Some(CompiledAsm(mut asm)) => {
            let bytes = check!(
                asm.to_bytecode_mut(source_map),
                return err(warnings, errors),
                warnings,
                errors,
            );
            ok(CompiledBytecode(bytes), warnings, errors)
        }
        None => err(warnings, errors),
    }
}

pub fn clear_lazy_statics() {
    type_system::clear_type_engine();
    declaration_engine::declaration_engine::de_clear();
}

/// Given a [ty::TyProgram], which is type-checked Sway source, construct a graph to analyze
/// control flow and determine if it is valid.
fn perform_control_flow_analysis(program: &ty::TyProgram) -> CompileResult<()> {
    let dca_res = dead_code_analysis(program);
    let rpa_errors = return_path_analysis(program);
    let rpa_res = if rpa_errors.is_empty() {
        ok((), vec![], vec![])
    } else {
        err(vec![], rpa_errors)
    };
    dca_res.flat_map(|_| rpa_res)
}

/// Constructs a dead code graph from all modules within the graph and then attempts to find dead
/// code.
///
/// Returns the graph that was used for analysis.
fn dead_code_analysis(program: &ty::TyProgram) -> CompileResult<ControlFlowGraph> {
    let mut dead_code_graph = Default::default();
    let tree_type = program.kind.tree_type();
    module_dead_code_analysis(&program.root, &tree_type, &mut dead_code_graph).flat_map(|_| {
        let warnings = dead_code_graph.find_dead_code();
        ok(dead_code_graph, warnings, vec![])
    })
}

/// Recursively collect modules into the given `ControlFlowGraph` ready for dead code analysis.
fn module_dead_code_analysis(
    module: &ty::TyModule,
    tree_type: &parsed::TreeType,
    graph: &mut ControlFlowGraph,
) -> CompileResult<()> {
    let init_res = ok((), vec![], vec![]);
    let submodules_res = module
        .submodules
        .iter()
        .fold(init_res, |res, (_, submodule)| {
            let name = submodule.library_name.clone();
            let tree_type = parsed::TreeType::Library { name };
            res.flat_map(|_| module_dead_code_analysis(&submodule.module, &tree_type, graph))
        });
    submodules_res.flat_map(|()| {
        ControlFlowGraph::append_module_to_dead_code_graph(&module.all_nodes, tree_type, graph)
            .map(|_| ok((), vec![], vec![]))
            .unwrap_or_else(|error| err(vec![], vec![error]))
    })
}

fn return_path_analysis(program: &ty::TyProgram) -> Vec<CompileError> {
    let mut errors = vec![];
    module_return_path_analysis(&program.root, &mut errors);
    errors
}

fn module_return_path_analysis(module: &ty::TyModule, errors: &mut Vec<CompileError>) {
    for (_, submodule) in &module.submodules {
        module_return_path_analysis(&submodule.module, errors);
    }
    let graph = ControlFlowGraph::construct_return_path_graph(&module.all_nodes);
    match graph {
        Ok(graph) => errors.extend(graph.analyze_return_paths()),
        Err(error) => errors.push(error),
    }
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
    use crate::language::{self, parsed};

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
    if let parsed::AstNode {
        content:
            parsed::AstNodeContent::Declaration(parsed::Declaration::FunctionDeclaration(
                parsed::FunctionDeclaration { body, .. },
            )),
        ..
    } = &prog.root.tree.root_nodes[0]
    {
        if let parsed::AstNode {
            content:
                parsed::AstNodeContent::Expression(parsed::Expression {
                    kind:
                        parsed::ExpressionKind::LazyOperator(parsed::LazyOperatorExpression {
                            op, ..
                        }),
                    ..
                }),
            ..
        } = &body.contents[2]
        {
            assert_eq!(op, &language::LazyOp::And)
        } else {
            panic!("Was not lazy operator.")
        }
    } else {
        panic!("Was not ast node")
    };
}

/// Return an irrecoverable compile result deduping any errors and warnings.
fn deduped_err<T>(warnings: Vec<CompileWarning>, errors: Vec<CompileError>) -> CompileResult<T> {
    err(dedup_unsorted(warnings), dedup_unsorted(errors))
}

/// We want compile errors and warnings to retain their ordering, since typically
/// they are grouped by relevance. However, we want to deduplicate them.
/// Stdlib dedup in Rust assumes sorted data for efficiency, but we don't want that.
/// A hash set would also mess up the order, so this is just a brute force way of doing it
/// with a vector.
fn dedup_unsorted<T: PartialEq + std::hash::Hash>(mut data: Vec<T>) -> Vec<T> {
    // TODO(Centril): Consider using `IndexSet` instead for readability.
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
