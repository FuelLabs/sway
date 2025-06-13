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
    server_state::{self, CompilationContext},
    traverse::{
        dependency, lexed_tree::LexedTree, parsed_tree::ParsedTree, typed_tree::TypedTree,
        ParseContext,
    },
};
use dashmap::DashMap;
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
    BuildTarget, Engines, LspConfig, Namespace, Programs,
};
use sway_error::{error::CompileError, handler::Handler, warning::CompileWarning};
use sway_types::{ProgramId, SourceEngine, Spanned};
use sway_utils::PerformanceData;

pub type RunnableMap = DashMap<PathBuf, Vec<Box<dyn Runnable>>>;
pub type ProjectDirectory = PathBuf;

#[derive(Default, Debug)]
pub struct CompiledProgram {
    pub lexed: Option<Arc<LexedProgram>>,
    pub parsed: Option<Arc<ParseProgram>>,
    pub typed: Option<Arc<ty::TyProgram>>,
}

/// A `Session` is used to store information about a single member in a workspace.
///
/// The API provides methods for responding to LSP requests from the server.
#[derive(Debug)]
pub struct Session {
    pub runnables: RunnableMap,
    pub build_plan_cache: BuildPlanCache,
    pub compiled_program: RwLock<CompiledProgram>,
    // Cached diagnostic results that require a lock to access. Readers will wait for writers to complete.
    pub diagnostics: Arc<RwLock<DiagnosticMap>>,
    pub metrics: DashMap<ProgramId, PerformanceData>,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

impl Session {
    pub fn new() -> Self {
        Session {
            runnables: DashMap::new(),
            build_plan_cache: BuildPlanCache::default(),
            metrics: DashMap::new(),
            compiled_program: RwLock::new(CompiledProgram::default()),
            diagnostics: Arc::new(RwLock::new(DiagnosticMap::new())),
        }
    }

    /// Clean up memory in the [TypeEngine] and [DeclEngine] for the user's workspace.
    pub fn garbage_collect_program(
        &self,
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
    pub fn garbage_collect_module(
        &self,
        engines: &mut Engines,
        uri: &Url,
    ) -> Result<(), LanguageServerError> {
        let path = uri.to_file_path().unwrap();
        let source_id = { engines.se().get_source_id(&path) };
        engines.clear_module(&source_id);

        Ok(())
    }

    pub fn token_references(
        &self,
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
        &self,
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
        &self,
        uri: &Url,
        position: Position,
        engines: &Engines,
        token_map: &TokenMap,
        sync: &SyncWorkspace,
    ) -> Option<GotoDefinitionResponse> {
        thread_local! {
            static RECURSION_DEPTH: std::cell::RefCell<usize> = std::cell::RefCell::new(0);
        }
        let exceeded = RECURSION_DEPTH.with(|depth| {
            let mut d = depth.borrow_mut();
            *d += 1;
            if *d > 64 {
                true
            } else {
                false
            }
        });
        if exceeded {
            eprintln!("token_definition_response recursion limit exceeded");
            return None;
        }
        let result = (|| {
            let token_at_pos = token_map.token_at_position(uri, position)?;
            let token = token_at_pos.value();
            // If this is a type parameter, check if we're inside a struct declaration and restrict lookup
            if let Some(crate::core::token::TypedAstToken::TypedParameter(_)) = token.as_typed() {
                let param_range = &token_at_pos.key().range;
                // Pass 1: Collect all struct declarations and their ranges
                let mut struct_ranges = Vec::new();
                for entry in token_map.tokens_for_file(uri) {
                    if let Some(crate::core::token::TypedAstToken::TypedDeclaration(
                        sway_core::language::ty::TyDecl::StructDecl(_),
                    )) = entry.value().as_typed() {
                        struct_ranges.push(entry.key().range);
                    }
                }
                // Pass 2: For each struct, collect its type parameters
                let mut struct_type_params: Vec<(lsp_types::Range, Vec<_>)> = Vec::new();
                for struct_range in &struct_ranges {
                    let mut params = Vec::new();
                    for entry in token_map.tokens_for_file(uri) {
                        if let Some(crate::core::token::TypedAstToken::TypedParameter(_)) = entry.value().as_typed() {
                            let param_range = &entry.key().range;
                            if param_range.start >= struct_range.start && param_range.end <= struct_range.end {
                                params.push(entry);
                            }
                        }
                    }
                    struct_type_params.push((struct_range.clone(), params));
                }
                // Find the struct whose range contains the usage
                for (struct_range, params) in &struct_type_params {
                    if struct_range.start <= param_range.start && struct_range.end >= param_range.end {
                        // Find the matching type parameter by name
                        let name = &token_at_pos.key().name;
                        for entry in params {
                            if &entry.key().name == name {
                                if let Some(decl_ident) = entry.value().declared_token_ident(engines) {
                                    if let Some(path) = decl_ident.path {
                                        if let Ok(url) = Url::from_file_path(path) {
                                            if let Some(url) = sync.to_workspace_url(url) {
                                                return Some(GotoDefinitionResponse::Scalar(Location::new(url, decl_ident.range)));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Fallback to original logic
            token.declared_token_ident(engines)
                .and_then(|decl_ident| {
                    decl_ident.path.and_then(|path| {
                        Url::from_file_path(path).ok().and_then(|url| {
                            sync.to_workspace_url(url).map(|url| {
                                GotoDefinitionResponse::Scalar(Location::new(url, decl_ident.range))
                            })
                        })
                    })
                })
        })();
        RECURSION_DEPTH.with(|depth| {
            let mut d = depth.borrow_mut();
            *d -= 1;
        });
        result
    }

    pub fn completion_items(
        &self,
        uri: &Url,
        position: Position,
        trigger_char: &str,
        token_map: &TokenMap,
        engines: &Engines,
    ) -> Option<Vec<CompletionItem>> {
        let _p = tracing::trace_span!("completion_items").entered();
        let shifted_position = Position {
            line: position.line,
            character: position.character - trigger_char.len() as u32 - 1,
        };
        let t = token_map.token_at_position(uri, shifted_position)?;
        let ident_to_complete = t.key();
        let fn_tokens = token_map.tokens_at_position(engines, uri, shifted_position, Some(true));
        let fn_token = fn_tokens.first()?.value();
        let compiled_program = &*self.compiled_program.read();
        if let Some(TypedAstToken::TypedFunctionDeclaration(fn_decl)) = fn_token.as_typed() {
            if let Some(program) = &compiled_program.typed {
                return Some(capabilities::completion::to_completion_items(
                    &program.namespace,
                    engines,
                    ident_to_complete,
                    fn_decl,
                    position,
                ));
            }
        }
        None
    }

    /// Returns the [Namespace] from the compiled program if it exists.
    pub fn namespace(&self) -> Option<Namespace> {
        let compiled_program = &*self.compiled_program.read();
        if let Some(program) = &compiled_program.typed {
            return Some(program.namespace.clone());
        }
        None
    }

    /// Generate hierarchical document symbols for the given file.
    pub fn document_symbols(
        &self,
        url: &Url,
        token_map: &TokenMap,
        engines: &Engines,
    ) -> Option<Vec<DocumentSymbol>> {
        let _p = tracing::trace_span!("document_symbols").entered();
        let path = url.to_file_path().ok()?;
        self.compiled_program
            .read()
            .typed
            .as_ref()
            .map(|ty_program| {
                capabilities::document_symbol::to_document_symbols(
                    url, &path, ty_program, engines, token_map,
                )
            })
    }
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
    session: Arc<Session>,
    token_map: &TokenMap,
    modified_file: Option<&PathBuf>,
) -> Result<Option<CompileResults>, LanguageServerError> {
    let _p = tracing::trace_span!("traverse").entered();

    // Remove tokens for the modified file from the token map.
    if let Some(path) = modified_file {
        token_map.remove_tokens_for_file(path);
    }

    session.metrics.clear();
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
            session.metrics.insert(program_id, metrics.clone());

            if let Some(modified_file) = &modified_file {
                let modified_program_id = program_id_from_path(modified_file, engines)?;
                // We can skip traversing the programs for this iteration as they are unchanged.
                if program_id != modified_program_id {
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
        if program_path == member_path {
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

            let compiled_program = &mut *session.compiled_program.write();
            compiled_program.lexed = Some(lexed.clone());
            compiled_program.parsed = Some(parsed.clone());
            compiled_program.typed = typed.as_ref().map(|x| x.clone()).ok();
        } else {
            // Collect tokens from dependencies and the standard library prelude.
            parse_ast_to_tokens(parsed, &ctx, modified_file, |an, ctx| {
                dependency::collect_parsed_declaration(an, ctx);
            });

            parse_ast_to_typed_tokens(&root_module, &ctx, modified_file, |node, ctx| {
                dependency::collect_typed_declaration(node, ctx);
            });
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
    let token_map = ctx.token_map.clone();
    let build_plan = session
        .build_plan_cache
        .get_or_update(&sync.workspace_manifest_path(), || build_plan(uri))?;

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
    let program_id = program_id_from_path(&path, engines_clone)?;
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
            session.clone(),
            &token_map,
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

        session.runnables.clear();
        if let Some(metrics) = session.metrics.get(&program_id) {
            // Check if the cached AST was returned by the compiler for the users workspace.
            // If it was, then we need to use the original engines.
            let engines = if metrics.reused_programs > 0 {
                &*engines_original.read()
            } else {
                engines_clone
            };
            let compiled_program = session.compiled_program.read();
            create_runnables(
                &session.runnables,
                compiled_program.typed.as_deref(),
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
    thread_local! {
        static PARSE_LEXED_DEPTH: std::cell::RefCell<usize> = std::cell::RefCell::new(0);
    }
    let exceeded = PARSE_LEXED_DEPTH.with(|depth| {
        let mut d = depth.borrow_mut();
        *d += 1;
        if *d > 64 {
            true
        } else {
            false
        }
    });
    if exceeded {
        eprintln!("parse_lexed_program recursion limit exceeded");
        return;
    }
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
    PARSE_LEXED_DEPTH.with(|depth| {
        let mut d = depth.borrow_mut();
        *d -= 1;
    });
}

/// Parse the [ParseProgram] AST to populate the [TokenMap] with parsed AST nodes.
fn parse_ast_to_tokens(
    parse_program: &ParseProgram,
    ctx: &ParseContext,
    modified_file: Option<&PathBuf>,
    f: impl Fn(&AstNode, &ParseContext) + Sync,
) {
    thread_local! {
        static PARSE_AST_DEPTH: std::cell::RefCell<usize> = std::cell::RefCell::new(0);
    }
    let exceeded = PARSE_AST_DEPTH.with(|depth| {
        let mut d = depth.borrow_mut();
        *d += 1;
        if *d > 64 {
            true
        } else {
            false
        }
    });
    if exceeded {
        eprintln!("parse_ast_to_tokens recursion limit exceeded");
        return;
    }
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
    PARSE_AST_DEPTH.with(|depth| {
        let mut d = depth.borrow_mut();
        *d -= 1;
    });
}

/// Parse the [ty::TyProgram] AST to populate the [TokenMap] with typed AST nodes.
fn parse_ast_to_typed_tokens(
    root: &ty::TyModule,
    ctx: &ParseContext,
    modified_file: Option<&PathBuf>,
    f: impl Fn(&ty::TyAstNode, &ParseContext) + Sync,
) {
    thread_local! {
        static PARSE_TYPED_DEPTH: std::cell::RefCell<usize> = std::cell::RefCell::new(0);
    }
    let exceeded = PARSE_TYPED_DEPTH.with(|depth| {
        let mut d = depth.borrow_mut();
        *d += 1;
        if *d > 64 {
            true
        } else {
            false
        }
    });
    if exceeded {
        eprintln!("parse_ast_to_typed_tokens recursion limit exceeded");
        return;
    }
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
    PARSE_TYPED_DEPTH.with(|depth| {
        let mut d = depth.borrow_mut();
        *d -= 1;
    });
}

/// Create runnables if the `TyProgramKind` of the `TyProgram` is a script.
fn create_runnables(
    runnables: &RunnableMap,
    typed_program: Option<&ty::TyProgram>,
    decl_engine: &DeclEngine,
    source_engine: &SourceEngine,
) {
    let root_module = typed_program.map(|program| &program.root_module);

    let _p = tracing::trace_span!("create_runnables").entered();
    // Insert runnable test functions.
    for (decl, _) in root_module
        .into_iter()
        .flat_map(|x| x.test_fns(decl_engine))
    {
        // Get the span of the first attribute if it exists, otherwise use the span of the function name.
        let span = decl
            .attributes
            .first()
            .map_or_else(|| decl.name.span(), |attr| attr.span.clone());
        if let Some(source_id) = span.source_id() {
            let path = source_engine.get_path(source_id);
            let runnable = Box::new(RunnableTestFn {
                range: token::get_range_from_span(&span.clone()),
                test_name: Some(decl.name.to_string()),
            });
            runnables.entry(path).or_default().push(runnable);
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
                range: token::get_range_from_span(&span.clone()),
                tree_type: sway_core::language::parsed::TreeType::Script,
            });
            runnables.entry(path).or_default().push(runnable);
        }
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
