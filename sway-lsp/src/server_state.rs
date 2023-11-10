//! The context or environment in which the language server functions.

use crate::{
    capabilities::diagnostic::get_diagnostics,
    config::{Config, Warnings},
    core::session::{self, Session},
    error::{DirectoryError, DocumentError, LanguageServerError},
    utils::debug,
    utils::keyword_docs::KeywordDocs,
};
use dashmap::DashMap;
use forc_pkg::PackageManifestFile;
use lsp_types::{Diagnostic, Url};
use parking_lot::RwLock;
use std::{path::PathBuf, sync::Arc};
use tower_lsp::{jsonrpc, Client};

/// `ServerState` is the primary mutable state of the language server
pub struct ServerState {
    pub(crate) client: Option<Client>,
    pub(crate) config: Arc<RwLock<Config>>,
    pub(crate) keyword_docs: Arc<KeywordDocs>,
    pub(crate) sessions: Arc<Sessions>,
}

impl Default for ServerState {
    fn default() -> Self {
        ServerState {
            client: None,
            config: Arc::new(RwLock::new(Default::default())),
            keyword_docs: Arc::new(KeywordDocs::new()),
            sessions: Arc::new(Sessions(DashMap::new())),
        }
    }
}

impl ServerState {
    pub fn new(client: Client) -> ServerState {
        ServerState {
            client: Some(client),
            ..Default::default()
        }
    }

    pub fn shutdown_server(&self) -> jsonrpc::Result<()> {
        tracing::info!("Shutting Down the Sway Language Server");
        let _ = self.sessions.iter().map(|item| {
            let session = item.value();
            session.shutdown();
        });
        Ok(())
    }

    pub(crate) fn diagnostics(&self, uri: &Url, session: Arc<Session>) -> Vec<Diagnostic> {
        let mut diagnostics_to_publish = vec![];
        let config = &self.config.read();
        let tokens = session.token_map().tokens_for_file(uri);
        match config.debug.show_collected_tokens_as_warnings {
            // If collected_tokens_as_warnings is Parsed or Typed,
            // take over the normal error and warning display behavior
            // and instead show the either the parsed or typed tokens as warnings.
            // This is useful for debugging the lsp parser.
            Warnings::Parsed => {
                diagnostics_to_publish = debug::generate_warnings_for_parsed_tokens(tokens)
            }
            Warnings::Typed => {
                diagnostics_to_publish = debug::generate_warnings_for_typed_tokens(tokens)
            }
            Warnings::Default => {
                let diagnostics_map = session.wait_for_parsing();
                if let Some(diagnostics) = diagnostics_map.get(&PathBuf::from(uri.path())) {
                    if config.diagnostic.show_warnings {
                        diagnostics_to_publish.extend(diagnostics.warnings.clone());
                    }
                    if config.diagnostic.show_errors {
                        diagnostics_to_publish.extend(diagnostics.errors.clone());
                    }
                }
            }
        }
        diagnostics_to_publish
    }

    pub(crate) async fn parse_project(
        &self,
        uri: Url,
        workspace_uri: Url,
        version: Option<i32>,
        session: Arc<Session>,
    ) {
        match run_blocking_parse_project(uri.clone(), version, session.clone()).await {
            Ok(_) => {
                // Note: Even if the computed diagnostics vec is empty, we still have to push the empty Vec
                // in order to clear former diagnostics. Newly pushed diagnostics always replace previously pushed diagnostics.
                if let Some(client) = self.client.as_ref() {
                    client
                        .publish_diagnostics(
                            workspace_uri.clone(),
                            self.diagnostics(&uri, session),
                            None,
                        )
                        .await;
                }
            }
            Err(err) => {
                if matches!(err, LanguageServerError::FailedToParse) {
                    tracing::error!("Error parsing project: {:?}", err);
                }
            }
        }
    }
}

/// Runs parse_project in a blocking thread, because parsing is not async.
async fn run_blocking_parse_project(
    uri: Url,
    version: Option<i32>,
    session: Arc<Session>,
) -> Result<(), LanguageServerError> {
    // Acquire a permit to parse the project. If there are none available, return false. This way,
    // we avoid publishing the same diagnostics multiple times.
    if session.parse_permits.try_acquire().is_err() {
        return Err(LanguageServerError::UnableToAcquirePermit);
    }
    tokio::task::spawn_blocking(move || {
        // Lock the diagnostics result to prevent multiple threads from parsing the project at the same time.
        let mut diagnostics = session.diagnostics.write();

        if let Some(version) = version {
            // Garbage collection is fairly expsensive so we only clear on every 10th keystroke.
            if version % 10 == 0 {
                if let Err(err) = session.garbage_collect() {
                    tracing::error!("Unable to perform garbage collection: {}", err.to_string());
                }
            }
        }
        let parse_result = session::parse_project(&uri, &session.engines.read())?;
        let (errors, warnings) = parse_result.diagnostics.clone();
        session.write_parse_result(parse_result);
        *diagnostics = get_diagnostics(&warnings, &errors, session.engines.read().se());
        Ok(())
    })
    .await
    .unwrap_or_else(|_| Err(LanguageServerError::FailedToParse))
}

/// `Sessions` is a collection of [Session]s, each of which represents a project
/// that has been opened in the users workspace.
pub(crate) struct Sessions(DashMap<PathBuf, Arc<Session>>);

impl Sessions {
    fn init(&self, uri: &Url) -> Result<(), LanguageServerError> {
        let session = Arc::new(Session::new());
        let project_name = session.init(uri)?;
        self.insert(project_name, session);
        Ok(())
    }

    /// Constructs and returns a tuple of `(Url, Arc<Session>)` from a given workspace URI.
    /// The returned URL represents the temp directory workspace.
    pub(crate) fn uri_and_session_from_workspace(
        &self,
        workspace_uri: &Url,
    ) -> Result<(Url, Arc<Session>), LanguageServerError> {
        let session = self.url_to_session(workspace_uri)?;
        let uri = session.sync.workspace_to_temp_url(workspace_uri)?;
        Ok((uri, session))
    }

    fn url_to_session(&self, uri: &Url) -> Result<Arc<Session>, LanguageServerError> {
        let path = PathBuf::from(uri.path());
        let manifest = PackageManifestFile::from_dir(&path).map_err(|_| {
            DocumentError::ManifestFileNotFound {
                dir: path.to_string_lossy().to_string(),
            }
        })?;

        // strip Forc.toml from the path to get the manifest directory
        let manifest_dir = manifest
            .path()
            .parent()
            .ok_or(DirectoryError::ManifestDirNotFound)?
            .to_path_buf();

        let session = match self.try_get(&manifest_dir).try_unwrap() {
            Some(item) => item.value().clone(),
            None => {
                // If no session can be found, then we need to call init and inserst a new session into the map
                self.init(uri)?;
                self.try_get(&manifest_dir)
                    .try_unwrap()
                    .map(|item| item.value().clone())
                    .expect("no session found even though it was just inserted into the map")
            }
        };
        Ok(session)
    }
}

impl std::ops::Deref for Sessions {
    type Target = DashMap<PathBuf, Arc<Session>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
