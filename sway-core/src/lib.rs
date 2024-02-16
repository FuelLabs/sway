#[macro_use]
pub mod error;

#[macro_use]
pub mod engine_threading;

pub mod abi_generation;
pub mod asm_generation;
mod asm_lang;
mod build_config;
pub mod compiler_generated;
mod concurrent_slab;
mod control_flow_analysis;
pub mod decl_engine;
pub mod ir_generation;
pub mod language;
mod metadata;
pub mod query_engine;
pub mod semantic_analysis;
pub mod source_map;
pub mod transform;
pub mod type_system;

use crate::ir_generation::check_function_purity;
use crate::query_engine::ModuleCacheEntry;
use crate::source_map::SourceMap;
pub use asm_generation::from_ir::compile_ir_to_asm;
use asm_generation::FinalizedAsm;
pub use asm_generation::{CompiledBytecode, FinalizedEntry};
pub use build_config::{BuildConfig, BuildTarget, LspConfig, OptLevel};
use control_flow_analysis::ControlFlowGraph;
use indexmap::IndexMap;
use metadata::MetadataManager;
use query_engine::{ModuleCacheKey, ModulePath, ProgramsCacheEntry};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use sway_ast::AttributeDecl;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_ir::{
    create_o1_pass_group, register_known_passes, Context, Kind, Module, PassGroup, PassManager,
    ARGDEMOTION_NAME, CONSTDEMOTION_NAME, DCE_NAME, INLINE_MODULE_NAME, MEM2REG_NAME,
    MEMCPYOPT_NAME, MISCDEMOTION_NAME, MODULEPRINTER_NAME, RETDEMOTION_NAME, SIMPLIFYCFG_NAME,
    SROA_NAME,
};
use sway_types::constants::DOC_COMMENT_ATTRIBUTE_NAME;
use sway_types::SourceEngine;
use sway_utils::{time_expr, PerformanceData, PerformanceMetric};
use transform::{Attribute, AttributeArg, AttributeKind, AttributesMap};
use types::*;

pub use semantic_analysis::namespace::{self, Namespace};
pub mod types;

use sway_error::error::CompileError;
use sway_types::{ident::Ident, span, Spanned};
pub use type_system::*;

pub use language::Programs;
use language::{lexed, parsed, ty, Visibility};
use transform::to_parsed_lang::{self, convert_module_kind};

pub mod fuel_prelude {
    pub use fuel_vm::{self, fuel_asm, fuel_crypto, fuel_tx, fuel_types};
}

pub use build_config::ExperimentalFlags;
pub use engine_threading::Engines;

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
    handler: &Handler,
    engines: &Engines,
    config: Option<&BuildConfig>,
) -> Result<(lexed::LexedProgram, parsed::ParseProgram), ErrorEmitted> {
    match config {
        None => parse_in_memory(handler, engines, input),
        // When a `BuildConfig` is given,
        // the module source may declare `dep`s that must be parsed from other files.
        Some(config) => parse_module_tree(
            handler,
            engines,
            input,
            config.canonical_root_module(),
            None,
            config.build_target,
            config.include_tests,
            config.experimental,
        )
        .map(
            |ParsedModuleTree {
                 tree_type: kind,
                 lexed_module,
                 parse_module,
             }| {
                let lexed = lexed::LexedProgram {
                    kind: kind.clone(),
                    root: lexed_module,
                };
                let parsed = parsed::ParseProgram {
                    kind,
                    root: parse_module,
                };
                (lexed, parsed)
            },
        ),
    }
}

/// Parses the tree kind in the input provided.
///
/// This will lex the entire input, but parses only the module kind.
pub fn parse_tree_type(
    handler: &Handler,
    input: Arc<str>,
) -> Result<parsed::TreeType, ErrorEmitted> {
    sway_parse::parse_module_kind(handler, input, None).map(|kind| convert_module_kind(&kind))
}

/// Convert attributes from `Annotated<Module>` to an [AttributesMap].
fn module_attrs_to_map(
    handler: &Handler,
    attribute_list: &[AttributeDecl],
) -> Result<AttributesMap, ErrorEmitted> {
    let mut attrs_map: IndexMap<_, Vec<Attribute>> = IndexMap::new();
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
    engines: &Engines,
    src: Arc<str>,
) -> Result<(lexed::LexedProgram, parsed::ParseProgram), ErrorEmitted> {
    let mut hasher = DefaultHasher::new();
    src.hash(&mut hasher);
    let hash = hasher.finish();
    let module = sway_parse::parse_file(handler, src, None)?;

    let (kind, tree) = to_parsed_lang::convert_parse_tree(
        &mut to_parsed_lang::Context::default(),
        handler,
        engines,
        module.value.clone(),
    )?;
    let module_kind_span = module.value.kind.span();
    let submodules = Default::default();
    let attributes = module_attrs_to_map(handler, &module.attribute_list)?;
    let root = parsed::ParseModule {
        span: span::Span::dummy(),
        module_kind_span,
        tree,
        submodules,
        attributes,
        hash,
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

pub struct Submodule {
    name: Ident,
    path: ModulePath,
    lexed: lexed::LexedSubmodule,
    parsed: parsed::ParseSubmodule,
}

/// Contains the lexed and parsed submodules 'deps' of a module.
pub type Submodules = Vec<Submodule>;

/// Parse all dependencies `deps` as submodules.
#[allow(clippy::too_many_arguments)]
fn parse_submodules(
    handler: &Handler,
    engines: &Engines,
    module_name: Option<&str>,
    module: &sway_ast::Module,
    module_dir: &Path,
    build_target: BuildTarget,
    include_tests: bool,
    experimental: ExperimentalFlags,
) -> Submodules {
    // Assume the happy path, so there'll be as many submodules as dependencies, but no more.
    let mut submods = Vec::with_capacity(module.submodules().count());

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

        if let Ok(ParsedModuleTree {
            tree_type: kind,
            lexed_module,
            parse_module,
        }) = parse_module_tree(
            handler,
            engines,
            submod_str.clone(),
            submod_path.clone(),
            Some(submod.name.as_str()),
            build_target,
            include_tests,
            experimental,
        ) {
            if !matches!(kind, parsed::TreeType::Library) {
                let source_id = engines.se().get_source_id(submod_path.as_ref());
                let span = span::Span::new(submod_str, 0, 0, Some(source_id)).unwrap();
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
            let submodule = Submodule {
                name: submod.name.clone(),
                path: submod_path,
                lexed: lexed_submodule,
                parsed: parse_submodule,
            };
            submods.push(submodule);
        }
    });

    submods
}

pub type SourceHash = u64;

#[derive(Clone, Debug)]
pub struct ParsedModuleTree {
    pub tree_type: parsed::TreeType,
    pub lexed_module: lexed::LexedModule,
    pub parse_module: parsed::ParseModule,
}

/// Given the source of the module along with its path,
/// parse this module including all of its submodules.
#[allow(clippy::too_many_arguments)]
fn parse_module_tree(
    handler: &Handler,
    engines: &Engines,
    src: Arc<str>,
    path: Arc<PathBuf>,
    module_name: Option<&str>,
    build_target: BuildTarget,
    include_tests: bool,
    experimental: ExperimentalFlags,
) -> Result<ParsedModuleTree, ErrorEmitted> {
    let query_engine = engines.qe();

    // Parse this module first.
    let module_dir = path.parent().expect("module file has no parent directory");
    let source_id = engines.se().get_source_id(&path.clone());
    let module = sway_parse::parse_file(handler, src.clone(), Some(source_id))?;

    // Parse all submodules before converting to the `ParseTree`.
    // This always recovers on parse errors for the file itself by skipping that file.
    let submodules = parse_submodules(
        handler,
        engines,
        module_name,
        &module.value,
        module_dir,
        build_target,
        include_tests,
        experimental,
    );

    // Convert from the raw parsed module to the `ParseTree` ready for type-check.
    let (kind, tree) = to_parsed_lang::convert_parse_tree(
        &mut to_parsed_lang::Context::new(build_target, experimental),
        handler,
        engines,
        module.value.clone(),
    )?;
    let module_kind_span = module.value.kind.span();
    let attributes = module_attrs_to_map(handler, &module.attribute_list)?;

    let lexed_submodules = submodules
        .iter()
        .map(|s| (s.name.clone(), s.lexed.clone()))
        .collect::<Vec<_>>();
    let lexed = lexed::LexedModule {
        tree: module.value,
        submodules: lexed_submodules,
    };

    let mut hasher = DefaultHasher::new();
    src.hash(&mut hasher);
    let hash = hasher.finish();

    let parsed_submodules = submodules
        .iter()
        .map(|s| (s.name.clone(), s.parsed.clone()))
        .collect::<Vec<_>>();
    let parsed = parsed::ParseModule {
        span: span::Span::new(src, 0, 0, Some(source_id)).unwrap(),
        module_kind_span,
        tree,
        submodules: parsed_submodules,
        attributes,
        hash,
    };

    // Let's prime the cache with the module dependency and hash data.
    let modified_time = std::fs::metadata(path.as_path())
        .ok()
        .and_then(|m| m.modified().ok());
    let dependencies = submodules.into_iter().map(|s| s.path).collect::<Vec<_>>();
    let parsed_module_tree = ParsedModuleTree {
        tree_type: kind,
        lexed_module: lexed,
        parse_module: parsed,
    };
    let cache_entry = ModuleCacheEntry {
        path,
        modified_time,
        hash,
        dependencies,
        include_tests,
    };
    query_engine.insert_parse_module_cache_entry(cache_entry);

    Ok(parsed_module_tree)
}

fn is_parse_module_cache_up_to_date(
    engines: &Engines,
    path: &Arc<PathBuf>,
    include_tests: bool,
) -> bool {
    let query_engine = engines.qe();
    let key = ModuleCacheKey::new(path.clone(), include_tests);
    let entry = query_engine.get_parse_module_cache_entry(&key);
    match entry {
        Some(entry) => {
            let modified_time = std::fs::metadata(path.as_path())
                .ok()
                .and_then(|m| m.modified().ok());

            // Let's check if we can re-use the dependency information
            // we got from the cache, which is only true if the file hasn't been
            // modified since or if its hash is the same.
            let cache_up_to_date = entry.modified_time == modified_time || {
                let src = std::fs::read_to_string(path.as_path()).unwrap();

                let mut hasher = DefaultHasher::new();
                src.hash(&mut hasher);
                let hash = hasher.finish();

                hash == entry.hash
            };

            // Look at the dependencies recursively to make sure they have not been
            // modified either.
            if cache_up_to_date {
                entry
                    .dependencies
                    .iter()
                    .all(|path| is_parse_module_cache_up_to_date(engines, path, include_tests))
            } else {
                false
            }
        }
        None => false,
    }
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
    handler: &Handler,
    engines: &Engines,
    parse_program: &parsed::ParseProgram,
    initial_namespace: namespace::Module,
    build_config: Option<&BuildConfig>,
    package_name: &str,
    retrigger_compilation: Option<Arc<AtomicBool>>,
) -> Result<ty::TyProgram, ErrorEmitted> {
    let experimental = build_config.map(|x| x.experimental).unwrap_or_default();
    let lsp_config = build_config.map(|x| x.lsp_mode.clone()).unwrap_or_default();

    // Type check the program.
    let typed_program_opt = ty::TyProgram::type_check(
        handler,
        engines,
        parse_program,
        initial_namespace,
        package_name,
        build_config,
    );

    check_should_abort(handler, retrigger_compilation.clone())?;

    // Only clear the parsed AST nodes if we are running a regular compilation pipeline.
    // LSP needs these to build its token map, and they are cleared by `clear_module` as
    // part of the LSP garbage collection functionality instead.
    if lsp_config.is_none() {
        engines.pe().clear();
    }

    let mut typed_program = match typed_program_opt {
        Ok(typed_program) => typed_program,
        Err(e) => return Err(e),
    };

    typed_program.check_deprecated(engines, handler);

    match typed_program.check_recursive(engines, handler) {
        Ok(()) => {}
        Err(e) => {
            handler.dedup();
            return Err(e);
        }
    };

    // Skip collecting metadata if we triggered an optimised build from LSP.
    let types_metadata = if !lsp_config
        .as_ref()
        .map(|lsp| lsp.optimized_build)
        .unwrap_or(false)
    {
        // Collect information about the types used in this program
        let types_metadata_result = typed_program.collect_types_metadata(
            handler,
            &mut CollectTypesMetadataContext::new(engines, experimental),
        );
        let types_metadata = match types_metadata_result {
            Ok(types_metadata) => types_metadata,
            Err(e) => {
                handler.dedup();
                return Err(e);
            }
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

        check_should_abort(handler, retrigger_compilation.clone())?;

        // Perform control flow analysis and extend with any errors.
        let _ = perform_control_flow_analysis(
            handler,
            engines,
            &typed_program,
            print_graph,
            print_graph_url_format,
        );

        types_metadata
    } else {
        vec![]
    };

    // Evaluate const declarations, to allow storage slots initialization with consts.
    let mut ctx = Context::new(
        engines.se(),
        sway_ir::ExperimentalFlags {
            new_encoding: experimental.new_encoding,
        },
    );
    let mut md_mgr = MetadataManager::default();
    let module = Module::new(&mut ctx, Kind::Contract);
    if let Err(e) = ir_generation::compile::compile_constants(
        engines,
        &mut ctx,
        &mut md_mgr,
        module,
        typed_program.root.namespace.module(),
    ) {
        handler.emit_err(e);
    }

    // CEI pattern analysis
    let cei_analysis_warnings =
        semantic_analysis::cei_pattern_analysis::analyze_program(engines, &typed_program);
    for warn in cei_analysis_warnings {
        handler.emit_warn(warn);
    }

    // Check that all storage initializers can be evaluated at compile time.
    let typed_wiss_res = typed_program.get_typed_program_with_initialized_storage_slots(
        handler,
        engines,
        &mut ctx,
        &mut md_mgr,
        module,
    );
    let typed_program_with_storage_slots = match typed_wiss_res {
        Ok(typed_program_with_storage_slots) => typed_program_with_storage_slots,
        Err(e) => {
            handler.dedup();
            return Err(e);
        }
    };

    // All unresolved types lead to compile errors.
    for err in types_metadata.iter().filter_map(|m| match m {
        TypeMetadata::UnresolvedType(name, call_site_span_opt) => {
            Some(CompileError::UnableToInferGeneric {
                ty: name.as_str().to_string(),
                span: call_site_span_opt.clone().unwrap_or_else(|| name.span()),
            })
        }
        _ => None,
    }) {
        handler.emit_err(err);
    }

    // Check if a non-test function calls `#[test]` function.

    handler.dedup();
    Ok(typed_program_with_storage_slots)
}

pub fn compile_to_ast(
    handler: &Handler,
    engines: &Engines,
    input: Arc<str>,
    initial_namespace: namespace::Module,
    build_config: Option<&BuildConfig>,
    package_name: &str,
    retrigger_compilation: Option<Arc<AtomicBool>>,
) -> Result<Programs, ErrorEmitted> {
    check_should_abort(handler, retrigger_compilation.clone())?;

    let query_engine = engines.qe();
    let mut metrics = PerformanceData::default();

    if let Some(config) = build_config {
        let path = config.canonical_root_module();
        let include_tests = config.include_tests;

        // Check if we can re-use the data in the cache.
        if is_parse_module_cache_up_to_date(engines, &path, include_tests) {
            let mut entry = query_engine.get_programs_cache_entry(&path).unwrap();
            entry.programs.metrics.reused_modules += 1;

            let (warnings, errors) = entry.handler_data;
            let new_handler = Handler::from_parts(warnings, errors);
            handler.append(new_handler);
            return Ok(entry.programs);
        };
    }

    // Parse the program to a concrete syntax tree (CST).
    let parse_program_opt = time_expr!(
        "parse the program to a concrete syntax tree (CST)",
        "parse_cst",
        parse(input, handler, engines, build_config),
        build_config,
        metrics
    );

    check_should_abort(handler, retrigger_compilation.clone())?;

    let (lexed_program, mut parsed_program) = match parse_program_opt {
        Ok(modules) => modules,
        Err(e) => {
            handler.dedup();
            return Err(e);
        }
    };

    // If tests are not enabled, exclude them from `parsed_program`.
    if build_config
        .map(|config| !config.include_tests)
        .unwrap_or(true)
    {
        parsed_program.exclude_tests(engines);
    }

    // Type check (+ other static analysis) the CST to a typed AST.
    let typed_res = time_expr!(
        "parse the concrete syntax tree (CST) to a typed AST",
        "parse_ast",
        parsed_to_ast(
            handler,
            engines,
            &parsed_program,
            initial_namespace,
            build_config,
            package_name,
            retrigger_compilation.clone(),
        ),
        build_config,
        metrics
    );

    check_should_abort(handler, retrigger_compilation.clone())?;

    handler.dedup();

    let programs = Programs::new(lexed_program, parsed_program, typed_res, metrics);

    if let Some(config) = build_config {
        let path = config.canonical_root_module();
        let cache_entry = ProgramsCacheEntry {
            path,
            programs: programs.clone(),
            handler_data: handler.clone().consume(),
        };
        query_engine.insert_programs_cache_entry(cache_entry);
    }

    check_should_abort(handler, retrigger_compilation.clone())?;

    Ok(programs)
}

/// Given input Sway source code, try compiling to a `CompiledAsm`,
/// containing the asm in opcode form (not raw bytes/bytecode).
pub fn compile_to_asm(
    handler: &Handler,
    engines: &Engines,
    input: Arc<str>,
    initial_namespace: namespace::Module,
    build_config: BuildConfig,
    package_name: &str,
) -> Result<CompiledAsm, ErrorEmitted> {
    let ast_res = compile_to_ast(
        handler,
        engines,
        input,
        initial_namespace,
        Some(&build_config),
        package_name,
        None,
    )?;
    ast_to_asm(handler, engines, &ast_res, &build_config)
}

/// Given an AST compilation result, try compiling to a `CompiledAsm`,
/// containing the asm in opcode form (not raw bytes/bytecode).
pub fn ast_to_asm(
    handler: &Handler,
    engines: &Engines,
    programs: &Programs,
    build_config: &BuildConfig,
) -> Result<CompiledAsm, ErrorEmitted> {
    let typed_program = match &programs.typed {
        Ok(typed_program) => typed_program,
        Err(err) => return Err(*err),
    };

    let asm = match compile_ast_to_ir_to_asm(handler, engines, typed_program, build_config) {
        Ok(res) => res,
        Err(err) => {
            handler.dedup();
            return Err(err);
        }
    };
    Ok(CompiledAsm(asm))
}

pub(crate) fn compile_ast_to_ir_to_asm(
    handler: &Handler,
    engines: &Engines,
    program: &ty::TyProgram,
    build_config: &BuildConfig,
) -> Result<FinalizedAsm, ErrorEmitted> {
    // The IR pipeline relies on type information being fully resolved.
    // If type information is found to still be generic or unresolved inside of
    // IR, this is considered an internal compiler error. To resolve this situation,
    // we need to explicitly ensure all types are resolved before going into IR.
    //
    // We _could_ introduce a new type here that uses TypeInfo instead of TypeId and throw away
    // the engine, since we don't need inference for IR. That'd be a _lot_ of copy-pasted code,
    // though, so instead, we are just going to do a pass and throw any unresolved generics as
    // errors and then hold as a runtime invariant that none of the types will be unresolved in the
    // IR phase.

    let mut ir = match ir_generation::compile_program(
        program,
        build_config.include_tests,
        engines,
        build_config.experimental,
    ) {
        Ok(ir) => ir,
        Err(errors) => {
            let mut last = None;
            for e in errors {
                last = Some(handler.emit_err(e))
            }
            return Err(last.unwrap());
        }
    };

    // Find all the entry points for purity checking and DCE.
    let entry_point_functions: Vec<::sway_ir::Function> = ir
        .module_iter()
        .flat_map(|module| module.function_iter(&ir))
        .filter(|func| func.is_entry(&ir))
        .collect();

    // Do a purity check on the _unoptimised_ IR.
    {
        let mut env = ir_generation::PurityEnv::default();
        let mut md_mgr = metadata::MetadataManager::default();
        for entry_point in &entry_point_functions {
            check_function_purity(handler, &mut env, &ir, &mut md_mgr, entry_point);
        }
    }

    // Initialize the pass manager and register known passes.
    let mut pass_mgr = PassManager::default();
    register_known_passes(&mut pass_mgr);
    let mut pass_group = PassGroup::default();

    match build_config.optimization_level {
        OptLevel::Opt1 => {
            pass_group.append_group(create_o1_pass_group());
        }
        OptLevel::Opt0 => {
            // Inlining is necessary until #4899 is resolved.
            pass_group.append_pass(INLINE_MODULE_NAME);
        }
    }

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
        pass_group.append_pass(SIMPLIFYCFG_NAME);

        match build_config.optimization_level {
            OptLevel::Opt1 => {
                pass_group.append_pass(SROA_NAME);
                pass_group.append_pass(MEM2REG_NAME);
                pass_group.append_pass(DCE_NAME);
            }
            OptLevel::Opt0 => {}
        }
    }

    if build_config.print_ir {
        pass_group.append_pass(MODULEPRINTER_NAME);
    }

    // Run the passes.
    let res = if let Err(ir_error) = pass_mgr.run(&mut ir, &pass_group) {
        Err(handler.emit_err(CompileError::InternalOwned(
            ir_error.to_string(),
            span::Span::dummy(),
        )))
    } else {
        Ok(())
    };
    res?;

    let final_asm = compile_ir_to_asm(handler, &ir, Some(build_config))?;

    Ok(final_asm)
}

/// Given input Sway source code, compile to [CompiledBytecode], containing the asm in bytecode form.
#[allow(clippy::too_many_arguments)]
pub fn compile_to_bytecode(
    handler: &Handler,
    engines: &Engines,
    input: Arc<str>,
    initial_namespace: namespace::Module,
    build_config: BuildConfig,
    source_map: &mut SourceMap,
    package_name: &str,
) -> Result<CompiledBytecode, ErrorEmitted> {
    let asm_res = compile_to_asm(
        handler,
        engines,
        input,
        initial_namespace,
        build_config,
        package_name,
    )?;
    asm_to_bytecode(handler, asm_res, source_map, engines.se())
}

/// Given the assembly (opcodes), compile to [CompiledBytecode], containing the asm in bytecode form.
pub fn asm_to_bytecode(
    handler: &Handler,
    mut asm: CompiledAsm,
    source_map: &mut SourceMap,
    source_engine: &SourceEngine,
) -> Result<CompiledBytecode, ErrorEmitted> {
    let compiled_bytecode = asm.0.to_bytecode_mut(handler, source_map, source_engine)?;
    Ok(compiled_bytecode)
}

/// Given a [ty::TyProgram], which is type-checked Sway source, construct a graph to analyze
/// control flow and determine if it is valid.
fn perform_control_flow_analysis(
    handler: &Handler,
    engines: &Engines,
    program: &ty::TyProgram,
    print_graph: Option<String>,
    print_graph_url_format: Option<String>,
) -> Result<(), ErrorEmitted> {
    let dca_res = dead_code_analysis(handler, engines, program);
    let rpa_errors = return_path_analysis(engines, program);
    let rpa_res = handler.scope(|handler| {
        for err in rpa_errors {
            handler.emit_err(err);
        }
        Ok(())
    });

    if let Ok(graph) = dca_res.clone() {
        graph.visualize(engines, print_graph, print_graph_url_format);
    }
    dca_res?;
    rpa_res
}

/// Constructs a dead code graph from all modules within the graph and then attempts to find dead
/// code.
///
/// Returns the graph that was used for analysis.
fn dead_code_analysis<'a>(
    handler: &Handler,
    engines: &'a Engines,
    program: &ty::TyProgram,
) -> Result<ControlFlowGraph<'a>, ErrorEmitted> {
    let decl_engine = engines.de();
    let mut dead_code_graph = Default::default();
    let tree_type = program.kind.tree_type();
    module_dead_code_analysis(
        handler,
        engines,
        &program.root,
        &tree_type,
        &mut dead_code_graph,
    )?;
    let warnings = dead_code_graph.find_dead_code(decl_engine);
    for warn in warnings {
        handler.emit_warn(warn)
    }
    Ok(dead_code_graph)
}

/// Recursively collect modules into the given `ControlFlowGraph` ready for dead code analysis.
fn module_dead_code_analysis<'eng: 'cfg, 'cfg>(
    handler: &Handler,
    engines: &'eng Engines,
    module: &ty::TyModule,
    tree_type: &parsed::TreeType,
    graph: &mut ControlFlowGraph<'cfg>,
) -> Result<(), ErrorEmitted> {
    module.submodules.iter().try_fold((), |_, (_, submodule)| {
        let tree_type = parsed::TreeType::Library;
        module_dead_code_analysis(handler, engines, &submodule.module, &tree_type, graph)
    })?;
    let res = {
        ControlFlowGraph::append_module_to_dead_code_graph(
            engines,
            &module.all_nodes,
            tree_type,
            graph,
        )
        .map_err(|err| handler.emit_err(err))
    };
    graph.connect_pending_entry_edges();
    res
}

fn return_path_analysis(engines: &Engines, program: &ty::TyProgram) -> Vec<CompileError> {
    let mut errors = vec![];
    module_return_path_analysis(engines, &program.root, &mut errors);
    errors
}

fn module_return_path_analysis(
    engines: &Engines,
    module: &ty::TyModule,
    errors: &mut Vec<CompileError>,
) {
    for (_, submodule) in &module.submodules {
        module_return_path_analysis(engines, &submodule.module, errors);
    }
    let graph = ControlFlowGraph::construct_return_path_graph(engines, &module.all_nodes);
    match graph {
        Ok(graph) => errors.extend(graph.analyze_return_paths(engines)),
        Err(mut error) => errors.append(&mut error),
    }
}

/// Check if the retrigger compilation flag has been set to true in the language server.
/// If it has, there is a new compilation request, so we should abort the current compilation.
fn check_should_abort(
    handler: &Handler,
    retrigger_compilation: Option<Arc<AtomicBool>>,
) -> Result<(), ErrorEmitted> {
    if let Some(ref retrigger_compilation) = retrigger_compilation {
        if retrigger_compilation.load(Ordering::SeqCst) {
            return Err(handler.cancel());
        }
    }
    Ok(())
}

#[test]
fn test_basic_prog() {
    let handler = Handler::default();
    let engines = Engines::default();
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
        &handler,
        &engines,
        None,
    );
    prog.unwrap();
}
#[test]
fn test_parenthesized() {
    let handler = Handler::default();
    let engines = Engines::default();
    let prog = parse(
        r#"
        contract;
        pub fn some_abi_func() -> unit {
            let x = (5 + 6 / (1 + (2 / 1) + 4));
            return;
        }
    "#
        .into(),
        &handler,
        &engines,
        None,
    );
    prog.unwrap();
}

#[test]
fn test_unary_ordering() {
    use crate::language::{self, parsed};
    let handler = Handler::default();
    let engines = Engines::default();
    let prog = parse(
        r#"
    script;
    fn main() -> bool {
        let a = true;
        let b = true;
        !a && b;
    }"#
        .into(),
        &handler,
        &engines,
        None,
    );
    let (.., prog) = prog.unwrap();
    // this should parse as `(!a) && b`, not `!(a && b)`. So, the top level
    // expression should be `&&`
    if let parsed::AstNode {
        content:
            parsed::AstNodeContent::Declaration(parsed::Declaration::FunctionDeclaration(decl_id)),
        ..
    } = &prog.root.tree.root_nodes[0]
    {
        let fn_decl = engines.pe().get_function(decl_id);
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
        } = &fn_decl.body.contents[2]
        {
            assert_eq!(op, &language::LazyOp::And)
        } else {
            panic!("Was not lazy operator.")
        }
    } else {
        panic!("Was not ast node")
    };
}

#[test]
fn test_parser_recovery() {
    let handler = Handler::default();
    let engines = Engines::default();
    let prog = parse(
        r#"
    script;
    fn main() -> bool {
        let
        let a = true;
        true
    }"#
        .into(),
        &handler,
        &engines,
        None,
    );
    let (_, _) = prog.unwrap();
    assert!(handler.has_errors());
    dbg!(handler);
}
