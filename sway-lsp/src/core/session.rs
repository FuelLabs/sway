use crate::{
    capabilities::{
        self,
        diagnostic::{get_diagnostics, Diagnostics},
        formatting::get_page_text_edit,
        runnable::{Runnable, RunnableMainFn, RunnableTestFn},
    },
    core::{
        document::TextDocument, sync::SyncWorkspace, token::get_range_from_span,
        token_map::TokenMap,
    },
    error::{DocumentError, LanguageServerError},
    traverse::{
        dependency::Dependency, lexed_tree::LexedTree, parsed_tree::ParsedTree,
        typed_tree::TypedTree,
    },
};
use dashmap::DashMap;
use forc_pkg as pkg;
use parking_lot::RwLock;
use pkg::{manifest::ManifestFile, Programs};
use std::{fs::File, io::Write, path::PathBuf, sync::Arc, vec};
use sway_core::{
    decl_engine::DeclEngine,
    language::{
        lexed::LexedProgram,
        parsed::{AstNode, ParseProgram},
        ty,
    },
    BuildTarget, CompileResult, Engines, TypeEngine,
};
use sway_types::{Span, Spanned};
use sway_utils::helpers::get_sway_files;
use tower_lsp::lsp_types::{
    CompletionItem, GotoDefinitionResponse, Location, Position, Range, SymbolInformation,
    TextDocumentContentChangeEvent, TextEdit, Url,
};

pub type Documents = DashMap<String, TextDocument>;
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
    pub documents: Documents,
    pub runnables: DashMap<Span, Box<dyn Runnable>>,
    pub compiled_program: RwLock<CompiledProgram>,
    pub type_engine: RwLock<TypeEngine>,
    pub decl_engine: RwLock<DeclEngine>,
    pub sync: SyncWorkspace,
}

impl Session {
    pub fn new() -> Self {
        Session {
            token_map: TokenMap::new(),
            documents: DashMap::new(),
            runnables: DashMap::new(),
            compiled_program: RwLock::new(Default::default()),
            type_engine: <_>::default(),
            decl_engine: <_>::default(),
            sync: SyncWorkspace::new(),
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

    pub fn parse_project(&self, uri: &Url) -> Result<Diagnostics, LanguageServerError> {
        self.token_map.clear();
        self.runnables.clear();

        let manifest_dir = PathBuf::from(uri.path());
        let locked = false;
        let offline = false;

        let manifest = ManifestFile::from_dir(&manifest_dir).map_err(|_| {
            DocumentError::ManifestFileNotFound {
                dir: uri.path().into(),
            }
        })?;

        let member_manifests =
            manifest
                .member_manifests()
                .map_err(|_| DocumentError::MemberManifestsFailed {
                    dir: uri.path().into(),
                })?;

        let lock_path =
            manifest
                .lock_path()
                .map_err(|_| DocumentError::ManifestsLockPathFailed {
                    dir: uri.path().into(),
                })?;

        let plan =
            pkg::BuildPlan::from_lock_and_manifests(&lock_path, &member_manifests, locked, offline)
                .map_err(LanguageServerError::BuildPlanFailed)?;

        let mut diagnostics = Diagnostics {
            warnings: vec![],
            errors: vec![],
        };

        *self.type_engine.write() = <_>::default();
        *self.decl_engine.write() = <_>::default();
        let type_engine = &*self.type_engine.read();
        let decl_engine = &*self.decl_engine.read();
        let engines = Engines::new(type_engine, decl_engine);
        let tests_enabled = true;
        let results = pkg::check(&plan, BuildTarget::default(), true, tests_enabled, engines)
            .map_err(LanguageServerError::FailedToCompile)?;
        let results_len = results.len();
        for (i, res) in results.into_iter().enumerate() {
            // We can convert these destructured elements to a Vec<Diagnostic> later on.
            let CompileResult {
                value,
                warnings,
                errors,
            } = res;

            if value.is_none() {
                continue;
            }
            let Programs {
                lexed,
                parsed,
                typed,
            } = value.unwrap();

            let ast_res = CompileResult::new(typed, warnings, errors);
            let typed_program = self.compile_res_to_typed_program(&ast_res)?;

            // The final element in the results is the main program.
            if i == results_len - 1 {
                // First, populate our token_map with sway keywords.
                let lexed_tree = LexedTree::new(&self.token_map);
                lexed_tree.parse(&lexed);

                // Next, populate our token_map with un-typed yet parsed ast nodes.
                let parsed_tree = ParsedTree::new(type_engine, &self.token_map);
                parsed_tree.collect_module_spans(&parsed);
                self.parse_ast_to_tokens(&parsed, |an| parsed_tree.traverse_node(an));

                // Finally, create runnables and populate our token_map with typed ast nodes.
                self.create_runnables(typed_program);

                let typed_tree =
                    TypedTree::new(engines, &self.token_map, &typed_program.root.namespace);
                typed_tree.collect_module_spans(typed_program);
                self.parse_ast_to_typed_tokens(typed_program, |node| {
                    typed_tree.traverse_node(node)
                });

                self.save_lexed_program(lexed.to_owned().clone());
                self.save_parsed_program(parsed.to_owned().clone());
                self.save_typed_program(typed_program.to_owned().clone());

                diagnostics = get_diagnostics(&ast_res.warnings, &ast_res.errors);
            } else {
                // Collect tokens from dependencies and the standard library prelude.
                let dependency = Dependency::new(&self.token_map);
                self.parse_ast_to_tokens(&parsed, |an| dependency.collect_parsed_declaration(an));

                self.parse_ast_to_typed_tokens(typed_program, |node| {
                    dependency.collect_typed_declaration(node)
                });
            }
        }
        Ok(diagnostics)
    }

    pub fn token_ranges(&self, url: &Url, position: Position) -> Option<Vec<Range>> {
        let (_, token) = self.token_map.token_at_position(url, position)?;
        let token_ranges = self
            .token_map
            .all_references_of_token(&token, &self.type_engine.read())
            .map(|(ident, _)| get_range_from_span(&ident.span()))
            .collect();

        Some(token_ranges)
    }

    pub fn token_definition_response(
        &self,
        uri: Url,
        position: Position,
    ) -> Option<GotoDefinitionResponse> {
        self.token_map
            .token_at_position(&uri, position)
            .and_then(|(_, token)| token.declared_token_ident(&self.type_engine.read()))
            .and_then(|decl_ident| {
                let range = get_range_from_span(&decl_ident.span());
                decl_ident.span().path().and_then(|path| {
                    // We use ok() here because we don't care about propagating the error from from_file_path
                    Url::from_file_path(path.as_ref()).ok().and_then(|url| {
                        self.sync
                            .to_workspace_url(url)
                            .map(|url| GotoDefinitionResponse::Scalar(Location::new(url, range)))
                    })
                })
            })
    }

    pub fn completion_items(&self) -> Option<Vec<CompletionItem>> {
        Some(capabilities::completion::to_completion_items(
            self.token_map(),
        ))
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

    /// Parse the [ParseProgram] AST to populate the [TokenMap] with parsed AST nodes.
    fn parse_ast_to_tokens(&self, parse_program: &ParseProgram, f: impl Fn(&AstNode)) {
        let root_nodes = parse_program.root.tree.root_nodes.iter();
        let sub_nodes = parse_program
            .root
            .submodules
            .iter()
            .flat_map(|(_, submodule)| &submodule.module.tree.root_nodes);

        root_nodes.chain(sub_nodes).for_each(f);
    }

    /// Parse the [ty::TyProgram] AST to populate the [TokenMap] with typed AST nodes.
    fn parse_ast_to_typed_tokens(&self, typed_program: &ty::TyProgram, f: impl Fn(&ty::TyAstNode)) {
        let root_nodes = typed_program.root.all_nodes.iter();
        let sub_nodes = typed_program
            .root
            .submodules
            .iter()
            .flat_map(|(_, submodule)| submodule.module.all_nodes.iter());

        root_nodes.chain(sub_nodes).for_each(f);
    }

    /// Get a reference to the [ty::TyProgram] AST.
    fn compile_res_to_typed_program<'a>(
        &'a self,
        ast_res: &'a CompileResult<ty::TyProgram>,
    ) -> Result<&'a ty::TyProgram, LanguageServerError> {
        ast_res
            .value
            .as_ref()
            .ok_or(LanguageServerError::FailedToParse {
                diagnostics: get_diagnostics(&ast_res.warnings, &ast_res.errors),
            })
    }

    /// Create runnables if the `TyProgramKind` of the `TyProgram` is a script.
    fn create_runnables(&self, typed_program: &ty::TyProgram) {
        // Insert runnable test functions.
        let decl_engine = &*self.decl_engine.read();
        for (decl, _) in typed_program.test_fns(decl_engine) {
            // Get the span of the first attribute if it exists, otherwise use the span of the function name.
            let span = decl
                .attributes
                .first()
                .map_or_else(|| decl.name.span(), |(_, attr)| attr.span.clone());
            let runnable = Box::new(RunnableTestFn {
                span,
                tree_type: typed_program.kind.tree_type(),
                test_name: Some(decl.name.to_string()),
            });
            self.runnables.insert(runnable.span().clone(), runnable);
        }

        // Insert runnable main function if the program is a script.
        if let ty::TyProgramKind::Script {
            ref main_function, ..
        } = typed_program.kind
        {
            let span = main_function.name.span();
            let runnable = Box::new(RunnableMainFn {
                span,
                tree_type: typed_program.kind.tree_type(),
            });
            self.runnables.insert(runnable.span().clone(), runnable);
        }
    }

    /// Save the `LexedProgram` AST in the session.
    fn save_lexed_program(&self, lexed_program: LexedProgram) {
        let mut program = self.compiled_program.write();
        program.lexed = Some(lexed_program);
    }

    /// Save the `ParseProgram` AST in the session.
    fn save_parsed_program(&self, parse_program: ParseProgram) {
        let mut program = self.compiled_program.write();
        program.parsed = Some(parse_program);
    }

    /// Save the `TyProgram` AST in the session.
    fn save_typed_program(&self, typed_program: ty::TyProgram) {
        let mut program = self.compiled_program.write();
        program.typed = Some(typed_program);
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
        let session = Session::new();
        let dir = get_absolute_path("sway-lsp/tests/fixtures");
        let uri = get_url(&dir);
        let result =
            Session::parse_project(&session, &uri).expect_err("expected ManifestFileNotFound");
        assert!(matches!(
            result,
            LanguageServerError::DocumentError(
                DocumentError::ManifestFileNotFound { dir: test_dir }
            )
            if test_dir == dir
        ));
    }
}
