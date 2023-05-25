#[macro_use]
pub mod error;

#[macro_use]
mod engine_threading;

pub mod abi_generation;
pub mod asm_generation;
mod asm_lang;
mod build_config;
mod concurrent_slab;
mod control_flow_analysis;
pub mod decl_engine;
pub mod ir_generation;
pub mod language;
mod metadata;
mod monomorphize;
pub mod query_engine;
pub mod semantic_analysis;
pub mod source_map;
pub mod transform;
pub mod type_system;

use crate::ir_generation::check_function_purity;
use crate::{error::*, source_map::SourceMap};
pub use asm_generation::from_ir::compile_ir_to_asm;
use asm_generation::FinalizedAsm;
pub use asm_generation::{CompiledBytecode, FinalizedEntry};
pub use build_config::{BuildConfig, BuildTarget};
use control_flow_analysis::ControlFlowGraph;
use metadata::MetadataManager;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use sway_ast::AttributeDecl;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_ir::{
    create_o1_pass_group, register_known_passes, Context, Kind, Module, PassManager,
    ARGDEMOTION_NAME, CONSTDEMOTION_NAME, DCE_NAME, MEMCPYOPT_NAME, MISCDEMOTION_NAME,
    MODULEPRINTER_NAME, RETDEMOTION_NAME,
};
use sway_types::constants::DOC_COMMENT_ATTRIBUTE_NAME;
use sway_utils::{time_expr, PerformanceData, PerformanceMetric};
use transform::{Attribute, AttributeArg, AttributeKind, AttributesMap};
use types::*;

pub use semantic_analysis::namespace::{self, Namespace};
pub mod types;

pub use error::CompileResult;
use sway_error::error::CompileError;
use sway_error::warning::CompileWarning;
use sway_types::{ident::Ident, span, Spanned};
pub use type_system::*;

use language::{lexed, parsed, ty, Visibility};
use transform::to_parsed_lang::{self, convert_module_kind};

pub mod fuel_prelude {
    pub use fuel_vm::{self, fuel_asm, fuel_crypto, fuel_tx, fuel_types};
}

pub use engine_threading::Engines;
use sysinfo::{System, SystemExt};

/// Given an input `Arc<str>` and an optional [BuildConfig], parse the input into a [lexed::LexedProgram] and [parsed::ParseProgram].
///
/// # Example
/// ```ignore
/// # use sway_core::parse;
/// # fn main() {
///     let input = "script; fn main() -> bool { true }";
///     let result = parse(input.into(), <_>::default(), None);
/// # }
/// ```
///
/// # Panics
/// Panics if the parser panics.
pub fn parse(
    input: Arc<str>,
    engines: Engines<'_>,
    config: Option<&BuildConfig>,
) -> CompileResult<(lexed::LexedProgram, parsed::ParseProgram)> {
    CompileResult::with_handler(|h| match config {
        None => parse_in_memory(h, engines, input),
        // When a `BuildConfig` is given,
        // the module source may declare `dep`s that must be parsed from other files.
        Some(config) => parse_module_tree(
            h,
            engines,
            input,
            config.canonical_root_module(),
            None,
            config.build_target,
        )
        .map(|(kind, lexed, parsed)| {
            let lexed = lexed::LexedProgram {
                kind: kind.clone(),
                root: lexed,
            };
            let parsed = parsed::ParseProgram { kind, root: parsed };
            (lexed, parsed)
        }),
    })
}

/// Parses the tree kind in the input provided.
///
/// This will lex the entire input, but parses only the module kind.
pub fn parse_tree_type(input: Arc<str>) -> CompileResult<parsed::TreeType> {
    CompileResult::with_handler(|h| {
        sway_parse::parse_module_kind(h, input, None).map(|kind| convert_module_kind(&kind))
    })
}

/// Convert attributes from `Annotated<Module>` to an [AttributesMap].
fn module_attrs_to_map(
    handler: &Handler,
    attribute_list: &[AttributeDecl],
) -> Result<AttributesMap, ErrorEmitted> {
    let mut attrs_map: HashMap<_, Vec<Attribute>> = HashMap::new();
    for attr_decl in attribute_list {
        let attrs = attr_decl.attribute.get().into_iter();
        for attr in attrs {
            let name = attr.name.as_str();
            if name != DOC_COMMENT_ATTRIBUTE_NAME {
                // prevent using anything except doc comment attributes
                handler.emit_err(CompileError::ExpectedModuleDocComment {
                    span: attr.name.span(),
                });
            }

            let args = attr
                .args
                .as_ref()
                .map(|parens| {
                    parens
                        .get()
                        .into_iter()
                        .cloned()
                        .map(|arg| AttributeArg {
                            name: arg.name.clone(),
                            value: arg.value.clone(),
                            span: arg.span(),
                        })
                        .collect()
                })
                .unwrap_or_else(Vec::new);

            let attribute = Attribute {
                name: attr.name.clone(),
                args,
                span: attr_decl.span(),
            };

            if let Some(attr_kind) = match name {
                DOC_COMMENT_ATTRIBUTE_NAME => Some(AttributeKind::DocComment),
                _ => None,
            } {
                attrs_map.entry(attr_kind).or_default().push(attribute);
            }
        }
    }
    Ok(AttributesMap::new(Arc::new(attrs_map)))
}

/// When no `BuildConfig` is given, we're assumed to be parsing in-memory with no submodules.
fn parse_in_memory(
    handler: &Handler,
    engines: Engines<'_>,
    src: Arc<str>,
) -> Result<(lexed::LexedProgram, parsed::ParseProgram), ErrorEmitted> {
    let module = sway_parse::parse_file(handler, src, None)?;
    let (kind, tree) = to_parsed_lang::convert_parse_tree(
        &mut to_parsed_lang::Context::default(),
        handler,
        engines,
        module.value.clone(),
    )?;
    let submodules = Default::default();
    let attributes = module_attrs_to_map(handler, &module.attribute_list)?;
    let root = parsed::ParseModule {
        span: span::Span::dummy(),
        tree,
        submodules,
        attributes,
    };
    let lexed_program = lexed::LexedProgram::new(
        kind.clone(),
        lexed::LexedModule {
            tree: module.value,
            submodules: Default::default(),
        },
    );
    Ok((lexed_program, parsed::ParseProgram { kind, root }))
}

/// Contains the lexed and parsed submodules 'deps' of a module.
struct Submodules {
    lexed: Vec<(Ident, lexed::LexedSubmodule)>,
    parsed: Vec<(Ident, parsed::ParseSubmodule)>,
}

/// Parse all dependencies `deps` as submodules.
fn parse_submodules(
    handler: &Handler,
    engines: Engines<'_>,
    module_name: Option<&str>,
    module: &sway_ast::Module,
    module_dir: &Path,
    build_target: BuildTarget,
) -> Submodules {
    // Assume the happy path, so there'll be as many submodules as dependencies, but no more.
    let mut lexed_submods = Vec::with_capacity(module.submodules().count());
    let mut parsed_submods = Vec::with_capacity(lexed_submods.capacity());

    module.submodules().for_each(|submod| {
        // Read the source code from the dependency.
        // If we cannot, record as an error, but continue with other files.
        let submod_path = Arc::new(module_path(module_dir, module_name, submod));
        let submod_str: Arc<str> = match std::fs::read_to_string(&*submod_path) {
            Ok(s) => Arc::from(s),
            Err(e) => {
                handler.emit_err(CompileError::FileCouldNotBeRead {
                    span: submod.name.span(),
                    file_path: submod_path.to_string_lossy().to_string(),
                    stringified_error: e.to_string(),
                });
                return;
            }
        };

        if let Ok((kind, lexed_module, parse_module)) = parse_module_tree(
            handler,
            engines,
            submod_str.clone(),
            submod_path.clone(),
            Some(submod.name.as_str()),
            build_target,
        ) {
            if !matches!(kind, parsed::TreeType::Library) {
                let span = span::Span::new(submod_str, 0, 0, Some(submod_path)).unwrap();
                handler.emit_err(CompileError::ImportMustBeLibrary { span });
                return;
            }

            let parse_submodule = parsed::ParseSubmodule {
                module: parse_module,
                visibility: match submod.visibility {
                    Some(..) => Visibility::Public,
                    None => Visibility::Private,
                },
                mod_name_span: submod.name.span(),
            };
            let lexed_submodule = lexed::LexedSubmodule {
                module: lexed_module,
            };
            lexed_submods.push((submod.name.clone(), lexed_submodule));
            parsed_submods.push((submod.name.clone(), parse_submodule));
        }
    });

    Submodules {
        lexed: lexed_submods,
        parsed: parsed_submods,
    }
}

/// Given the source of the module along with its path,
/// parse this module including all of its submodules.
fn parse_module_tree(
    handler: &Handler,
    engines: Engines<'_>,
    src: Arc<str>,
    path: Arc<PathBuf>,
    module_name: Option<&str>,
    build_target: BuildTarget,
) -> Result<(parsed::TreeType, lexed::LexedModule, parsed::ParseModule), ErrorEmitted> {
    // Parse this module first.
    let module_dir = path.parent().expect("module file has no parent directory");
    let module = sway_parse::parse_file(handler, src.clone(), Some(path.clone()))?;

    // Parse all submodules before converting to the `ParseTree`.
    // This always recovers on parse errors for the file itself by skipping that file.
    let submodules = parse_submodules(
        handler,
        engines,
        module_name,
        &module.value,
        module_dir,
        build_target,
    );

    // Convert from the raw parsed module to the `ParseTree` ready for type-check.
    let (kind, tree) = to_parsed_lang::convert_parse_tree(
        &mut to_parsed_lang::Context::new(build_target),
        handler,
        engines,
        module.value.clone(),
    )?;
    let attributes = module_attrs_to_map(handler, &module.attribute_list)?;

    let lexed = lexed::LexedModule {
        tree: module.value,
        submodules: submodules.lexed,
    };
    let parsed = parsed::ParseModule {
        span: span::Span::new(src, 0, 0, Some(path)).unwrap(),
        tree,
        submodules: submodules.parsed,
        attributes,
    };
    Ok((kind, lexed, parsed))
}

fn module_path(
    parent_module_dir: &Path,
    parent_module_name: Option<&str>,
    submod: &sway_ast::Submodule,
) -> PathBuf {
    if let Some(parent_name) = parent_module_name {
        parent_module_dir
            .join(parent_name)
            .join(submod.name.to_string())
            .with_extension(sway_types::constants::DEFAULT_FILE_EXTENSION)
    } else {
        // top level module
        parent_module_dir
            .join(submod.name.to_string())
            .with_extension(sway_types::constants::DEFAULT_FILE_EXTENSION)
    }
}

pub struct CompiledAsm(pub FinalizedAsm);

pub fn parsed_to_ast(
    engines: Engines<'_>,
    parse_program: &parsed::ParseProgram,
    initial_namespace: namespace::Module,
    build_config: Option<&BuildConfig>,
    package_name: &str,
) -> CompileResult<ty::TyProgram> {
    // Type check the program.
    let CompileResult {
        value: typed_program_opt,
        mut warnings,
        mut errors,
    } = ty::TyProgram::type_check(engines, parse_program, initial_namespace, package_name);

    let mut typed_program = match typed_program_opt {
        Some(typed_program) => typed_program,
        None => return err(warnings, errors),
    };

    // Collect information about the types used in this program
    let CompileResult {
        value: types_metadata_result,
        warnings: new_warnings,
        errors: new_errors,
    } = typed_program.collect_types_metadata(&mut CollectTypesMetadataContext::new(engines));
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

    typed_program
        .messages_types
        .extend(types_metadata.iter().filter_map(|m| match m {
            TypeMetadata::MessageType(message_id, type_id) => Some((*message_id, *type_id)),
            _ => None,
        }));

    let (print_graph, print_graph_url_format) = match build_config {
        Some(cfg) => (
            cfg.print_dca_graph.clone(),
            cfg.print_dca_graph_url_format.clone(),
        ),
        None => (None, None),
    };
    // Perform control flow analysis and extend with any errors.
    let cfa_res =
        perform_control_flow_analysis(engines, &typed_program, print_graph, print_graph_url_format);
    errors.extend(cfa_res.errors);
    warnings.extend(cfa_res.warnings);

    // Evaluate const declarations, to allow storage slots initializion with consts.
    let mut ctx = Context::default();
    let mut md_mgr = MetadataManager::default();
    let module = Module::new(&mut ctx, Kind::Contract);
    if let Err(e) = ir_generation::compile::compile_constants(
        engines,
        &mut ctx,
        &mut md_mgr,
        module,
        &typed_program.root.namespace,
    ) {
        errors.push(e);
    }

    // CEI pattern analysis
    let cei_analysis_warnings =
        semantic_analysis::cei_pattern_analysis::analyze_program(engines, &typed_program);
    warnings.extend(cei_analysis_warnings);

    // Check that all storage initializers can be evaluated at compile time.
    let typed_wiss_res = typed_program.get_typed_program_with_initialized_storage_slots(
        engines,
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
        TypeMetadata::UnresolvedType(name, call_site_span_opt) => {
            Some(CompileError::UnableToInferGeneric {
                ty: name.as_str().to_string(),
                span: call_site_span_opt.clone().unwrap_or_else(|| name.span()),
            })
        }
        _ => None,
    }));

    // Check if a non-test function calls `#[test]` function.

    ok(
        typed_program_with_storage_slots,
        dedup_unsorted(warnings),
        dedup_unsorted(errors),
    )
}

pub fn compile_to_ast(
    engines: Engines<'_>,
    input: Arc<str>,
    initial_namespace: namespace::Module,
    build_config: Option<&BuildConfig>,
    package_name: &str,
    metrics: &mut PerformanceData,
) -> CompileResult<ty::TyProgram> {
    // Parse the program to a concrete syntax tree (CST).
    let CompileResult {
        value: parse_program_opt,
        mut warnings,
        mut errors,
    } = time_expr!(
        "parse the program to a concrete syntax tree (CST)",
        "parse_cst",
        parse(input, engines, build_config),
        build_config,
        metrics
    );

    let (.., mut parse_program) = match parse_program_opt {
        Some(parse_program) => parse_program,
        None => return deduped_err(warnings, errors),
    };

    // If tests are not enabled, exclude them from `parsed_program`.
    if build_config
        .map(|config| !config.include_tests)
        .unwrap_or(true)
    {
        parse_program.exclude_tests();
    }

    // Type check (+ other static analysis) the CST to a typed AST.
    let typed_res = time_expr!(
        "parse the concrete syntax tree (CST) to a typed AST",
        "parse_ast",
        parsed_to_ast(
            engines,
            &parse_program,
            initial_namespace,
            build_config,
            package_name,
        ),
        build_config,
        metrics
    );

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
    engines: Engines<'_>,
    input: Arc<str>,
    initial_namespace: namespace::Module,
    build_config: BuildConfig,
    package_name: &str,
    metrics: &mut PerformanceData,
) -> CompileResult<CompiledAsm> {
    let ast_res = compile_to_ast(
        engines,
        input,
        initial_namespace,
        Some(&build_config),
        package_name,
        metrics,
    );
    ast_to_asm(engines, &ast_res, &build_config)
}

/// Given an AST compilation result,
/// try compiling to a `CompiledAsm`,
/// containing the asm in opcode form (not raw bytes/bytecode).
pub fn ast_to_asm(
    engines: Engines<'_>,
    ast_res: &CompileResult<ty::TyProgram>,
    build_config: &BuildConfig,
) -> CompileResult<CompiledAsm> {
    match &ast_res.value {
        None => err(ast_res.warnings.clone(), ast_res.errors.clone()),
        Some(typed_program) => {
            let mut errors = ast_res.errors.clone();
            let mut warnings = ast_res.warnings.clone();
            let asm = check!(
                compile_ast_to_ir_to_asm(engines, typed_program, build_config),
                return deduped_err(warnings, errors),
                warnings,
                errors
            );
            ok(CompiledAsm(asm), warnings, errors)
        }
    }
}

pub(crate) fn compile_ast_to_ir_to_asm(
    engines: Engines<'_>,
    program: &ty::TyProgram,
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

    let mut ir = match ir_generation::compile_program(program, build_config.include_tests, engines)
    {
        Ok(ir) => ir,
        Err(e) => return err(warnings, vec![e]),
    };

    // Find all the entry points for purity checking and DCE.
    let entry_point_functions: Vec<::sway_ir::Function> = ir
        .module_iter()
        .flat_map(|module| module.function_iter(&ir))
        .filter(|func| func.is_entry(&ir))
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

    // Initialize the pass manager and register known passes.
    let mut pass_mgr = PassManager::default();
    register_known_passes(&mut pass_mgr);
    let mut pass_group = create_o1_pass_group();

    // Target specific transforms should be moved into something more configured.
    if build_config.build_target == BuildTarget::Fuel {
        // FuelVM target specific transforms.
        //
        // Demote large by-value constants, arguments and return values to by-reference values
        // using temporaries.
        pass_group.append_pass(CONSTDEMOTION_NAME);
        pass_group.append_pass(ARGDEMOTION_NAME);
        pass_group.append_pass(RETDEMOTION_NAME);
        pass_group.append_pass(MISCDEMOTION_NAME);

        // Convert loads and stores to mem_copys where possible.
        pass_group.append_pass(MEMCPYOPT_NAME);

        // Run a DCE and simplify-cfg to clean up any obsolete instructions.
        pass_group.append_pass(DCE_NAME);
        // XXX Oh no, if we add simplifycfg here it unearths a bug in the register allocator which
        // manifests in the `should_pass/language/while_loops` test.  Fixing the register allocator
        // is a very high priority but isn't a part of this change.
        //pass_group.append_pass(SIMPLIFYCFG_NAME);
    }

    if build_config.print_ir {
        pass_group.append_pass(MODULEPRINTER_NAME);
    }

    // Run the passes.
    let res = CompileResult::with_handler(|handler| {
        if let Err(ir_error) = pass_mgr.run(&mut ir, &pass_group) {
            Err(handler.emit_err(CompileError::InternalOwned(
                ir_error.to_string(),
                span::Span::dummy(),
            )))
        } else {
            Ok(())
        }
    });
    check!(res, return err(warnings, errors), warnings, errors);

    let final_asm = check!(
        compile_ir_to_asm(&ir, Some(build_config)),
        return err(warnings, errors),
        warnings,
        errors
    );

    ok(final_asm, warnings, errors)
}

/// Given input Sway source code, compile to [CompiledBytecode], containing the asm in bytecode form.
pub fn compile_to_bytecode(
    engines: Engines<'_>,
    input: Arc<str>,
    initial_namespace: namespace::Module,
    build_config: BuildConfig,
    source_map: &mut SourceMap,
    package_name: &str,
    metrics: &mut PerformanceData,
) -> CompileResult<CompiledBytecode> {
    let asm_res = compile_to_asm(
        engines,
        input,
        initial_namespace,
        build_config,
        package_name,
        metrics,
    );
    asm_to_bytecode(asm_res, source_map)
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
            let compiled_bytecode = check!(
                asm.to_bytecode_mut(source_map),
                return err(warnings, errors),
                warnings,
                errors,
            );
            ok(compiled_bytecode, warnings, errors)
        }
        None => err(warnings, errors),
    }
}

/// Given a [ty::TyProgram], which is type-checked Sway source, construct a graph to analyze
/// control flow and determine if it is valid.
fn perform_control_flow_analysis(
    engines: Engines<'_>,
    program: &ty::TyProgram,
    print_graph: Option<String>,
    print_graph_url_format: Option<String>,
) -> CompileResult<()> {
    let dca_res = dead_code_analysis(engines, program);
    let rpa_errors = return_path_analysis(engines, program);
    let rpa_res = if rpa_errors.is_empty() {
        ok((), vec![], vec![])
    } else {
        err(vec![], rpa_errors)
    };
    if let Some(graph) = dca_res.clone().value {
        graph.visualize(engines, print_graph, print_graph_url_format);
    }
    dca_res.flat_map(|_| rpa_res)
}

/// Constructs a dead code graph from all modules within the graph and then attempts to find dead
/// code.
///
/// Returns the graph that was used for analysis.
fn dead_code_analysis<'a>(
    engines: Engines<'a>,
    program: &ty::TyProgram,
) -> CompileResult<ControlFlowGraph<'a>> {
    let decl_engine = engines.de();
    let mut dead_code_graph = Default::default();
    let tree_type = program.kind.tree_type();
    module_dead_code_analysis(engines, &program.root, &tree_type, &mut dead_code_graph).flat_map(
        |_| {
            let warnings = dead_code_graph.find_dead_code(decl_engine);
            ok(dead_code_graph, warnings, vec![])
        },
    )
}

/// Recursively collect modules into the given `ControlFlowGraph` ready for dead code analysis.
fn module_dead_code_analysis<'eng: 'cfg, 'cfg>(
    engines: Engines<'eng>,
    module: &ty::TyModule,
    tree_type: &parsed::TreeType,
    graph: &mut ControlFlowGraph<'cfg>,
) -> CompileResult<()> {
    let init_res = ok((), vec![], vec![]);
    let submodules_res = module
        .submodules
        .iter()
        .fold(init_res, |res, (_, submodule)| {
            let tree_type = parsed::TreeType::Library;
            res.flat_map(|_| {
                module_dead_code_analysis(engines, &submodule.module, &tree_type, graph)
            })
        });
    let res = submodules_res.flat_map(|()| {
        ControlFlowGraph::append_module_to_dead_code_graph(
            engines,
            &module.all_nodes,
            tree_type,
            graph,
        )
        .map(|_| ok((), vec![], vec![]))
        .unwrap_or_else(|error| err(vec![], vec![error]))
    });
    graph.connect_pending_entry_edges();
    res
}

fn return_path_analysis(engines: Engines<'_>, program: &ty::TyProgram) -> Vec<CompileError> {
    let mut errors = vec![];
    module_return_path_analysis(engines, &program.root, &mut errors);
    errors
}

fn module_return_path_analysis(
    engines: Engines<'_>,
    module: &ty::TyModule,
    errors: &mut Vec<CompileError>,
) {
    for (_, submodule) in &module.submodules {
        module_return_path_analysis(engines, &submodule.module, errors);
    }
    let graph = ControlFlowGraph::construct_return_path_graph(engines, &module.all_nodes);
    match graph {
        Ok(graph) => errors.extend(graph.analyze_return_paths(engines)),
        Err(error) => errors.push(error),
    }
}

#[test]
fn test_basic_prog() {
    use crate::{decl_engine::DeclEngine, query_engine::QueryEngine, TypeEngine};
    let type_engine = TypeEngine::default();
    let decl_engine = DeclEngine::default();
    let query_engine = QueryEngine::default();
    let engines = Engines::new(&type_engine, &decl_engine, &query_engine);
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
        engines,
        None,
    );
    let mut warnings: Vec<CompileWarning> = Vec::new();
    let mut errors: Vec<CompileError> = Vec::new();
    prog.unwrap(&mut warnings, &mut errors);
}
#[test]
fn test_parenthesized() {
    use crate::{decl_engine::DeclEngine, query_engine::QueryEngine, TypeEngine};
    let type_engine = TypeEngine::default();
    let decl_engine = DeclEngine::default();
    let query_engine = QueryEngine::default();
    let engines = Engines::new(&type_engine, &decl_engine, &query_engine);
    let prog = parse(
        r#"
        contract;
        pub fn some_abi_func() -> unit {
            let x = (5 + 6 / (1 + (2 / 1) + 4));
            return;
        }
    "#
        .into(),
        engines,
        None,
    );
    let mut warnings: Vec<CompileWarning> = Vec::new();
    let mut errors: Vec<CompileError> = Vec::new();
    prog.unwrap(&mut warnings, &mut errors);
}

#[test]
fn test_unary_ordering() {
    use crate::language::{self, parsed};
    use crate::{decl_engine::DeclEngine, query_engine::QueryEngine, TypeEngine};
    let type_engine = TypeEngine::default();
    let decl_engine = DeclEngine::default();
    let query_engine = QueryEngine::default();
    let engines = Engines::new(&type_engine, &decl_engine, &query_engine);
    let prog = parse(
        r#"
    script;
    fn main() -> bool {
        let a = true;
        let b = true;
        !a && b;
    }"#
        .into(),
        engines,
        None,
    );
    let mut warnings: Vec<CompileWarning> = Vec::new();
    let mut errors: Vec<CompileError> = Vec::new();
    let (.., prog) = prog.unwrap(&mut warnings, &mut errors);
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
