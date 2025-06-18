use crate::{
    capabilities::{
        self,
        diagnostic::DiagnosticMap,
        runnable::{Runnable, RunnableMainFn, RunnableTestFn},
    },
    core::{
        sync::SyncWorkspace,
        token::{self, TypedAstToken},
        token_map::{TokenMap, TokenMapExt},
    },
    error::{DirectoryError, DocumentError, LanguageServerError},
    server_state::{self, CompilationContext, CompiledPrograms, RunnableMap},
    traverse::{
        dependency, lexed_tree::LexedTree, parsed_tree::ParsedTree, typed_tree::TypedTree,
        ParseContext,
    },
};
use forc_pkg as pkg;
use lsp_types::{
    CompletionItem, DocumentSymbol, GotoDefinitionResponse, Location, Position, Range, Url,
};
use parking_lot::RwLock;
use pkg::{
    manifest::{GenericManifestFile, ManifestFile},
    BuildPlan,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::HashMap,
    ops::Deref,
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
    time::SystemTime,
};
use sway_ast::{attribute::Annotated, ItemKind};
use sway_core::{
    decl_engine::DeclEngine,
    language::{
        lexed::LexedProgram,
        parsed::{AstNode, ParseProgram},
        ty::{self},
        HasSubmodules,
    },
    BuildTarget, Engines, LspConfig, Programs,
};
use sway_error::{error::CompileError, handler::Handler, warning::CompileWarning};
use sway_types::{ProgramId, SourceEngine, Spanned};

/// A `Session` is used to store information about a single member in a workspace.
///
/// The API provides methods for responding to LSP requests from the server.
#[derive(Debug)]
pub struct Session {
    pub build_plan_cache: BuildPlanCache,
    // Cached diagnostic results that require a lock to access. Readers will wait for writers to complete.
    pub diagnostics: Arc<RwLock<DiagnosticMap>>,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

impl Session {
    pub fn new() -> Self {
        Session {
            build_plan_cache: BuildPlanCache::default(),
            diagnostics: Arc::new(RwLock::new(DiagnosticMap::new())),
        }
    }
}

/// Clean up memory in the [TypeEngine] and [DeclEngine] for the user's workspace.
pub fn garbage_collect_program(
    engines: &mut Engines,
    sync: &SyncWorkspace,
) -> Result<(), LanguageServerError> {
    let _p = tracing::trace_span!("garbage_collect").entered();
    let path = sync.temp_dir()?;
    let program_id = { engines.se().get_program_id_from_manifest_path(&path) };
    if let Some(program_id) = program_id {
        engines.clear_program(&program_id);
    }
    Ok(())
}

/// Clean up memory in the [TypeEngine] and [DeclEngine] for the modified file.
pub fn garbage_collect_module(engines: &mut Engines, uri: &Url) -> Result<(), LanguageServerError> {
    let path = uri.to_file_path().unwrap();
    let source_id = { engines.se().get_source_id(&path) };
    engines.clear_module(&source_id);
    Ok(())
}

pub fn token_references(
    url: &Url,
    position: Position,
    token_map: &TokenMap,
    engines: &Engines,
    sync: &SyncWorkspace,
) -> Option<Vec<Location>> {
    let _p = tracing::trace_span!("token_references").entered();
    let token_references: Vec<_> = token_map
        .iter()
        .all_references_of_token(token_map.token_at_position(url, position)?.value(), engines)
        .filter_map(|item| {
            let path = item.key().path.as_ref()?;
            let uri = Url::from_file_path(path).ok()?;
            sync.to_workspace_url(uri)
                .map(|workspace_url| Location::new(workspace_url, item.key().range))
        })
        .collect();
    Some(token_references)
}

pub fn token_ranges(
    engines: &Engines,
    token_map: &TokenMap,
    url: &Url,
    position: Position,
) -> Option<Vec<Range>> {
    let _p = tracing::trace_span!("token_ranges").entered();
    let mut token_ranges: Vec<_> = token_map
        .tokens_for_file(url)
        .all_references_of_token(token_map.token_at_position(url, position)?.value(), engines)
        .map(|item| item.key().range)
        .collect();

    token_ranges.sort_by(|a, b| a.start.line.cmp(&b.start.line));
    Some(token_ranges)
}

pub fn token_definition_response(
    uri: &Url,
    position: Position,
    engines: &Engines,
    token_map: &TokenMap,
    sync: &SyncWorkspace,
) -> Option<GotoDefinitionResponse> {
    let _p = tracing::trace_span!("token_definition_response").entered();
    token_map
        .token_at_position(uri, position)
        .and_then(|item| item.value().declared_token_ident(engines))
        .and_then(|decl_ident| {
            decl_ident.path.and_then(|path| {
                // We use ok() here because we don't care about propagating the error from from_file_path
                Url::from_file_path(path).ok().and_then(|url| {
                    sync.to_workspace_url(url).map(|url| {
                        GotoDefinitionResponse::Scalar(Location::new(url, decl_ident.range))
                    })
                })
            })
        })
}

pub fn completion_items(
    uri: &Url,
    position: Position,
    trigger_char: &str,
    token_map: &TokenMap,
    engines: &Engines,
    compiled_programs: &CompiledPrograms,
) -> Option<Vec<CompletionItem>> {
    let _p = tracing::trace_span!("completion_items").entered();
    let program = compiled_programs.program_from_uri(uri, engines)?;
    let shifted_position = Position {
        line: position.line,
        character: position.character - trigger_char.len() as u32 - 1,
    };
    let t = token_map.token_at_position(uri, shifted_position)?;
    let ident_to_complete = t.key();
    let fn_tokens = token_map.tokens_at_position(engines, uri, shifted_position, Some(true));
    let fn_token = fn_tokens.first()?.value();
    if let Some(TypedAstToken::TypedFunctionDeclaration(fn_decl)) = fn_token.as_typed() {
        return Some(capabilities::completion::to_completion_items(
            &program.value().typed.as_ref().unwrap().namespace,
            engines,
            ident_to_complete,
            fn_decl,
            position,
        ));
    }
    None
}

/// Generate hierarchical document symbols for the given file.
pub fn document_symbols(
    url: &Url,
    token_map: &TokenMap,
    engines: &Engines,
    compiled_programs: &CompiledPrograms,
) -> Option<Vec<DocumentSymbol>> {
    let _p = tracing::trace_span!("document_symbols").entered();
    let path = url.to_file_path().ok()?;
    let program = compiled_programs.program_from_uri(url, engines)?;
    let typed_program = program.value().typed.as_ref().unwrap().clone();
    Some(capabilities::document_symbol::to_document_symbols(
        url,
        &path,
        &typed_program,
        engines,
        token_map,
    ))
}

/// Create a [BuildPlan] from the given [Url] appropriate for the language server.
pub fn build_plan(uri: &Url) -> Result<BuildPlan, LanguageServerError> {
    let _p = tracing::trace_span!("build_plan").entered();
    let manifest_dir = PathBuf::from(uri.path());
    let manifest =
        ManifestFile::from_dir(manifest_dir).map_err(|_| DocumentError::ManifestFileNotFound {
            dir: uri.path().into(),
        })?;
    let member_manifests =
        manifest
            .member_manifests()
            .map_err(|_| DocumentError::MemberManifestsFailed {
                dir: uri.path().into(),
            })?;
    let lock_path = manifest
        .lock_path()
        .map_err(|_| DocumentError::ManifestsLockPathFailed {
            dir: uri.path().into(),
        })?;
    // TODO: Either we want LSP to deploy a local node in the background or we want this to
    // point to Fuel operated IPFS node.
    let ipfs_node = pkg::source::IPFSNode::Local;
    pkg::BuildPlan::from_lock_and_manifests(&lock_path, &member_manifests, false, false, &ipfs_node)
        .map_err(LanguageServerError::BuildPlanFailed)
}

pub fn compile(
    build_plan: &BuildPlan,
    engines: &Engines,
    retrigger_compilation: Option<Arc<AtomicBool>>,
    lsp_mode: Option<&LspConfig>,
) -> Result<Vec<(Option<Programs>, Handler)>, LanguageServerError> {
    let _p = tracing::trace_span!("compile").entered();
    pkg::check(
        build_plan,
        BuildTarget::default(),
        true,
        lsp_mode.cloned(),
        true,
        engines,
        retrigger_compilation,
        &[],
        &[sway_features::Feature::NewEncoding],
        sway_core::DbgGeneration::None,
    )
    .map_err(LanguageServerError::FailedToCompile)
}

type CompileResults = (Vec<CompileError>, Vec<CompileWarning>);

pub fn traverse(
    member_path: PathBuf,
    results: Vec<(Option<Programs>, Handler)>,
    engines_original: Arc<RwLock<Engines>>,
    engines_clone: &Engines,
    token_map: &TokenMap,
    compiled_programs: &CompiledPrograms,
    modified_file: Option<&PathBuf>,
) -> Result<Option<CompileResults>, LanguageServerError> {
    let _p = tracing::trace_span!("traverse").entered();

    // Remove tokens for the modified file from the token map.
    if let Some(path) = modified_file {
        token_map.remove_tokens_for_file(path);
    }

    let mut diagnostics: CompileResults = (Vec::default(), Vec::default());
    for (value, handler) in results.into_iter() {
        // We can convert these destructured elements to a Vec<Diagnostic> later on.
        let current_diagnostics = handler.consume();
        diagnostics = current_diagnostics;

        let Some(Programs {
            lexed,
            parsed,
            typed,
            metrics,
        }) = value.as_ref()
        else {
            continue;
        };

        // Ensure that the typed program result is Ok before proceeding.
        // If it's an Err, it indicates a failure in generating the typed AST,
        // and we should return an error rather than panicking on unwrap.
        if typed.is_err() {
            return Err(LanguageServerError::FailedToParse);
        }

        let program_id = typed
            .as_ref()
            .unwrap() // safe to unwrap because we checked for Err above
            .namespace
            .current_package_ref()
            .program_id;
        let program_path = engines_clone
            .se()
            .get_manifest_path_from_program_id(&program_id)
            .unwrap();

        // Check if the cached AST was returned by the compiler for the users workspace.
        // If it was, then we need to use the original engines for traversal.
        //
        // This is due to the garbage collector removing types from the engines_clone
        // and they have not been re-added due to compilation being skipped.
        let engines_ref = engines_original.read();
        let engines = if program_path == member_path && metrics.reused_programs > 0 {
            &*engines_ref
        } else {
            engines_clone
        };

        // Convert the source_id to a path so we can use the manifest path to get the program_id.
        // This is used to store the metrics for the module.
        if let Some(source_id) = lexed.root.tree.value.span().source_id() {
            let path = engines.se().get_path(source_id);
            let program_id = program_id_from_path(&path, engines)?;

            if let Some(modified_file) = &modified_file {
                let modified_program_id = program_id_from_path(modified_file, engines)?;
                // We can skip traversing the programs for this iteration as they are unchanged.
                if program_id != modified_program_id {
                    // Update the metrics for the program before continuing. Otherwise we can't query if the program was reused.
                    compiled_programs.get_mut(&program_id).map(|mut item| {
                        item.value_mut().metrics = metrics.clone();
                    });
                    continue;
                }
            }
        }

        let (root_module, root) = match &typed {
            Ok(p) => (
                p.root_module.clone(),
                p.namespace.current_package_ref().clone(),
            ),
            Err(e) => {
                if let Some(root) = &e.root_module {
                    (root.deref().clone(), e.namespace.clone())
                } else {
                    return Err(LanguageServerError::FailedToParse);
                }
            }
        };

        // Create context with write guards to make readers wait until the update to token_map is complete.
        // This operation is fast because we already have the compile results.
        let ctx = ParseContext::new(token_map, engines, &root);

        // We do an extensive traversal of the users program to populate the token_map.
        // Perhaps we should do this for the workspace now as well and not just the workspace member?
        // if program_path == member_path {
        if program_path
            .to_str()
            .unwrap()
            .contains(SyncWorkspace::LSP_TEMP_PREFIX)
        {
            // First, populate our token_map with sway keywords.
            let lexed_tree = LexedTree::new(&ctx);
            lexed_tree.collect_module_kinds(lexed);
            parse_lexed_program(lexed, &ctx, modified_file, |an, _ctx| {
                lexed_tree.traverse_node(an)
            });

            // Next, populate our token_map with un-typed yet parsed ast nodes.
            let parsed_tree = ParsedTree::new(&ctx);
            parsed_tree.collect_module_spans(parsed);
            parse_ast_to_tokens(parsed, &ctx, modified_file, |an, _ctx| {
                parsed_tree.traverse_node(an)
            });

            // Finally, populate our token_map with typed ast nodes.
            let typed_tree = TypedTree::new(&ctx);
            typed_tree.collect_module_spans(&root_module);
            parse_ast_to_typed_tokens(&root_module, &ctx, modified_file, |node, _ctx| {
                typed_tree.traverse_node(node);
            });
        } else {
            // Collect tokens from dependencies and the standard library prelude.
            parse_ast_to_tokens(parsed, &ctx, modified_file, |an, ctx| {
                dependency::collect_parsed_declaration(an, ctx);
            });

            parse_ast_to_typed_tokens(&root_module, &ctx, modified_file, |node, ctx| {
                dependency::collect_typed_declaration(node, ctx);
            });
        }

        // Update the compiled program in the cache.
        let compiled_program = value.expect("value was checked above");
        if let Some(mut item) = compiled_programs.get_mut(&program_id) {
            *item.value_mut() = compiled_program;
        } else {
            compiled_programs.insert(program_id, compiled_program);
        }
    }

    Ok(Some(diagnostics))
}

/// Parses the project and returns true if the compiler diagnostics are new and should be published.
pub fn parse_project(
    uri: &Url,
    engines_clone: &Engines,
    retrigger_compilation: Option<Arc<AtomicBool>>,
    ctx: &CompilationContext,
    lsp_mode: Option<&LspConfig>,
) -> Result<(), LanguageServerError> {
    let _p = tracing::trace_span!("parse_project").entered();
    let engines_original = ctx.engines.clone();
    let session = ctx.session.as_ref().unwrap().clone();
    let sync = ctx.sync.as_ref().unwrap().clone();
    let build_plan = session
        .build_plan_cache
        .get_or_update(&sync.workspace_manifest_path(), || build_plan(uri))?;
    let token_map = ctx.token_map.clone();
    let compiled_programs = ctx.compiled_programs.as_ref().unwrap().clone();
    let runnables = ctx.runnables.as_ref().unwrap().clone();

    let results = compile(&build_plan, engines_clone, retrigger_compilation, lsp_mode)?;

    // First check if results is empty or if all program values are None,
    // indicating an error occurred in the compiler
    if results.is_empty()
        || results
            .iter()
            .all(|(programs_opt, _)| programs_opt.is_none())
    {
        return Err(LanguageServerError::ProgramsIsNone);
    }

    let path = uri.to_file_path().unwrap();
    let member_path = sync
        .member_path(uri)
        .ok_or(DirectoryError::TempMemberDirNotFound)?;

    // Next check that the member path is present in the results.
    let found_program_for_member = results.iter().any(|(programs_opt, _handler)| {
        programs_opt.as_ref().is_some_and(|programs| {
            programs
                .typed
                .as_ref()
                .ok()
                .and_then(|typed| {
                    let program_id = typed.as_ref().namespace.current_package_ref().program_id();
                    engines_clone
                        .se()
                        .get_manifest_path_from_program_id(&program_id)
                })
                .is_some_and(|program_manifest_path| program_manifest_path == *member_path)
        })
    });

    if !found_program_for_member {
        // If we don't return an error here, then we will likely crash when trying to access the Engines
        // during traversal or when creating runnables.
        return Err(LanguageServerError::MemberProgramNotFound);
    }

    // Check if we need to reprocess the project.
    let (needs_reprocessing, modified_file) =
        server_state::needs_reprocessing(&ctx.token_map, &path, lsp_mode);

    // Only traverse and create runnables if we have no tokens yet, or if a file was modified
    if needs_reprocessing {
        let diagnostics = traverse(
            member_path,
            results,
            engines_original.clone(),
            engines_clone,
            &token_map,
            &compiled_programs,
            modified_file,
        )?;

        // Write diagnostics if not optimized build
        if let Some(LspConfig {
            optimized_build: false,
            ..
        }) = &lsp_mode
        {
            if let Some((errors, warnings)) = &diagnostics {
                *session.diagnostics.write() =
                    capabilities::diagnostic::get_diagnostics(warnings, errors, engines_clone.se());
            }
        }

        if let Some(program) = compiled_programs.program_from_uri(uri, engines_clone) {
            // Check if the cached AST was returned by the compiler for the users workspace.
            // If it was, then we need to use the original engines.
            let engines = if program.value().metrics.reused_programs > 0 {
                &*engines_original.read()
            } else {
                engines_clone
            };
            create_runnables(
                &runnables,
                Some(program.value().typed.as_ref().unwrap()),
                engines.de(),
                engines.se(),
            );
        }
    }

    Ok(())
}

/// Parse the [LexedProgram] to populate the [TokenMap] with lexed nodes.
pub fn parse_lexed_program(
    lexed_program: &LexedProgram,
    ctx: &ParseContext,
    modified_file: Option<&PathBuf>,
    f: impl Fn(&Annotated<ItemKind>, &ParseContext) + Sync,
) {
    let should_process = |item: &&Annotated<ItemKind>| {
        modified_file
            .map(|path| {
                item.span()
                    .source_id()
                    .is_some_and(|id| ctx.engines.se().get_path(id) == *path)
            })
            .unwrap_or(true)
    };

    lexed_program
        .root
        .tree
        .value
        .items
        .iter()
        .chain(
            lexed_program
                .root
                .submodules_recursive()
                .flat_map(|(_, submodule)| &submodule.module.tree.value.items),
        )
        .filter(should_process)
        .collect::<Vec<_>>()
        .par_iter()
        .for_each(|item| f(item, ctx));
}

/// Parse the [ParseProgram] AST to populate the [TokenMap] with parsed AST nodes.
fn parse_ast_to_tokens(
    parse_program: &ParseProgram,
    ctx: &ParseContext,
    modified_file: Option<&PathBuf>,
    f: impl Fn(&AstNode, &ParseContext) + Sync,
) {
    let should_process = |node: &&AstNode| {
        modified_file
            .map(|path| {
                node.span
                    .source_id()
                    .is_some_and(|id| ctx.engines.se().get_path(id) == *path)
            })
            .unwrap_or(true)
    };

    parse_program
        .root
        .tree
        .root_nodes
        .iter()
        .chain(
            parse_program
                .root
                .submodules_recursive()
                .flat_map(|(_, submodule)| &submodule.module.tree.root_nodes),
        )
        .filter(should_process)
        .collect::<Vec<_>>()
        .par_iter()
        .for_each(|n| f(n, ctx));
}

/// Parse the [ty::TyProgram] AST to populate the [TokenMap] with typed AST nodes.
fn parse_ast_to_typed_tokens(
    root: &ty::TyModule,
    ctx: &ParseContext,
    modified_file: Option<&PathBuf>,
    f: impl Fn(&ty::TyAstNode, &ParseContext) + Sync,
) {
    let should_process = |node: &&ty::TyAstNode| {
        modified_file
            .map(|path| {
                node.span
                    .source_id()
                    .is_some_and(|id| ctx.engines.se().get_path(id) == *path)
            })
            .unwrap_or(true)
    };

    root.all_nodes
        .iter()
        .chain(
            root.submodules_recursive()
                .flat_map(|(_, submodule)| &submodule.module.all_nodes),
        )
        .filter(should_process)
        .collect::<Vec<_>>()
        .par_iter()
        .for_each(|n| f(n, ctx));
}

/// Create runnables for script main and all test functions.
fn create_runnables(
    runnables: &RunnableMap,
    typed_program: Option<&ty::TyProgram>,
    decl_engine: &DeclEngine,
    source_engine: &SourceEngine,
) {
    let _p = tracing::trace_span!("create_runnables").entered();
    let root_module = typed_program.map(|program| &program.root_module);

    // Use a local map to collect all runnables per path first
    let mut new_runnables: HashMap<PathBuf, Vec<Box<dyn Runnable>>> = HashMap::new();

    // Insert runnable test functions.
    for (decl, _) in root_module
        .into_iter()
        .flat_map(|x| x.test_fns_recursive(decl_engine))
    {
        // Get the span of the first attribute if it exists, otherwise use the span of the function name.
        let span = decl
            .attributes
            .first()
            .map_or_else(|| decl.name.span(), |attr| attr.span.clone());
        if let Some(source_id) = span.source_id() {
            let path = source_engine.get_path(source_id);
            let runnable = Box::new(RunnableTestFn {
                range: token::get_range_from_span(&span),
                test_name: Some(decl.name.to_string()),
            });
            new_runnables.entry(path).or_default().push(runnable);
        }
    }

    // Insert runnable main function if the program is a script.
    if let Some(ty::TyProgramKind::Script {
        ref main_function, ..
    }) = typed_program.map(|x| &x.kind)
    {
        let main_function = decl_engine.get_function(main_function);
        let span = main_function.name.span();
        if let Some(source_id) = span.source_id() {
            let path = source_engine.get_path(source_id);
            let runnable = Box::new(RunnableMainFn {
                range: token::get_range_from_span(&span),
                tree_type: sway_core::language::parsed::TreeType::Script,
            });
            new_runnables.entry(path).or_default().push(runnable);
        }
    }

    // Now overwrite each path's entry with the new complete vector
    let runnables_to_insert: Vec<_> = new_runnables.into_iter().collect();
    for (path, new_runnable_vec) in runnables_to_insert {
        runnables.insert(path, new_runnable_vec);
    }
}

/// Resolves a `ProgramId` from a given `path` using the manifest directory.
pub fn program_id_from_path(
    path: &PathBuf,
    engines: &Engines,
) -> Result<ProgramId, DirectoryError> {
    let program_id = sway_utils::find_parent_manifest_dir(path)
        .and_then(|manifest_path| {
            engines
                .se()
                .get_program_id_from_manifest_path(&manifest_path)
        })
        .ok_or_else(|| DirectoryError::ProgramIdNotFound {
            path: path.to_string_lossy().to_string(),
        })?;
    Ok(program_id)
}

/// A cache for storing and retrieving BuildPlan objects.
#[derive(Debug, Clone)]
pub struct BuildPlanCache {
    /// The cached BuildPlan and its last update time
    cache: Arc<RwLock<Option<(BuildPlan, SystemTime)>>>,
}

impl Default for BuildPlanCache {
    fn default() -> Self {
        Self {
            cache: Arc::new(RwLock::new(None)),
        }
    }
}

impl BuildPlanCache {
    /// Retrieves a BuildPlan from the cache or updates it if necessary.
    pub fn get_or_update<F>(
        &self,
        manifest_path: &Option<PathBuf>,
        update_fn: F,
    ) -> Result<BuildPlan, LanguageServerError>
    where
        F: FnOnce() -> Result<BuildPlan, LanguageServerError>,
    {
        let should_update = {
            let cache = self.cache.read();
            manifest_path
                .as_ref()
                .and_then(|path| path.metadata().ok()?.modified().ok())
                .map_or(cache.is_none(), |time| {
                    cache.as_ref().is_none_or(|&(_, last)| time > last)
                })
        };

        if should_update {
            let new_plan = update_fn()?;
            let mut cache = self.cache.write();
            *cache = Some((new_plan.clone(), SystemTime::now()));
            Ok(new_plan)
        } else {
            let cache = self.cache.read();
            cache
                .as_ref()
                .map(|(plan, _)| plan.clone())
                .ok_or(LanguageServerError::BuildPlanCacheIsEmpty)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GarbageCollectionConfig;
    use sway_lsp_test_utils::{get_absolute_path, get_url};

    #[test]
    fn parse_project_returns_manifest_file_not_found() {
        let dir = get_absolute_path("sway-lsp/tests/fixtures");
        let uri = get_url(&dir);
        let engines_original = Arc::new(RwLock::new(Engines::default()));
        let engines = Engines::default();
        let session = Some(Arc::new(Session::new()));
        let sync = Some(Arc::new(SyncWorkspace::new()));
        let token_map = Arc::new(TokenMap::new());
        let ctx = CompilationContext {
            session,
            sync,
            token_map,
            engines: engines_original,
            compiled_programs: None,
            runnables: None,
            optimized_build: false,
            file_versions: Default::default(),
            uri: Some(uri.clone()),
            version: None,
            gc_options: GarbageCollectionConfig::default(),
        };
        let result = parse_project(&uri, &engines, None, &ctx, None)
            .expect_err("expected ManifestFileNotFound");
        assert!(matches!(
            result,
            LanguageServerError::DocumentError(
                DocumentError::ManifestFileNotFound { dir: test_dir }
            )
            if test_dir == dir
        ));
    }
}
