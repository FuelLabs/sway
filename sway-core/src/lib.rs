#![recursion_limit = "256"]

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
mod debug_generation;
pub mod decl_engine;
pub mod ir_generation;
pub mod language;
pub mod marker_traits;
mod metadata;
pub mod obs_engine;
pub mod query_engine;
pub mod semantic_analysis;
pub mod source_map;
pub mod transform;
pub mod type_system;

use crate::ir_generation::check_function_purity;
use crate::language::{CallPath, CallPathType};
use crate::query_engine::ModuleCacheEntry;
use crate::semantic_analysis::namespace::ResolvedDeclaration;
use crate::semantic_analysis::type_resolve::{resolve_call_path, VisibilityCheck};
use crate::source_map::SourceMap;
pub use asm_generation::from_ir::compile_ir_context_to_finalized_asm;
use asm_generation::FinalizedAsm;
pub use asm_generation::{CompiledBytecode, FinalizedEntry};
pub use build_config::DbgGeneration;
pub use build_config::{
    Backtrace, BuildBackend, BuildConfig, BuildTarget, IrCli, LspConfig, OptLevel, PrintAsm,
};
use control_flow_analysis::ControlFlowGraph;
pub use debug_generation::write_dwarf;
use itertools::Itertools;
use metadata::MetadataManager;
use query_engine::{ModuleCacheKey, ModuleCommonInfo, ParsedModuleInfo, ProgramsCacheEntry};
use semantic_analysis::program::TypeCheckFailed;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use sway_ast::AttributeDecl;
use sway_error::convert_parse_tree_error::ConvertParseTreeError;
use sway_error::handler::{ErrorEmitted, Handler};
use sway_error::warning::{CollectedTraitImpl, CompileInfo, CompileWarning, Info, Warning};
use sway_features::ExperimentalFeatures;
use sway_ir::{
    create_o1_pass_group, register_known_passes, Context, Kind, Module, PassGroup, PassManager,
    PrintPassesOpts, VerifyPassesOpts, ARG_DEMOTION_NAME, ARG_POINTEE_MUTABILITY_TAGGER_NAME,
    CONST_DEMOTION_NAME, DCE_NAME, FN_DEDUP_DEBUG_PROFILE_NAME, FN_INLINE_NAME, GLOBALS_DCE_NAME,
    MEM2REG_NAME, MEMCPYOPT_NAME, MEMCPYPROP_REVERSE_NAME, MISC_DEMOTION_NAME, RET_DEMOTION_NAME,
    SIMPLIFY_CFG_NAME, SROA_NAME,
};
#[cfg(feature = "llvm-backend")]
use sway_llvm::{lower_module_to_string, BackendOptions};
use sway_types::span::Source;
use sway_types::{SourceEngine, SourceLocation, Span};
use sway_utils::{time_expr, PerformanceData, PerformanceMetric};
use transform::{ArgsExpectValues, Attribute, AttributeKind, Attributes, ExpectedArgs};
use types::{CollectTypesMetadata, CollectTypesMetadataContext, LogId, TypeMetadata};

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

pub use engine_threading::Engines;
pub use obs_engine::{ObservabilityEngine, Observer};

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
    src: Source,
    handler: &Handler,
    engines: &Engines,
    config: Option<&BuildConfig>,
    experimental: ExperimentalFeatures,
    package_name: &str,
) -> Result<(lexed::LexedProgram, parsed::ParseProgram), ErrorEmitted> {
    match config {
        None => parse_in_memory(
            handler,
            engines,
            src,
            experimental,
            DbgGeneration::None,
            package_name,
        ),
        // When a `BuildConfig` is given,
        // the module source may declare `mod`s that must be parsed from other files.
        Some(config) => parse_module_tree(
            handler,
            engines,
            src,
            config.canonical_root_module(),
            None,
            config.build_target,
            config.dbg_generation,
            config.include_tests,
            experimental,
            config.lsp_mode.as_ref(),
            package_name,
        )
        .map(
            |ParsedModuleTree {
                 tree_type: kind,
                 lexed_module,
                 parse_module,
             }| {
                let lexed = lexed::LexedProgram {
                    kind,
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
pub fn parse_tree_type(handler: &Handler, src: Source) -> Result<parsed::TreeType, ErrorEmitted> {
    // Parsing only the module kind does not depend on any
    // experimental feature. So, we can just pass the default
    // experimental features here.
    let experimental = ExperimentalFeatures::default();
    sway_parse::parse_module_kind(handler, src, None, experimental)
        .map(|kind| convert_module_kind(&kind))
}

/// Converts `attribute_decls` to [Attributes].
///
/// This function always returns [Attributes], even if the attributes are erroneous.
/// Errors and warnings are returned via [Handler]. The callers should ignore eventual errors
/// in attributes and proceed with the compilation. [Attributes] are tolerant to erroneous
/// attributes and follows the last-wins principle, which allows annotated elements to
/// proceed with compilation. After their successful compilation, callers need to inspect
/// the [Handler] and still emit errors if there were any.
pub(crate) fn attr_decls_to_attributes(
    attribute_decls: &[AttributeDecl],
    can_annotate: impl Fn(&Attribute) -> bool,
    target_friendly_name: &'static str,
) -> (Handler, Attributes) {
    let handler = Handler::default();
    // Check if attribute is an unsupported inner attribute (`#!`).
    // Note that we are doing that before creating the flattened `attributes`,
    // because we want the error to point at the `#!` token.
    // Note also that we will still include those attributes into
    // the `attributes`. There are cases, like e.g., LSP, where
    // having complete list of attributes is needed.
    // In the below analysis, though, we will be ignoring inner attributes,
    // means not checking their content.
    for attr_decl in attribute_decls
        .iter()
        .filter(|attr| !attr.is_doc_comment() && attr.is_inner())
    {
        handler.emit_err(CompileError::Unimplemented {
            span: attr_decl.hash_kind.span(),
            feature: "Using inner attributes (`#!`)".to_string(),
            help: vec![],
        });
    }

    let attributes = Attributes::new(attribute_decls);

    // Check for unknown attributes.
    for attribute in attributes.unknown().filter(|attr| attr.is_outer()) {
        handler.emit_warn(CompileWarning {
            span: attribute.name.span(),
            warning_content: Warning::UnknownAttribute {
                attribute: (&attribute.name).into(),
                known_attributes: attributes.known_attribute_names(),
            },
        });
    }

    // Check for attributes annotating invalid targets.
    for ((attribute_kind, _attribute_direction), mut attributes) in &attributes
        .all()
        .filter(|attr| attr.is_doc_comment() || attr.is_outer())
        .chunk_by(|attr| (attr.kind, attr.direction))
    {
        // For doc comments, we want to show the error on a complete doc comment,
        // and not on every documentation line.
        if attribute_kind == AttributeKind::DocComment {
            let first_doc_line = attributes
                .next()
                .expect("`chunk_by` guarantees existence of at least one element in the chunk");
            if !can_annotate(first_doc_line) {
                let last_doc_line = match attributes.last() {
                    Some(last_attr) => last_attr,
                    // There is only one doc line in the complete doc comment.
                    None => first_doc_line,
                };
                handler.emit_err(
                    ConvertParseTreeError::InvalidAttributeTarget {
                        span: Span::join(
                            first_doc_line.span.clone(),
                            &last_doc_line.span.start_span(),
                        ),
                        attribute: first_doc_line.name.clone(),
                        target_friendly_name,
                        can_only_annotate_help: first_doc_line
                            .can_only_annotate_help(target_friendly_name),
                    }
                    .into(),
                );
            }
        } else {
            // For other attributes, the error is shown for every individual attribute.
            for attribute in attributes {
                if !can_annotate(attribute) {
                    handler.emit_err(
                        ConvertParseTreeError::InvalidAttributeTarget {
                            span: attribute.name.span(),
                            attribute: attribute.name.clone(),
                            target_friendly_name,
                            can_only_annotate_help: attribute
                                .can_only_annotate_help(target_friendly_name),
                        }
                        .into(),
                    );
                }
            }
        }
    }

    // In all the subsequent test we are checking only non-doc-comment attributes
    // and only those that didn't produce invalid target or unsupported inner attributes errors.
    let should_be_checked =
        |attr: &&Attribute| !attr.is_doc_comment() && attr.is_outer() && can_annotate(attr);

    // Check for attributes multiplicity.
    for (_attribute_kind, attributes_of_kind) in
        attributes.all_by_kind(|attr| should_be_checked(attr) && !attr.kind.allows_multiple())
    {
        if attributes_of_kind.len() > 1 {
            let (last_attribute, previous_attributes) = attributes_of_kind
                .split_last()
                .expect("`attributes_of_kind` has more than one element");
            handler.emit_err(
                ConvertParseTreeError::InvalidAttributeMultiplicity {
                    last_occurrence: (&last_attribute.name).into(),
                    previous_occurrences: previous_attributes
                        .iter()
                        .map(|attr| (&attr.name).into())
                        .collect(),
                }
                .into(),
            );
        }
    }

    // Check for arguments multiplicity.
    // For attributes that can be applied only once but are applied several times
    // we will still check arguments in every attribute occurrence.
    for attribute in attributes.all().filter(should_be_checked) {
        let _ = attribute.check_args_multiplicity(&handler);
    }

    // Check for expected arguments.
    // For attributes that can be applied only once but are applied more times
    // we will check arguments of every attribute occurrence.
    // If an attribute does not expect any arguments, we will not check them,
    // but emit only the above error about invalid number of arguments.
    for attribute in attributes
        .all()
        .filter(|attr| should_be_checked(attr) && attr.can_have_arguments())
    {
        match attribute.expected_args() {
            ExpectedArgs::None => unreachable!("`attribute` can have arguments"),
            ExpectedArgs::Any => {}
            ExpectedArgs::MustBeIn(expected_args) => {
                for arg in attribute.args.iter() {
                    if !expected_args.contains(&arg.name.as_str()) {
                        handler.emit_err(
                            ConvertParseTreeError::InvalidAttributeArg {
                                attribute: attribute.name.clone(),
                                arg: (&arg.name).into(),
                                expected_args: expected_args.clone(),
                            }
                            .into(),
                        );
                    }
                }
            }
            ExpectedArgs::ShouldBeIn(expected_args) => {
                for arg in attribute.args.iter() {
                    if !expected_args.contains(&arg.name.as_str()) {
                        handler.emit_warn(CompileWarning {
                            span: arg.name.span(),
                            warning_content: Warning::UnknownAttributeArg {
                                attribute: attribute.name.clone(),
                                arg: (&arg.name).into(),
                                expected_args: expected_args.clone(),
                            },
                        });
                    }
                }
            }
        }
    }

    // Check for expected argument values.
    // We use here the same logic for what to check, as in the above check
    // for expected arguments.
    for attribute in attributes
        .all()
        .filter(|attr| should_be_checked(attr) && attr.can_have_arguments())
    {
        // In addition, if an argument **must** be in expected args but is not,
        // we will not be checking it, but only emit the error above.
        // But if it **should** be in expected args and is not,
        // we still impose on it the expectation coming from its attribute.
        fn check_value_expected(handler: &Handler, attribute: &Attribute, is_value_expected: bool) {
            for arg in attribute.args.iter() {
                if let ExpectedArgs::MustBeIn(expected_args) = attribute.expected_args() {
                    if !expected_args.contains(&arg.name.as_str()) {
                        continue;
                    }
                }

                if (is_value_expected && arg.value.is_none())
                    || (!is_value_expected && arg.value.is_some())
                {
                    handler.emit_err(
                        ConvertParseTreeError::InvalidAttributeArgExpectsValue {
                            attribute: attribute.name.clone(),
                            arg: (&arg.name).into(),
                            value_span: arg.value.as_ref().map(|literal| literal.span()),
                        }
                        .into(),
                    );
                }
            }
        }

        match attribute.args_expect_values() {
            ArgsExpectValues::Yes => check_value_expected(&handler, attribute, true),
            ArgsExpectValues::No => check_value_expected(&handler, attribute, false),
            ArgsExpectValues::Maybe => {}
        }
    }

    (handler, attributes)
}

/// When no `BuildConfig` is given, we're assumed to be parsing in-memory with no submodules.
fn parse_in_memory(
    handler: &Handler,
    engines: &Engines,
    src: Source,
    experimental: ExperimentalFeatures,
    dbg_generation: DbgGeneration,
    package_name: &str,
) -> Result<(lexed::LexedProgram, parsed::ParseProgram), ErrorEmitted> {
    let mut hasher = DefaultHasher::new();
    src.text.hash(&mut hasher);
    let hash = hasher.finish();
    let module = sway_parse::parse_file(handler, src, None, experimental)?;

    let (attributes_handler, attributes) = attr_decls_to_attributes(
        &module.attributes,
        |attr| attr.can_annotate_module_kind(),
        module.value.kind.friendly_name(),
    );
    let attributes_error_emitted = handler.append(attributes_handler);

    let (kind, tree) = to_parsed_lang::convert_parse_tree(
        &mut to_parsed_lang::Context::new(
            BuildTarget::EVM,
            dbg_generation,
            experimental,
            package_name,
        ),
        handler,
        engines,
        module.value.clone(),
    )?;

    match attributes_error_emitted {
        Some(err) => Err(err),
        None => {
            let root = parsed::ParseModule {
                span: span::Span::dummy(),
                module_kind_span: module.value.kind.span(),
                module_eval_order: vec![],
                tree,
                submodules: vec![],
                attributes,
                hash,
            };
            let lexed_program = lexed::LexedProgram::new(
                kind,
                lexed::LexedModule {
                    tree: module,
                    submodules: vec![],
                },
            );
            Ok((lexed_program, parsed::ParseProgram { kind, root }))
        }
    }
}

pub struct Submodule {
    name: Ident,
    path: Arc<PathBuf>,
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
    dbg_generation: DbgGeneration,
    include_tests: bool,
    experimental: ExperimentalFeatures,
    lsp_mode: Option<&LspConfig>,
    package_name: &str,
) -> Submodules {
    // Assume the happy path, so there'll be as many submodules as dependencies, but no more.
    let mut submods = Vec::with_capacity(module.submodules().count());
    module.submodules().for_each(|submod| {
        // Read the source code from the dependency.
        // If we cannot, record as an error, but continue with other files.
        let submod_path = Arc::new(module_path(module_dir, module_name, submod));
        let submod_src: Source = match std::fs::read_to_string(&*submod_path) {
            Ok(s) => s.as_str().into(),
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
            submod_src.clone(),
            submod_path.clone(),
            Some(submod.name.as_str()),
            build_target,
            dbg_generation,
            include_tests,
            experimental,
            lsp_mode,
            package_name,
        ) {
            if !matches!(kind, parsed::TreeType::Library) {
                let source_id = engines.se().get_source_id(submod_path.as_ref());
                let span = span::Span::new(submod_src, 0, 0, Some(source_id)).unwrap();
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
    src: Source,
    path: Arc<PathBuf>,
    module_name: Option<&str>,
    build_target: BuildTarget,
    dbg_generation: DbgGeneration,
    include_tests: bool,
    experimental: ExperimentalFeatures,
    lsp_mode: Option<&LspConfig>,
    package_name: &str,
) -> Result<ParsedModuleTree, ErrorEmitted> {
    let query_engine = engines.qe();

    // Parse this module first.
    let module_dir = path.parent().expect("module file has no parent directory");
    let source_id = engines.se().get_source_id(&path.clone());
    // don't use reloaded file if we already have it in memory, that way new spans will still point to the same string
    let src = engines.se().get_or_create_source_buffer(&source_id, src);
    let module = sway_parse::parse_file(handler, src.clone(), Some(source_id), experimental)?;

    // Parse all submodules before converting to the `ParseTree`.
    // This always recovers on parse errors for the file itself by skipping that file.
    let submodules = parse_submodules(
        handler,
        engines,
        module_name,
        &module.value,
        module_dir,
        build_target,
        dbg_generation,
        include_tests,
        experimental,
        lsp_mode,
        package_name,
    );

    let (attributes_handler, attributes) = attr_decls_to_attributes(
        &module.attributes,
        |attr| attr.can_annotate_module_kind(),
        module.value.kind.friendly_name(),
    );
    let attributes_error_emitted = handler.append(attributes_handler);

    // Convert from the raw parsed module to the `ParseTree` ready for type-check.
    let (kind, tree) = to_parsed_lang::convert_parse_tree(
        &mut to_parsed_lang::Context::new(build_target, dbg_generation, experimental, package_name),
        handler,
        engines,
        module.value.clone(),
    )?;

    if let Some(err) = attributes_error_emitted {
        return Err(err);
    }

    let module_kind_span = module.value.kind.span();
    let lexed_submodules = submodules
        .iter()
        .map(|s| (s.name.clone(), s.lexed.clone()))
        .collect::<Vec<_>>();
    let lexed = lexed::LexedModule {
        tree: module,
        submodules: lexed_submodules,
    };

    let mut hasher = DefaultHasher::new();
    src.text.hash(&mut hasher);
    let hash = hasher.finish();

    let parsed_submodules = submodules
        .iter()
        .map(|s| (s.name.clone(), s.parsed.clone()))
        .collect::<Vec<_>>();
    let parsed = parsed::ParseModule {
        span: span::Span::new(src, 0, 0, Some(source_id)).unwrap(),
        module_kind_span,
        module_eval_order: vec![],
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
    let version = lsp_mode
        .and_then(|lsp| lsp.file_versions.get(path.as_ref()).copied())
        .unwrap_or(None);

    let common_info = ModuleCommonInfo {
        path: path.clone(),
        include_tests,
        dependencies,
        hash,
    };
    let parsed_info = ParsedModuleInfo {
        modified_time,
        version,
    };
    let cache_entry = ModuleCacheEntry::new(common_info, parsed_info);
    query_engine.update_or_insert_parsed_module_cache_entry(cache_entry);

    Ok(ParsedModuleTree {
        tree_type: kind,
        lexed_module: lexed,
        parse_module: parsed,
    })
}

/// Checks if the typed module cache for a given path is up to date.
///
/// This function determines whether the cached typed representation of a module
/// is still valid based on file versions and dependencies.
///
/// Note: This functionality is currently only supported when the compiler is
/// initiated from the language server.
pub(crate) fn is_ty_module_cache_up_to_date(
    engines: &Engines,
    path: &Arc<PathBuf>,
    include_tests: bool,
    build_config: Option<&BuildConfig>,
) -> bool {
    let cache = engines.qe().module_cache.read();
    let key = ModuleCacheKey::new(path.clone(), include_tests);
    cache.get(&key).is_some_and(|entry| {
        entry.typed.as_ref().is_some_and(|typed| {
            // Check if the cache is up to date based on file versions
            let cache_up_to_date = build_config
                .and_then(|x| x.lsp_mode.as_ref())
                .and_then(|lsp| lsp.file_versions.get(path.as_ref()))
                .is_none_or(|version| {
                    version.is_none_or(|v| typed.version.is_some_and(|tv| v <= tv))
                });

            // If the cache is up to date, recursively check all dependencies
            cache_up_to_date
                && entry.common.dependencies.iter().all(|dep_path| {
                    is_ty_module_cache_up_to_date(engines, dep_path, include_tests, build_config)
                })
        })
    })
}

/// Checks if the parsed module cache for a given path is up to date.
///
/// This function determines whether the cached parsed representation of a module
/// is still valid based on file versions, modification times, or content hashes.
pub(crate) fn is_parse_module_cache_up_to_date(
    engines: &Engines,
    path: &Arc<PathBuf>,
    include_tests: bool,
    build_config: Option<&BuildConfig>,
) -> bool {
    let cache = engines.qe().module_cache.read();
    let key = ModuleCacheKey::new(path.clone(), include_tests);
    cache.get(&key).is_some_and(|entry| {
        // Determine if the cached dependency information is still valid
        let cache_up_to_date = build_config
            .and_then(|x| x.lsp_mode.as_ref())
            .and_then(|lsp| lsp.file_versions.get(path.as_ref()))
            .map_or_else(
                || {
                    // If LSP mode is not active or file version is unavailable, fall back to filesystem checks.
                    let modified_time = std::fs::metadata(path.as_path())
                        .ok()
                        .and_then(|m| m.modified().ok());
                    // Check if modification time matches, or if not, compare file content hash
                    entry.parsed.modified_time == modified_time || {
                        let src = std::fs::read_to_string(path.as_path()).unwrap();
                        let mut hasher = DefaultHasher::new();
                        src.hash(&mut hasher);
                        hasher.finish() == entry.common.hash
                    }
                },
                |version| {
                    // Determine if the parse cache is up-to-date in LSP mode:
                    // - If there's no LSP file version (version is None), consider the cache up-to-date.
                    // - If there is an LSP file version:
                    //   - If there's no cached version (entry.parsed.version is None), the cache is outdated.
                    //   - If there's a cached version, compare them: cache is up-to-date if the LSP file version
                    //     is not greater than the cached version.
                    version.is_none_or(|v| entry.parsed.version.is_some_and(|ev| v <= ev))
                },
            );

        // Checks if the typed module cache for a given path is up to date// If the cache is up to date, recursively check all dependencies to make sure they have not been
        // modified either.
        cache_up_to_date
            && entry.common.dependencies.iter().all(|dep_path| {
                is_parse_module_cache_up_to_date(engines, dep_path, include_tests, build_config)
            })
    })
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

pub fn build_module_dep_graph(
    handler: &Handler,
    parse_module: &mut parsed::ParseModule,
) -> Result<(), ErrorEmitted> {
    let module_dep_graph = ty::TyModule::build_dep_graph(handler, parse_module)?;
    parse_module.module_eval_order = module_dep_graph.compute_order(handler)?;

    for (_, submodule) in &mut parse_module.submodules {
        build_module_dep_graph(handler, &mut submodule.module)?;
    }
    Ok(())
}

/// A possible occurrence of a `panic` expression that is located in code at [PanicOccurrence::loc].
///
/// Note that a single `panic` expression can have multiple [PanicOccurrence]s related to it.
///
/// For example:
/// - `panic "Some message.";` will have just a single occurrence, with `msg` containing the message.
/// - `panic some_value_of_a_concrete_type;` will have just a single occurrence, with `log_id` containing the [LogId] of the concrete type.
/// - `panic some_value_of_a_generic_type;` will have multiple occurrences, one with `log_id` for every monomorphized type.
///
/// **Every [PanicOccurrence] has exactly one revert code assigned to it.**
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PanicOccurrence {
    pub function: String,
    pub loc: SourceLocation,
    pub log_id: Option<LogId>,
    pub msg: Option<String>,
}

/// Represents a function call that could panic during execution.
/// E.g., for the following code:
///
/// ```ignore
/// fn some_function() {
///    let _ = this_function_might_panic(42);
///}
/// ```
///
/// the `function` field will contain the name of the function that might panic:
///   `function: "some_other_package::module::this_function_might_panic"`
///
/// and the `loc` and `caller_function` fields will contain the source location of the call to the `function`
/// that might panic:
///
/// ```ignore
///     caller_function: "some_package::some_module::some_function",
///     pkg: "some_package@0.1.0",
///     file: "src/some_module.sw",
///     ...
/// ```
///
/// Note that, in case of panicking function or caller function being
/// generic functions, a single panicking call can have multiple
/// [PanickingCallOccurrence]s related to it.
///
/// For example:
/// - `this_function_might_panic(42);` will have a single occurrence,
///   with `function` containing the full name of the function that might panic.
/// - `this_generic_function_might_panic::<u64>(42);` will have a single occurrence,
///   with `function` containing the full name of the function that might panic,
///   but with the generic type parameter `u64` included in the name.
/// - `this_generic_function_might_panic::<T>(42);` will have multiple occurrences,
///   one for every monomorphized type.
///
/// Similar is for a generic caller function.
///
/// **Every [PanickingCallOccurrence] has exactly one panicking call code assigned to it.**
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PanickingCallOccurrence {
    pub function: String,
    pub caller_function: String,
    pub loc: SourceLocation,
}

/// [PanicOccurrence]s mapped to their corresponding panic error codes.
pub type PanicOccurrences = HashMap<PanicOccurrence, u64>;

/// [PanickingCallOccurrence]s mapped to their corresponding panicking call codes.
pub type PanickingCallOccurrences = HashMap<PanickingCallOccurrence, u64>;

pub enum CompiledAsm {
    Fuel {
        finalized_asm: Box<FinalizedAsm>,
        panic_occurrences: PanicOccurrences,
        panicking_call_occurrences: PanickingCallOccurrences,
    },
    LLVM {
        llvm_ir: String,
    },
}

#[allow(clippy::result_large_err)]
#[allow(clippy::too_many_arguments)]
pub fn parsed_to_ast(
    handler: &Handler,
    engines: &Engines,
    parse_program: &mut parsed::ParseProgram,
    initial_namespace: namespace::Package,
    build_config: Option<&BuildConfig>,
    package_name: &str,
    retrigger_compilation: Option<Arc<AtomicBool>>,
    experimental: ExperimentalFeatures,
    backtrace: Backtrace,
) -> Result<ty::TyProgram, TypeCheckFailed> {
    let lsp_config = build_config.map(|x| x.lsp_mode.clone()).unwrap_or_default();

    // Build the dependency graph for the submodules.
    build_module_dep_graph(handler, &mut parse_program.root).map_err(|error| TypeCheckFailed {
        root_module: None,
        namespace: initial_namespace.clone(),
        error,
    })?;

    let collection_namespace = Namespace::new(handler, engines, initial_namespace.clone(), true)
        .map_err(|error| TypeCheckFailed {
            root_module: None,
            namespace: initial_namespace.clone(),
            error,
        })?;
    // Collect the program symbols.

    let mut collection_ctx =
        ty::TyProgram::collect(handler, engines, parse_program, collection_namespace).map_err(
            |error| TypeCheckFailed {
                root_module: None,
                namespace: initial_namespace.clone(),
                error,
            },
        )?;

    let typecheck_namespace =
        Namespace::new(handler, engines, initial_namespace, true).map_err(|error| {
            TypeCheckFailed {
                root_module: None,
                namespace: collection_ctx.namespace().current_package_ref().clone(),
                error,
            }
        })?;
    // Type check the program.
    let typed_program_opt = ty::TyProgram::type_check(
        handler,
        engines,
        parse_program,
        &mut collection_ctx,
        typecheck_namespace,
        package_name,
        build_config,
        experimental,
    );

    let mut typed_program = typed_program_opt?;

    check_should_abort(handler, retrigger_compilation.clone()).map_err(|error| {
        TypeCheckFailed {
            root_module: Some(Arc::new(typed_program.root_module.clone())),
            namespace: typed_program.namespace.current_package_ref().clone(),
            error,
        }
    })?;
    // Only clear the parsed AST nodes if we are running a regular compilation pipeline.
    // LSP needs these to build its token map, and they are cleared by `clear_program` as
    // part of the LSP garbage collection functionality instead.
    if lsp_config.is_none() {
        engines.pe().clear();
    }

    typed_program.check_deprecated(engines, handler);

    match typed_program.check_recursive(engines, handler) {
        Ok(()) => {}
        Err(error) => {
            handler.dedup();
            return Err(TypeCheckFailed {
                root_module: Some(Arc::new(typed_program.root_module.clone())),
                namespace: typed_program.namespace.current_package().clone(),
                error,
            });
        }
    };

    // Skip collecting metadata if we triggered an optimised build from LSP.
    let types_metadata = if !lsp_config.as_ref().is_some_and(|lsp| lsp.optimized_build) {
        // Collect information about the types used in this program
        let types_metadata_result = typed_program.collect_types_metadata(
            handler,
            &mut CollectTypesMetadataContext::new(engines, experimental, package_name.to_string()),
        );
        let types_metadata = match types_metadata_result {
            Ok(types_metadata) => types_metadata,
            Err(error) => {
                handler.dedup();
                return Err(TypeCheckFailed {
                    root_module: Some(Arc::new(typed_program.root_module.clone())),
                    namespace: typed_program.namespace.current_package().clone(),
                    error,
                });
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

        check_should_abort(handler, retrigger_compilation.clone()).map_err(|error| {
            TypeCheckFailed {
                root_module: Some(Arc::new(typed_program.root_module.clone())),
                namespace: typed_program.namespace.current_package_ref().clone(),
                error,
            }
        })?;

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
    let mut ctx = Context::new(engines.se(), experimental, backtrace.into());
    let module = Module::new(&mut ctx, Kind::Contract);
    if let Err(errs) = ir_generation::compile::compile_constants_for_package(
        engines,
        &mut ctx,
        module,
        &typed_program.namespace,
    ) {
        errs.into_iter().for_each(|err| {
            handler.emit_err(err.clone());
        });
    }

    // CEI pattern analysis
    let cei_analysis_warnings =
        semantic_analysis::cei_pattern_analysis::analyze_program(engines, &typed_program);
    for warn in cei_analysis_warnings {
        handler.emit_warn(warn);
    }

    let mut md_mgr = MetadataManager::default();
    // Check that all storage initializers can be evaluated at compile time.
    typed_program
        .get_typed_program_with_initialized_storage_slots(
            handler,
            engines,
            &mut ctx,
            &mut md_mgr,
            module,
        )
        .map_err(|error: ErrorEmitted| {
            handler.dedup();
            TypeCheckFailed {
                root_module: Some(Arc::new(typed_program.root_module.clone())),
                namespace: typed_program.namespace.current_package_ref().clone(),
                error,
            }
        })?;

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

    Ok(typed_program)
}

#[allow(clippy::too_many_arguments)]
pub fn compile_to_ast(
    handler: &Handler,
    engines: &Engines,
    src: Source,
    initial_namespace: namespace::Package,
    build_config: Option<&BuildConfig>,
    package_name: &str,
    retrigger_compilation: Option<Arc<AtomicBool>>,
    experimental: ExperimentalFeatures,
) -> Result<Programs, ErrorEmitted> {
    check_should_abort(handler, retrigger_compilation.clone())?;

    let query_engine = engines.qe();
    let mut metrics = PerformanceData::default();
    if let Some(config) = build_config {
        let path = config.canonical_root_module();
        let include_tests = config.include_tests;
        // Check if we can re-use the data in the cache.
        if is_parse_module_cache_up_to_date(engines, &path, include_tests, build_config) {
            let mut entry = query_engine.get_programs_cache_entry(&path).unwrap();
            entry.programs.metrics.reused_programs += 1;

            let (warnings, errors, infos) = entry.handler_data;
            let new_handler = Handler::from_parts(warnings, errors, infos);
            handler.append(new_handler);
            return Ok(entry.programs);
        };
    }

    // Parse the program to a concrete syntax tree (CST).
    let parse_program_opt = time_expr!(
        package_name,
        "parse the program to a concrete syntax tree (CST)",
        "parse_cst",
        parse(
            src,
            handler,
            engines,
            build_config,
            experimental,
            package_name
        ),
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
    if build_config.is_none_or(|config| !config.include_tests) {
        parsed_program.exclude_tests(engines);
    }

    // Type check (+ other static analysis) the CST to a typed AST.
    let program = time_expr!(
        package_name,
        "parse the concrete syntax tree (CST) to a typed AST",
        "parse_ast",
        parsed_to_ast(
            handler,
            engines,
            &mut parsed_program,
            initial_namespace,
            build_config,
            package_name,
            retrigger_compilation.clone(),
            experimental,
            build_config.map(|cfg| cfg.backtrace).unwrap_or_default()
        ),
        build_config,
        metrics
    );

    check_should_abort(handler, retrigger_compilation.clone())?;

    handler.dedup();

    let programs = Programs::new(
        Arc::new(lexed_program),
        Arc::new(parsed_program),
        program.map(Arc::new),
        metrics,
    );

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
    src: Source,
    initial_namespace: namespace::Package,
    build_config: &BuildConfig,
    package_name: &str,
    experimental: ExperimentalFeatures,
) -> Result<CompiledAsm, ErrorEmitted> {
    let ast_res = compile_to_ast(
        handler,
        engines,
        src,
        initial_namespace,
        Some(build_config),
        package_name,
        None,
        experimental,
    )?;

    ast_to_asm(handler, engines, &ast_res, build_config, experimental)
}

/// Given an AST compilation result, try compiling to a `CompiledAsm`,
/// containing the asm in opcode form (not raw bytes/bytecode).
pub fn ast_to_asm(
    handler: &Handler,
    engines: &Engines,
    programs: &Programs,
    build_config: &BuildConfig,
    experimental: ExperimentalFeatures,
) -> Result<CompiledAsm, ErrorEmitted> {
    let typed_program = match &programs.typed {
        Ok(typed_program) => typed_program,
        Err(err) => return Err(err.error),
    };

    let mut panic_occurrences = PanicOccurrences::default();
    let mut panicking_call_occurrences = PanickingCallOccurrences::default();

    match compile_ast_to_ir_to_asm(
        handler,
        engines,
        typed_program,
        &mut panic_occurrences,
        &mut panicking_call_occurrences,
        build_config,
        experimental,
    ) {
        Ok(res) => Ok(res),
        Err(err) => {
            handler.dedup();
            Err(err)
        }
    }
}

pub(crate) fn compile_ast_to_ir_to_asm(
    handler: &Handler,
    engines: &Engines,
    program: &ty::TyProgram,
    panic_occurrences: &mut PanicOccurrences,
    panicking_call_occurrences: &mut PanickingCallOccurrences,
    build_config: &BuildConfig,
    experimental: ExperimentalFeatures,
) -> Result<CompiledAsm, ErrorEmitted> {
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
        panic_occurrences,
        panicking_call_occurrences,
        build_config.include_tests,
        engines,
        experimental,
        build_config.backtrace.into(),
    ) {
        Ok(ir) => ir,
        Err(errors) => {
            let mut last = None;
            for e in errors {
                last = Some(handler.emit_err(e));
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
            // We run a function deduplication pass that only removes duplicate
            // functions when everything, including the metadata are identical.
            pass_group.append_pass(FN_DEDUP_DEBUG_PROFILE_NAME);

            // Inlining is necessary until #4899 is resolved.
            pass_group.append_pass(FN_INLINE_NAME);

            // Do DCE so other optimizations run faster.
            pass_group.append_pass(GLOBALS_DCE_NAME);
            pass_group.append_pass(DCE_NAME);
        }
    }

    // Target specific transforms should be moved into something more configured.
    if build_config.build_target == BuildTarget::Fuel {
        // FuelVM target specific transforms.
        //
        // Demote large by-value constants, arguments and return values to by-reference values
        // using temporaries.
        pass_group.append_pass(CONST_DEMOTION_NAME);
        pass_group.append_pass(ARG_DEMOTION_NAME);
        pass_group.append_pass(RET_DEMOTION_NAME);
        pass_group.append_pass(MISC_DEMOTION_NAME);

        // Convert loads and stores to mem_copies where possible.
        pass_group.append_pass(ARG_POINTEE_MUTABILITY_TAGGER_NAME);
        pass_group.append_pass(MEMCPYOPT_NAME);

        // Run a DCE and simplify-cfg to clean up any obsolete instructions.
        pass_group.append_pass(DCE_NAME);
        pass_group.append_pass(SIMPLIFY_CFG_NAME);

        match build_config.optimization_level {
            OptLevel::Opt1 => {
                pass_group.append_pass(MEMCPYPROP_REVERSE_NAME);
                pass_group.append_pass(SROA_NAME);
                pass_group.append_pass(MEM2REG_NAME);
                pass_group.append_pass(DCE_NAME);
            }
            OptLevel::Opt0 => {}
        }
    }

    // Run the passes.
    let print_passes_opts: PrintPassesOpts = (&build_config.print_ir).into();
    let verify_passes_opts: VerifyPassesOpts = (&build_config.verify_ir).into();
    let res = if let Err(ir_error) = pass_mgr.run_with_print_verify(
        &mut ir,
        &pass_group,
        &print_passes_opts,
        &verify_passes_opts,
    ) {
        Err(handler.emit_err(CompileError::InternalOwned(
            ir_error.to_string(),
            span::Span::dummy(),
        )))
    } else {
        Ok(())
    };
    res?;

    #[cfg(feature = "llvm-backend")]
    {
        if build_config.build_backend == BuildBackend::Fuel {
            if let Some(dump_dest) = std::env::var_os("SWAY_LLVM_DUMP") {
                if let Some(module) = ir.module_iter().next() {
                    match lower_module_to_string(&ir, module, &BackendOptions::default()) {
                        Ok(llvm_ir) => {
                            if dump_dest.is_empty() || dump_dest == "stdout" {
                                println!("{llvm_ir}");
                            } else if let Err(e) = std::fs::write(&dump_dest, llvm_ir) {
                                let err = handler.emit_err(CompileError::InternalOwned(
                                    format!("failed to write LLVM IR to {:?}: {e}", dump_dest),
                                    span::Span::dummy(),
                                ));
                                return Err(err);
                            }
                        }
                        Err(e) => {
                            let err = handler.emit_err(CompileError::InternalOwned(
                                format!("LLVM backend lowering failed: {e}"),
                                span::Span::dummy(),
                            ));
                            return Err(err);
                        }
                    }
                }
            }
        }
    }

    let compiled_result = match build_config.build_backend {
        BuildBackend::Fuel => {
            let finalized_asm =
                compile_ir_context_to_finalized_asm(handler, &ir, Some(build_config))?;
            let fuel_panic_occurrences = std::mem::take(panic_occurrences);
            let fuel_panicking_call_occurrences = std::mem::take(panicking_call_occurrences);
            CompiledAsm::Fuel {
                finalized_asm: Box::new(finalized_asm),
                panic_occurrences: fuel_panic_occurrences,
                panicking_call_occurrences: fuel_panicking_call_occurrences,
            }
        }
        BuildBackend::LLVM => {
            #[cfg(feature = "llvm-backend")]
            {
                if build_config.build_target == BuildTarget::Fuel {
                    let err = handler.emit_err(CompileError::InternalOwned(
                        "LLVM backend is not supported for fuel build target yet".to_string(),
                        span::Span::dummy(),
                    ));
                    return Err(err);
                }
                if let Some(module) = ir.module_iter().next() {
                    match lower_module_to_string(&ir, module, &BackendOptions::default()) {
                        Ok(llvm_ir) => CompiledAsm::LLVM { llvm_ir },
                        Err(err) => {
                            let err = handler.emit_err(CompileError::InternalOwned(
                                format!("LLVM backend lowering failed: {err}"),
                                span::Span::dummy(),
                            ));
                            return Err(err);
                        }
                    }
                } else {
                    let err = handler.emit_err(CompileError::InternalOwned(
                        "LLVM backend lowering requires at least one module to lower".to_string(),
                        span::Span::dummy(),
                    ));
                    return Err(err);
                }
            }
            #[cfg(not(feature = "llvm-backend"))]
            {
                let err = handler.emit_err(CompileError::InternalOwned(
                    "LLVM backend requires the `llvm-backend` feature to be enabled".to_string(),
                    span::Span::dummy(),
                ));
                return Err(err);
            }
        }
    };

    Ok(compiled_result)
}

/// Given input Sway source code, compile to [CompiledBytecode], containing the asm in bytecode form.
#[allow(clippy::too_many_arguments)]
pub fn compile_to_bytecode(
    handler: &Handler,
    engines: &Engines,
    src: Source,
    initial_namespace: namespace::Package,
    build_config: &BuildConfig,
    source_map: &mut SourceMap,
    package_name: &str,
    experimental: ExperimentalFeatures,
) -> Result<CompiledBytecode, ErrorEmitted> {
    let mut asm_res = compile_to_asm(
        handler,
        engines,
        src,
        initial_namespace,
        build_config,
        package_name,
        experimental,
    )?;
    asm_to_bytecode(
        handler,
        &mut asm_res,
        source_map,
        engines.se(),
        build_config,
    )
}

/// Size of the prelude's CONFIGURABLES_OFFSET section, in bytes.
pub const PRELUDE_CONFIGURABLES_SIZE_IN_BYTES: usize = 8;
/// Offset (in bytes) of the CONFIGURABLES_OFFSET section in the prelude.
pub const PRELUDE_CONFIGURABLES_OFFSET_IN_BYTES: usize = 16;
/// Total size of the prelude in bytes. Instructions start right after.
pub const PRELUDE_SIZE_IN_BYTES: usize = 32;

/// Given bytecode, overwrite the existing offset to configurables offset in the prelude with the given one.
pub fn set_bytecode_configurables_offset(
    compiled_bytecode: &mut CompiledBytecode,
    md: &[u8; PRELUDE_CONFIGURABLES_SIZE_IN_BYTES],
) {
    assert!(
        compiled_bytecode.bytecode.len()
            >= PRELUDE_CONFIGURABLES_OFFSET_IN_BYTES + PRELUDE_CONFIGURABLES_SIZE_IN_BYTES
    );
    let code = &mut compiled_bytecode.bytecode;
    for (index, byte) in md.iter().enumerate() {
        code[index + PRELUDE_CONFIGURABLES_OFFSET_IN_BYTES] = *byte;
    }
}

/// Given the assembly (opcodes), compile to [CompiledBytecode], containing the asm in bytecode form.
pub fn asm_to_bytecode(
    handler: &Handler,
    asm: &mut CompiledAsm,
    source_map: &mut SourceMap,
    source_engine: &SourceEngine,
    build_config: &BuildConfig,
) -> Result<CompiledBytecode, ErrorEmitted> {
    let compiled_bytecode = match asm {
        CompiledAsm::Fuel { finalized_asm, .. } => {
            finalized_asm.to_bytecode_mut(handler, source_map, source_engine, build_config)?
        }
        CompiledAsm::LLVM { .. } => {
            return Err(handler.emit_err(CompileError::InternalOwned(
                "LLVM backend does not emit Fuel bytecode".to_string(),
                span::Span::dummy(),
            )))
        }
    };
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
    let mut dead_code_graph = ControlFlowGraph::new(engines);
    let tree_type = program.kind.tree_type();
    module_dead_code_analysis(
        handler,
        engines,
        &program.root_module,
        &tree_type,
        &mut dead_code_graph,
    )?;
    let warnings = dead_code_graph.find_dead_code(decl_engine);
    for warn in warnings {
        handler.emit_warn(warn);
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
    module
        .submodules
        .iter()
        .try_fold((), |(), (_, submodule)| {
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
    module_return_path_analysis(engines, &program.root_module, &mut errors);
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

pub fn dump_trait_impls_for_typename(
    handler: &Handler,
    engines: &Engines,
    namespace: &namespace::Namespace,
    typename: &str,
) -> Result<(), ErrorEmitted> {
    let path: Vec<&str> = typename.split("::").collect();
    let mut call_path = CallPath::fullpath(&path);
    call_path.callpath_type = CallPathType::Ambiguous;

    let pkg_namespace = namespace.current_package_ref();
    let mod_path = [pkg_namespace.root_module().name().clone()];

    let resolve_handler = Handler::default();
    let resolved = resolve_call_path(
        &resolve_handler,
        engines,
        namespace,
        &mod_path,
        &call_path,
        None,
        VisibilityCheck::No,
    );

    if let Ok(resolved) = resolved {
        let module = &pkg_namespace.root_module();

        let mut impls = Vec::new();
        find_trait_impls_for_type(engines, namespace, &resolved, module, &mut impls);

        for ext_pkg in pkg_namespace.external_packages.iter() {
            let ext_module = ext_pkg.1.root_module();
            find_trait_impls_for_type(engines, namespace, &resolved, ext_module, &mut impls);
        }

        let unique_impls = impls
            .iter()
            .unique_by(|i| i.impl_span.clone())
            .cloned()
            .collect::<Vec<_>>();
        handler.emit_info(CompileInfo {
            span: resolved.span(engines).subset_first_of("{").unwrap(),
            content: Info::ImplTraitsForType {
                impls: unique_impls,
            },
        });
    }

    Ok(())
}

fn find_trait_impls_for_type(
    engines: &Engines,
    namespace: &namespace::Namespace,
    resolved_decl: &ResolvedDeclaration,
    module: &namespace::Module,
    impls: &mut Vec<CollectedTraitImpl>,
) {
    let handler = Handler::default();
    let struct_decl_source_id = resolved_decl
        .to_struct_decl(&handler, engines)
        .map(|d| d.expect_typed())
        .and_then(|decl| decl.to_struct_decl(&handler, engines))
        .map(|decl_id| engines.de().get_struct(&decl_id).span.source_id().cloned())
        .ok()
        .flatten();

    let enum_decl_source_id = resolved_decl
        .to_enum_decl(&handler, engines)
        .map(|d| d.expect_typed())
        .and_then(|decl| decl.to_enum_id(&handler, engines))
        .map(|decl_id| engines.de().get_enum(&decl_id).span.source_id().cloned())
        .ok()
        .flatten();

    module.walk_scope_chain(|lexical_scope| {
        module.submodules().iter().for_each(|(_, sub)| {
            find_trait_impls_for_type(engines, namespace, resolved_decl, sub, impls);
        });

        let trait_map = &lexical_scope.items.implemented_traits;

        for key in trait_map.trait_impls.keys() {
            for trait_entry in trait_map.trait_impls[key].iter() {
                let trait_type = engines.te().get(trait_entry.inner.key.type_id);

                let matched = match *trait_type {
                    TypeInfo::Enum(decl_id) => {
                        let trait_enum = engines.de().get_enum(&decl_id);
                        enum_decl_source_id == trait_enum.span.source_id().cloned()
                    }
                    TypeInfo::Struct(decl_id) => {
                        let trait_struct = engines.de().get_struct(&decl_id);
                        struct_decl_source_id == trait_struct.span.source_id().cloned()
                    }
                    _ => false,
                };

                if matched {
                    let trait_callpath = trait_entry.inner.key.name.to_fullpath(engines, namespace);
                    impls.push(CollectedTraitImpl {
                        impl_span: trait_entry
                            .inner
                            .value
                            .impl_span
                            .subset_first_of("{")
                            .unwrap(),
                        trait_name: engines.help_out(trait_callpath).to_string(),
                    });
                }
            }
        }
    });
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
        ExperimentalFeatures::default(),
        "test",
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
        ExperimentalFeatures::default(),
        "test",
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
        ExperimentalFeatures::default(),
        "test",
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
        ExperimentalFeatures::default(),
        "test",
    );
    let (_, _) = prog.unwrap();
    assert!(handler.has_errors());
    dbg!(handler);
}
