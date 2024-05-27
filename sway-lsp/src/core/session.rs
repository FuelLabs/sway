use crate::{
    capabilities::{
        self,
        diagnostic::DiagnosticMap,
        runnable::{Runnable, RunnableMainFn, RunnableTestFn},
    },
    core::{
        document::{Documents, TextDocument},
        sync::SyncWorkspace,
        token::{self, TypedAstToken},
        token_map::{TokenMap, TokenMapExt},
    },
    error::{DirectoryError, DocumentError, LanguageServerError},
    traverse::{
        dependency, lexed_tree, parsed_tree::ParsedTree, typed_tree::TypedTree, ParseContext,
    },
};
use dashmap::DashMap;
use forc_pkg as pkg;
use lsp_types::{
    CompletionItem, GotoDefinitionResponse, Location, Position, Range, SymbolInformation, Url,
};
use parking_lot::RwLock;
use pkg::{
    manifest::{GenericManifestFile, ManifestFile},
    BuildPlan,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
};
use sway_core::{
    decl_engine::DeclEngine,
    language::{
        lexed::LexedProgram,
        parsed::{AstNode, ParseProgram},
        ty, HasSubmodules,
    },
    BuildTarget, Engines, LspConfig, Namespace, Programs,
};
use sway_error::{error::CompileError, handler::Handler, warning::CompileWarning};
use sway_types::{ProgramId, SourceEngine, Spanned};
use sway_utils::{helpers::get_sway_files, PerformanceData};

pub type RunnableMap = DashMap<PathBuf, Vec<Box<dyn Runnable>>>;
pub type ProjectDirectory = PathBuf;

#[derive(Default, Debug)]
pub struct CompiledProgram {
    pub lexed: Option<LexedProgram>,
    pub parsed: Option<ParseProgram>,
    pub typed: Option<ty::TyProgram>,
}

/// A `Session` is used to store information about a single member in a workspace.
/// It stores the parsed and typed Tokens, as well as the [TypeEngine] associated with the project.
///
/// The API provides methods for responding to LSP requests from the server.
#[derive(Debug)]
pub struct Session {
    token_map: TokenMap,
    pub runnables: RunnableMap,
    pub compiled_program: RwLock<CompiledProgram>,
    pub engines: RwLock<Engines>,
    pub sync: SyncWorkspace,
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
            token_map: TokenMap::new(),
            runnables: DashMap::new(),
            metrics: DashMap::new(),
            compiled_program: RwLock::new(CompiledProgram::default()),
            engines: <_>::default(),
            sync: SyncWorkspace::new(),
            diagnostics: Arc::new(RwLock::new(DiagnosticMap::new())),
        }
    }

    pub async fn init(
        &self,
        uri: &Url,
        documents: &Documents,
    ) -> Result<ProjectDirectory, LanguageServerError> {
        let manifest_dir = PathBuf::from(uri.path());
        // Create a new temp dir that clones the current workspace
        // and store manifest and temp paths
        self.sync.create_temp_dir_from_workspace(&manifest_dir)?;
        self.sync.clone_manifest_dir_to_temp()?;
        // iterate over the project dir, parse all sway files
        let _ = self.store_sway_files(documents).await;
        self.sync.watch_and_sync_manifest();
        self.sync.manifest_dir().map_err(Into::into)
    }

    pub fn shutdown(&self) {
        // shutdown the thread watching the manifest file
        let handle = self.sync.notify_join_handle.read();
        if let Some(join_handle) = &*handle {
            join_handle.abort();
        }

        // Delete the temporary directory.
        self.sync.remove_temp_dir();
    }

    /// Return a reference to the [TokenMap] of the current session.
    pub fn token_map(&self) -> &TokenMap {
        &self.token_map
    }

    /// Clean up memory in the [TypeEngine] and [DeclEngine] for the user's workspace.
    pub fn garbage_collect(&self, engines: &mut Engines) -> Result<(), LanguageServerError> {
        let path = self.sync.temp_dir()?;
        let program_id = { engines.se().get_program_id(&path) };
        if let Some(program_id) = program_id {
            engines.clear_program(&program_id);
        }
        Ok(())
    }

    pub fn token_ranges(&self, url: &Url, position: Position) -> Option<Vec<Range>> {
        let mut token_ranges: Vec<_> = self
            .token_map
            .tokens_for_file(url)
            .all_references_of_token(
                self.token_map.token_at_position(url, position)?.value(),
                &self.engines.read(),
            )
            .map(|item| item.key().range)
            .collect();

        token_ranges.sort_by(|a, b| a.start.line.cmp(&b.start.line));
        Some(token_ranges)
    }

    pub fn token_definition_response(
        &self,
        uri: &Url,
        position: Position,
    ) -> Option<GotoDefinitionResponse> {
        self.token_map
            .token_at_position(uri, position)
            .and_then(|item| item.value().declared_token_ident(&self.engines.read()))
            .and_then(|decl_ident| {
                decl_ident.path.and_then(|path| {
                    // We use ok() here because we don't care about propagating the error from from_file_path
                    Url::from_file_path(path).ok().and_then(|url| {
                        self.sync.to_workspace_url(url).map(|url| {
                            GotoDefinitionResponse::Scalar(Location::new(url, decl_ident.range))
                        })
                    })
                })
            })
    }

    pub fn completion_items(
        &self,
        uri: &Url,
        position: Position,
        trigger_char: &str,
    ) -> Option<Vec<CompletionItem>> {
        let shifted_position = Position {
            line: position.line,
            character: position.character - trigger_char.len() as u32 - 1,
        };
        let t = self.token_map.token_at_position(uri, shifted_position)?;
        let ident_to_complete = t.key();
        let engines = self.engines.read();
        let fn_tokens =
            self.token_map
                .tokens_at_position(&engines, uri, shifted_position, Some(true));
        let fn_token = fn_tokens.first()?.value();
        let compiled_program = &*self.compiled_program.read();
        if let Some(TypedAstToken::TypedFunctionDeclaration(fn_decl)) = fn_token.typed.clone() {
            let program = compiled_program.typed.clone()?;
            let engines = self.engines.read();
            return Some(capabilities::completion::to_completion_items(
                program.root.namespace.module(&engines).current_items(),
                &engines,
                ident_to_complete,
                &fn_decl,
                position,
            ));
        }
        None
    }

    /// Returns the [Namespace] from the compiled program if it exists.
    pub fn namespace(&self) -> Option<Namespace> {
        let compiled_program = &*self.compiled_program.read();
        let program = compiled_program.typed.clone()?;
        Some(program.root.namespace)
    }

    pub fn symbol_information(&self, url: &Url) -> Option<Vec<SymbolInformation>> {
        let tokens = self.token_map.tokens_for_file(url);
        self.sync
            .to_workspace_url(url.clone())
            .map(|url| capabilities::document_symbol::to_symbol_information(tokens, &url))
    }

    /// Populate [Documents] with sway files found in the workspace.
    async fn store_sway_files(&self, documents: &Documents) -> Result<(), LanguageServerError> {
        let temp_dir = self.sync.temp_dir()?;
        // Store the documents.
        for path in get_sway_files(temp_dir).iter().filter_map(|fp| fp.to_str()) {
            documents.store_document(TextDocument::build_from_path(path).await?)?;
        }
        Ok(())
    }
}

/// Create a [BuildPlan] from the given [Url] appropriate for the language server.
pub(crate) fn build_plan(uri: &Url) -> Result<BuildPlan, LanguageServerError> {
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
    uri: &Url,
    engines: &Engines,
    retrigger_compilation: Option<Arc<AtomicBool>>,
    lsp_mode: Option<LspConfig>,
    experimental: sway_core::ExperimentalFlags,
) -> Result<Vec<(Option<Programs>, Handler)>, LanguageServerError> {
    let build_plan = build_plan(uri)?;
    let tests_enabled = true;
    pkg::check(
        &build_plan,
        BuildTarget::default(),
        true,
        lsp_mode,
        tests_enabled,
        engines,
        retrigger_compilation,
        experimental,
    )
    .map_err(LanguageServerError::FailedToCompile)
}

type CompileResults = (Vec<CompileError>, Vec<CompileWarning>);

pub fn traverse(
    results: Vec<(Option<Programs>, Handler)>,
    engines_clone: &Engines,
    session: Arc<Session>,
) -> Result<Option<CompileResults>, LanguageServerError> {
    session.token_map.clear();
    session.metrics.clear();
    let mut diagnostics: CompileResults = (Vec::default(), Vec::default());
    let results_len = results.len();
    for (i, (value, handler)) in results.into_iter().enumerate() {
        // We can convert these destructured elements to a Vec<Diagnostic> later on.
        let current_diagnostics = handler.consume();
        diagnostics = current_diagnostics;

        if value.is_none() {
            continue;
        }
        let Programs {
            lexed,
            parsed,
            typed,
            metrics,
        } = value.unwrap();

        // Check if the cached AST was returned by the compiler for the users workspace.
        // If it was, then we need to use the original engines for traversal.
        //
        // This is due to the garbage collector removing types from the engines_clone
        // and they have not been re-added due to compilation being skipped.
        let engines_ref = session.engines.read();
        let engines = if i == results_len - 1 && metrics.reused_programs > 0 {
            &*engines_ref
        } else {
            engines_clone
        };

        // Convert the source_id to a path so we can use the manifest path to get the program_id.
        // This is used to store the metrics for the module.
        if let Some(source_id) = lexed.root.tree.span().source_id() {
            let path = engines.se().get_path(source_id);
            let program_id = program_id_from_path(&path, engines)?;
            session.metrics.insert(program_id, metrics);
        }

        // Get a reference to the typed program AST.
        let typed_program = typed
            .as_ref()
            .ok()
            .ok_or_else(|| LanguageServerError::FailedToParse)?;

        // Create context with write guards to make readers wait until the update to token_map is complete.
        // This operation is fast because we already have the compile results.
        let ctx = ParseContext::new(
            &session.token_map,
            engines,
            typed_program.root.namespace.module(engines),
        );

        // The final element in the results is the main program.
        if i == results_len - 1 {
            // First, populate our token_map with sway keywords.
            lexed_tree::parse(&lexed, &ctx);

            // Next, populate our token_map with un-typed yet parsed ast nodes.
            let parsed_tree = ParsedTree::new(&ctx);
            parsed_tree.collect_module_spans(&parsed);
            parse_ast_to_tokens(&parsed, &ctx, |an, _ctx| parsed_tree.traverse_node(an));

            // Finally, populate our token_map with typed ast nodes.
            let typed_tree = TypedTree::new(&ctx);
            typed_tree.collect_module_spans(typed_program);
            parse_ast_to_typed_tokens(typed_program, &ctx, |node, _ctx| {
                typed_tree.traverse_node(node);
            });

            let compiled_program = &mut *session.compiled_program.write();
            compiled_program.lexed = Some(lexed);
            compiled_program.parsed = Some(parsed);
            compiled_program.typed = Some(typed_program.clone());
        } else {
            // Collect tokens from dependencies and the standard library prelude.
            parse_ast_to_tokens(&parsed, &ctx, |an, ctx| {
                dependency::collect_parsed_declaration(an, ctx);
            });

            parse_ast_to_typed_tokens(typed_program, &ctx, |node, ctx| {
                dependency::collect_typed_declaration(node, ctx);
            });
        }
    }
    Ok(Some(diagnostics))
}

/// Parses the project and returns true if the compiler diagnostics are new and should be published.
pub fn parse_project(
    uri: &Url,
    engines: &Engines,
    retrigger_compilation: Option<Arc<AtomicBool>>,
    lsp_mode: Option<LspConfig>,
    session: Arc<Session>,
    experimental: sway_core::ExperimentalFlags,
) -> Result<(), LanguageServerError> {
    let results = compile(
        uri,
        engines,
        retrigger_compilation,
        lsp_mode.clone(),
        experimental,
    )?;
    if results.last().is_none() {
        return Err(LanguageServerError::ProgramsIsNone);
    }
    let diagnostics = traverse(results, engines, session.clone())?;
    if let Some(config) = &lsp_mode {
        // Only write the diagnostics results on didSave or didOpen.
        if !config.optimized_build {
            if let Some((errors, warnings)) = &diagnostics {
                *session.diagnostics.write() =
                    capabilities::diagnostic::get_diagnostics(warnings, errors, engines.se());
            }
        }
    }
    if let Some(typed) = &session.compiled_program.read().typed {
        session.runnables.clear();
        create_runnables(&session.runnables, typed, engines.de(), engines.se());
    }
    Ok(())
}

/// Parse the [ParseProgram] AST to populate the [TokenMap] with parsed AST nodes.
fn parse_ast_to_tokens(
    parse_program: &ParseProgram,
    ctx: &ParseContext,
    f: impl Fn(&AstNode, &ParseContext) + Sync,
) {
    let nodes = parse_program
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
        .collect::<Vec<_>>();
    nodes.par_iter().for_each(|n| f(n, ctx));
}

/// Parse the [ty::TyProgram] AST to populate the [TokenMap] with typed AST nodes.
fn parse_ast_to_typed_tokens(
    typed_program: &ty::TyProgram,
    ctx: &ParseContext,
    f: impl Fn(&ty::TyAstNode, &ParseContext) + Sync,
) {
    let nodes = typed_program
        .root
        .all_nodes
        .iter()
        .chain(
            typed_program
                .root
                .submodules_recursive()
                .flat_map(|(_, submodule)| &submodule.module.all_nodes),
        )
        .collect::<Vec<_>>();
    nodes.par_iter().for_each(|n| f(n, ctx));
}

/// Create runnables if the `TyProgramKind` of the `TyProgram` is a script.
fn create_runnables(
    runnables: &RunnableMap,
    typed_program: &ty::TyProgram,
    decl_engine: &DeclEngine,
    source_engine: &SourceEngine,
) {
    // Insert runnable test functions.
    for (decl, _) in typed_program.test_fns(decl_engine) {
        // Get the span of the first attribute if it exists, otherwise use the span of the function name.
        let span = decl
            .attributes
            .first()
            .map_or_else(|| decl.name.span(), |(_, attr)| attr.span.clone());
        if let Some(source_id) = span.source_id() {
            let path = source_engine.get_path(source_id);
            let runnable = Box::new(RunnableTestFn {
                range: token::get_range_from_span(&span.clone()),
                tree_type: typed_program.kind.tree_type(),
                test_name: Some(decl.name.to_string()),
            });
            runnables.entry(path).or_default().push(runnable);
        }
    }

    // Insert runnable main function if the program is a script.
    if let ty::TyProgramKind::Script {
        entry_function: ref main_function,
        ..
    } = typed_program.kind
    {
        let main_function = decl_engine.get_function(main_function);
        let span = main_function.name.span();
        if let Some(source_id) = span.source_id() {
            let path = source_engine.get_path(source_id);
            let runnable = Box::new(RunnableMainFn {
                range: token::get_range_from_span(&span.clone()),
                tree_type: typed_program.kind.tree_type(),
            });
            runnables.entry(path).or_default().push(runnable);
        }
    }
}

/// Resolves a `ProgramId` from a given `path` using the manifest directory.
pub(crate) fn program_id_from_path(
    path: &PathBuf,
    engines: &Engines,
) -> Result<ProgramId, DirectoryError> {
    let program_id = sway_utils::find_parent_manifest_dir(path)
        .and_then(|manifest_path| engines.se().get_program_id(&manifest_path))
        .ok_or_else(|| DirectoryError::ProgramIdNotFound {
            path: path.to_string_lossy().to_string(),
        })?;
    Ok(program_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sway_lsp_test_utils::{get_absolute_path, get_url};

    #[test]
    fn parse_project_returns_manifest_file_not_found() {
        let dir = get_absolute_path("sway-lsp/tests/fixtures");
        let uri = get_url(&dir);
        let engines = Engines::default();
        let session = Arc::new(Session::new());
        let result = parse_project(
            &uri,
            &engines,
            None,
            None,
            session,
            sway_core::ExperimentalFlags {
                new_encoding: false,
            },
        )
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
