use crate::{
    capabilities::{
        self,
        diagnostic::DiagnosticMap,
        formatting::get_page_text_edit,
        runnable::{Runnable, RunnableMainFn, RunnableTestFn},
    },
    core::{
        document::TextDocument,
        sync::SyncWorkspace,
        token::{self, TypedAstToken},
        token_map::{TokenMap, TokenMapExt},
    },
    error::{DocumentError, LanguageServerError},
    traverse::{
        dependency, lexed_tree, parsed_tree::ParsedTree, typed_tree::TypedTree, ParseContext,
    },
};
use dashmap::DashMap;
use forc_pkg as pkg;
use lsp_types::{
    CompletionItem, GotoDefinitionResponse, Location, Position, Range, SymbolInformation,
    TextDocumentContentChangeEvent, TextEdit, Url,
};
use parking_lot::RwLock;
use pkg::{manifest::ManifestFile, BuildPlan};
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::{
    fs::File,
    io::Write,
    ops::Deref,
    path::PathBuf,
    sync::{atomic::Ordering, Arc},
    vec,
};
use sway_core::{
    decl_engine::DeclEngine,
    language::{
        lexed::LexedProgram,
        parsed::{AstNode, ParseProgram},
        ty::{self},
        HasSubmodules,
    },
    BuildTarget, Engines, Namespace, Programs,
};
use sway_error::{error::CompileError, handler::Handler, warning::CompileWarning};
use sway_types::{SourceEngine, SourceId, Spanned};
use sway_utils::{helpers::get_sway_files, PerformanceData};
use tokio::sync::Semaphore;

pub type Documents = DashMap<String, TextDocument>;
pub type ProjectDirectory = PathBuf;

#[derive(Default, Debug)]
pub struct CompiledProgram {
    pub lexed: Option<LexedProgram>,
    pub parsed: Option<ParseProgram>,
    pub typed: Option<ty::TyProgram>,
}

/// Used to write the result of compiling into so we can update
/// the types in [Session] after successfully parsing.
#[derive(Debug)]
pub struct ParseResult {
    pub(crate) diagnostics: (Vec<CompileError>, Vec<CompileWarning>),
    pub(crate) token_map: TokenMap,
    pub(crate) lexed: LexedProgram,
    pub(crate) parsed: ParseProgram,
    pub(crate) typed: ty::TyProgram,
    pub(crate) metrics: DashMap<SourceId, PerformanceData>,
}

/// A `Session` is used to store information about a single member in a workspace.
/// It stores the parsed and typed Tokens, as well as the [TypeEngine] associated with the project.
///
/// The API provides methods for responding to LSP requests from the server.
#[derive(Debug)]
pub struct Session {
    token_map: TokenMap,
    pub documents: Documents,
    pub runnables: DashMap<PathBuf, Vec<Box<dyn Runnable>>>,
    pub compiled_program: RwLock<CompiledProgram>,
    pub engines: RwLock<Engines>,
    pub sync: SyncWorkspace,
    // Limit the number of threads that can wait to parse at the same time. One thread can be parsing
    // and one thread can be waiting to start parsing. All others will return the cached diagnostics.
    pub parse_permits: Arc<Semaphore>,
    // Cached diagnostic results that require a lock to access. Readers will wait for writers to complete.
    pub diagnostics: Arc<RwLock<DiagnosticMap>>,
    pub metrics: DashMap<SourceId, PerformanceData>,
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
            documents: DashMap::new(),
            runnables: DashMap::new(),
            metrics: DashMap::new(),
            compiled_program: RwLock::new(Default::default()),
            engines: <_>::default(),
            sync: SyncWorkspace::new(),
            parse_permits: Arc::new(Semaphore::new(2)),
            diagnostics: Arc::new(RwLock::new(DiagnosticMap::new())),
        }
    }

    pub fn init(&self, uri: &Url) -> Result<ProjectDirectory, LanguageServerError> {
        let manifest_dir = PathBuf::from(uri.path());
        // Create a new temp dir that clones the current workspace
        // and store manifest and temp paths
        self.sync.create_temp_dir_from_workspace(&manifest_dir)?;
        self.sync.clone_manifest_dir_to_temp()?;
        // iterate over the project dir, parse all sway files
        let _ = self.store_sway_files();
        self.sync.watch_and_sync_manifest();
        self.sync.manifest_dir().map_err(Into::into)
    }

    pub fn shutdown(&self) {
        // Set the should_end flag to true
        self.sync.should_end.store(true, Ordering::Relaxed);

        // Wait for the thread to finish
        let mut join_handle_option = self.sync.notify_join_handle.write();
        if let Some(join_handle) = std::mem::take(&mut *join_handle_option) {
            let _ = join_handle.join();
        }

        // Delete the temporary directory.
        self.sync.remove_temp_dir();
    }

    /// Return a reference to the [TokenMap] of the current session.
    pub fn token_map(&self) -> &TokenMap {
        &self.token_map
    }

    /// Wait for the cached [DiagnosticMap] to be unlocked after parsing and return a copy.
    pub fn wait_for_parsing(&self) -> DiagnosticMap {
        self.diagnostics.read().clone()
    }

    /// Clean up memory in the [TypeEngine] and [DeclEngine] for the user's workspace.
    pub fn garbage_collect(&self) -> Result<(), LanguageServerError> {
        let path = self.sync.temp_dir()?;
        let module_id = { self.engines.read().se().get_module_id(&path) };
        if let Some(module_id) = module_id {
            self.engines.write().clear_module(&module_id);
        }
        Ok(())
    }

    /// Write the result of parsing to the session.
    /// This function should only be called after successfully parsing.
    pub fn write_parse_result(&self, res: ParseResult) {
        self.token_map.clear();
        self.runnables.clear();
        self.metrics.clear();

        res.token_map.deref().iter().for_each(|item| {
            let (i, t) = item.pair();
            self.token_map.insert(i.clone(), t.clone());
        });

        res.metrics.iter().for_each(|item| {
            let (s, t) = item.pair();
            self.metrics.insert(*s, t.clone());
        });

        self.create_runnables(
            &res.typed,
            self.engines.read().de(),
            self.engines.read().se(),
        );
        self.compiled_program.write().lexed = Some(res.lexed);
        self.compiled_program.write().parsed = Some(res.parsed);
        self.compiled_program.write().typed = Some(res.typed);
    }

    pub fn token_ranges(&self, url: &Url, position: Position) -> Option<Vec<Range>> {
        let (_, token) = self.token_map.token_at_position(url, position)?;
        let engines = self.engines.read();
        let mut token_ranges: Vec<_> = self
            .token_map
            .tokens_for_file(url)
            .all_references_of_token(&token, &engines)
            .map(|(ident, _)| ident.range)
            .collect();

        token_ranges.sort_by(|a, b| a.start.line.cmp(&b.start.line));
        Some(token_ranges)
    }

    pub fn token_definition_response(
        &self,
        uri: Url,
        position: Position,
    ) -> Option<GotoDefinitionResponse> {
        let engines = self.engines.read();
        self.token_map
            .token_at_position(&uri, position)
            .and_then(|(_, token)| token.declared_token_ident(&engines))
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
        let engines = self.engines.read();
        let (ident_to_complete, _) = self.token_map.token_at_position(uri, shifted_position)?;
        let fn_tokens =
            self.token_map
                .tokens_at_position(engines.se(), uri, shifted_position, Some(true));
        let (_, fn_token) = fn_tokens.first()?;
        let compiled_program = &*self.compiled_program.read();
        if let Some(TypedAstToken::TypedFunctionDeclaration(fn_decl)) = fn_token.typed.clone() {
            let program = compiled_program.typed.clone()?;
            return Some(capabilities::completion::to_completion_items(
                &program.root.namespace,
                &self.engines.read(),
                &ident_to_complete,
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
            .map(|url| capabilities::document_symbol::to_symbol_information(tokens, url))
    }

    pub fn format_text(&self, url: &Url) -> Result<Vec<TextEdit>, LanguageServerError> {
        let document = self
            .documents
            .try_get(url.path())
            .try_unwrap()
            .ok_or_else(|| DocumentError::DocumentNotFound {
                path: url.path().to_string(),
            })?;

        get_page_text_edit(Arc::from(document.get_text()), &mut <_>::default())
            .map(|page_text_edit| vec![page_text_edit])
    }

    pub fn handle_open_file(&self, uri: &Url) {
        if !self.documents.contains_key(uri.path()) {
            if let Ok(text_document) = TextDocument::build_from_path(uri.path()) {
                let _ = self.store_document(text_document);
            }
        }
    }

    /// Writes the changes to the file and updates the document.
    pub fn write_changes_to_file(
        &self,
        uri: &Url,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Result<(), LanguageServerError> {
        let src = self.update_text_document(uri, changes).ok_or_else(|| {
            DocumentError::DocumentNotFound {
                path: uri.path().to_string(),
            }
        })?;
        let mut file =
            File::create(uri.path()).map_err(|err| DocumentError::UnableToCreateFile {
                path: uri.path().to_string(),
                err: err.to_string(),
            })?;
        writeln!(&mut file, "{src}").map_err(|err| DocumentError::UnableToWriteFile {
            path: uri.path().to_string(),
            err: err.to_string(),
        })?;
        Ok(())
    }

    /// Get the document at the given [Url].
    pub fn get_text_document(&self, url: &Url) -> Result<TextDocument, DocumentError> {
        self.documents
            .try_get(url.path())
            .try_unwrap()
            .ok_or_else(|| DocumentError::DocumentNotFound {
                path: url.path().to_string(),
            })
            .map(|document| document.clone())
    }

    /// Update the document at the given [Url] with the Vec of changes returned by the client.
    pub fn update_text_document(
        &self,
        url: &Url,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Option<String> {
        self.documents
            .try_get_mut(url.path())
            .try_unwrap()
            .map(|mut document| {
                changes.iter().for_each(|change| {
                    document.apply_change(change);
                });
                document.get_text()
            })
    }

    /// Remove the text document from the session.
    pub fn remove_document(&self, url: &Url) -> Result<TextDocument, DocumentError> {
        self.documents
            .remove(url.path())
            .ok_or_else(|| DocumentError::DocumentNotFound {
                path: url.path().to_string(),
            })
            .map(|(_, text_document)| text_document)
    }

    /// Store the text document in the session.
    fn store_document(&self, text_document: TextDocument) -> Result<(), DocumentError> {
        let uri = text_document.get_uri().to_string();
        self.documents
            .insert(uri.clone(), text_document)
            .map_or(Ok(()), |_| {
                Err(DocumentError::DocumentAlreadyStored { path: uri })
            })
    }

    /// Create runnables if the `TyProgramKind` of the `TyProgram` is a script.
    fn create_runnables(
        &self,
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
                self.runnables
                    .entry(path)
                    .or_insert(Vec::new())
                    .push(runnable);
            }
        }

        // Insert runnable main function if the program is a script.
        if let ty::TyProgramKind::Script {
            ref main_function, ..
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
                self.runnables
                    .entry(path)
                    .or_insert(Vec::new())
                    .push(runnable);
            }
        }
    }

    /// Populate [Documents] with sway files found in the workspace.
    fn store_sway_files(&self) -> Result<(), LanguageServerError> {
        let temp_dir = self.sync.temp_dir()?;
        // Store the documents.
        for path in get_sway_files(temp_dir).iter().filter_map(|fp| fp.to_str()) {
            self.store_document(TextDocument::build_from_path(path)?)?;
        }
        Ok(())
    }
}

/// Create a [BuildPlan] from the given [Url] appropriate for the language server.
pub(crate) fn build_plan(uri: &Url) -> Result<BuildPlan, LanguageServerError> {
    let manifest_dir = PathBuf::from(uri.path());
    let manifest =
        ManifestFile::from_dir(&manifest_dir).map_err(|_| DocumentError::ManifestFileNotFound {
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
    pkg::BuildPlan::from_lock_and_manifests(&lock_path, &member_manifests, false, false, ipfs_node)
        .map_err(LanguageServerError::BuildPlanFailed)
}

pub fn compile(
    uri: &Url,
    engines: &Engines,
) -> Result<Vec<(Option<Programs>, Handler)>, LanguageServerError> {
    let build_plan = build_plan(uri)?;
    let tests_enabled = true;
    pkg::check(
        &build_plan,
        BuildTarget::default(),
        true,
        tests_enabled,
        engines,
    )
    .map_err(LanguageServerError::FailedToCompile)
}

pub struct TraversalResult {
    pub diagnostics: (Vec<CompileError>, Vec<CompileWarning>),
    pub programs: Option<(LexedProgram, ParseProgram, ty::TyProgram)>,
    pub token_map: TokenMap,
    pub metrics: DashMap<SourceId, PerformanceData>,
}

pub fn traverse(
    results: Vec<(Option<Programs>, Handler)>,
    engines: &Engines,
) -> Result<TraversalResult, LanguageServerError> {
    let token_map = TokenMap::new();
    let metrics_map = DashMap::new();
    let mut diagnostics = (Vec::<CompileError>::new(), Vec::<CompileWarning>::new());
    let mut programs = None;
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

        let source_id = lexed.root.tree.span().source_id().cloned();
        if let Some(source_id) = source_id {
            metrics_map.insert(source_id, metrics.clone());
        }

        // Get a reference to the typed program AST.
        let typed_program = typed
            .as_ref()
            .ok()
            .ok_or_else(|| LanguageServerError::FailedToParse)?;

        // Create context with write guards to make readers wait until the update to token_map is complete.
        // This operation is fast because we already have the compile results.
        let ctx = ParseContext::new(&token_map, engines, &typed_program.root.namespace);

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
                typed_tree.traverse_node(node)
            });

            programs = Some((lexed, parsed, typed_program.clone()));
        } else {
            // Collect tokens from dependencies and the standard library prelude.
            parse_ast_to_tokens(&parsed, &ctx, |an, ctx| {
                dependency::collect_parsed_declaration(an, ctx)
            });

            parse_ast_to_typed_tokens(typed_program, &ctx, |node, ctx| {
                dependency::collect_typed_declaration(node, ctx)
            });
        }
    }
    Ok(TraversalResult {
        diagnostics,
        programs,
        token_map,
        metrics: metrics_map,
    })
}

/// Parses the project and returns true if the compiler diagnostics are new and should be published.
pub fn parse_project(uri: &Url, engines: &Engines) -> Result<ParseResult, LanguageServerError> {
    let results = compile(uri, engines)?;
    let TraversalResult {
        diagnostics,
        programs,
        token_map,
        metrics,
    } = traverse(results, engines)?;
    let (lexed, parsed, typed) = programs.expect("Programs should be populated at this point.");
    Ok(ParseResult {
        diagnostics,
        token_map,
        lexed,
        parsed,
        typed,
        metrics,
    })
}

/// Parse the [ParseProgram] AST to populate the [TokenMap] with parsed AST nodes.
fn parse_ast_to_tokens(
    parse_program: &ParseProgram,
    ctx: &ParseContext,
    f: impl Fn(&AstNode, &ParseContext) + Sync,
) {
    let root_nodes = parse_program.root.tree.root_nodes.iter();
    let sub_nodes = parse_program
        .root
        .submodules_recursive()
        .flat_map(|(_, submodule)| &submodule.module.tree.root_nodes);

    root_nodes
        .chain(sub_nodes)
        .par_bridge()
        .for_each(|n| f(n, ctx));
}

/// Parse the [ty::TyProgram] AST to populate the [TokenMap] with typed AST nodes.
fn parse_ast_to_typed_tokens(
    typed_program: &ty::TyProgram,
    ctx: &ParseContext,
    f: impl Fn(&ty::TyAstNode, &ParseContext) + Sync,
) {
    let root_nodes = typed_program.root.all_nodes.iter();
    let sub_nodes = typed_program
        .root
        .submodules_recursive()
        .flat_map(|(_, submodule)| submodule.module.all_nodes.iter());

    root_nodes.chain(sub_nodes).for_each(|n| f(n, ctx));
}

#[cfg(test)]
mod tests {
    use super::*;
    use sway_lsp_test_utils::{get_absolute_path, get_url};

    #[test]
    fn store_document_returns_empty_tuple() {
        let session = Session::new();
        let path = get_absolute_path("sway-lsp/tests/fixtures/cats.txt");
        let document = TextDocument::build_from_path(&path).unwrap();
        let result = Session::store_document(&session, document);
        assert!(result.is_ok());
    }

    #[test]
    fn store_document_returns_document_already_stored_error() {
        let session = Session::new();
        let path = get_absolute_path("sway-lsp/tests/fixtures/cats.txt");
        let document = TextDocument::build_from_path(&path).unwrap();
        Session::store_document(&session, document).expect("expected successfully stored");
        let document = TextDocument::build_from_path(&path).unwrap();
        let result = Session::store_document(&session, document)
            .expect_err("expected DocumentAlreadyStored");
        assert_eq!(result, DocumentError::DocumentAlreadyStored { path });
    }

    #[test]
    fn parse_project_returns_manifest_file_not_found() {
        let dir = get_absolute_path("sway-lsp/tests/fixtures");
        let uri = get_url(&dir);
        let engines = Engines::default();
        let result = parse_project(&uri, &engines).expect_err("expected ManifestFileNotFound");
        assert!(matches!(
            result,
            LanguageServerError::DocumentError(
                DocumentError::ManifestFileNotFound { dir: test_dir }
            )
            if test_dir == dir
        ));
    }
}
