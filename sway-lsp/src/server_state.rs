//! The context or environment in which the language server functions.

use crate::{
    capabilities::diagnostic::get_diagnostics,
    config::{Config, Warnings},
    core::session::{self, ParseResult, Session},
    error::{DirectoryError, DocumentError, LanguageServerError},
    utils::debug,
    utils::keyword_docs::KeywordDocs,
};
use forc_pkg::PackageManifestFile;
use lsp_types::{Diagnostic, Url};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tower_lsp::{jsonrpc, Client};

/// `ServerState` is the primary mutable state of the language server
pub struct ServerState {
    pub(crate) client: Option<Client>,
    pub(crate) config: Arc<Config>,
    pub(crate) keyword_docs: Arc<KeywordDocs>,
    pub(crate) sessions: Sessions,
}

impl Default for ServerState {
    fn default() -> Self {
        ServerState {
            client: None,
            config: Arc::new(Default::default()),
            keyword_docs: Arc::new(KeywordDocs::new()),
            sessions: Sessions(HashMap::new()),
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
        let _ = self.sessions.iter().map(|(_, session)| {
            session.shutdown();
        });
        Ok(())
    }
}

pub(crate) fn diagnostics(config: &Config, uri: &Url, session: &Session) -> Vec<Diagnostic> {
    let mut diagnostics_to_publish = vec![];
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

pub(crate) async fn publish_diagnostics(
    config: &Config,
    client: &Option<Client>,
    uri: Url,
    workspace_uri: Url,
    session: &Session,
) {
    // Note: Even if the computed diagnostics vec is empty, we still have to push the empty Vec
    // in order to clear former diagnostics. Newly pushed diagnostics always replace previously pushed diagnostics.
    if let Some(client) = client.as_ref() {
        client
            .publish_diagnostics(
                workspace_uri.clone(),
                diagnostics(config, &uri, session),
                None,
            )
            .await;
    }
}

pub(crate) async fn parse_project(
    // &mut self,
    uri: Url,
    // workspace_uri: Url,
    session: &Session,
) -> Result<ParseResult, LanguageServerError> {
    // Acquire a permit to parse the project. If there are none available, return false. This way,
    // we avoid publishing the same diagnostics multiple times.
    try_acquire_parse_permit(session)?;

    // Lock the diagnostics result to prevent multiple threads from parsing the project at the same time.
    let mut diagnostics = session.diagnostics.write();
    let parse_result = run_blocking_parse_project(uri).await?;
    let (errors, warnings) = parse_result.diagnostics.clone();
    *diagnostics = get_diagnostics(&warnings, &errors, parse_result.engines.se());
    // // Note: Even if the computed diagnostics vec is empty, we still have to push the empty Vec
    // // in order to clear former diagnostics. Newly pushed diagnostics always replace previously pushed diagnostics.
    // if let Some(client) = self.client.as_ref() {
    //     client
    //         .publish_diagnostics(workspace_uri.clone(), self.diagnostics(&uri, session), None)
    //         .await;
    // }
    Ok(parse_result)
}

fn try_acquire_parse_permit(session: &Session) -> Result<(), LanguageServerError> {
    if session.parse_permits.try_acquire().is_err() {
        return Err(LanguageServerError::UnableToAcquirePermit);
    }
    Ok(())
}

/// Runs parse_project in a blocking thread, because parsing is not async.
async fn run_blocking_parse_project(uri: Url) -> Result<ParseResult, LanguageServerError> {
    tokio::task::spawn_blocking(move || {
        let parse_result = session::parse_project(&uri)?;
        Ok(parse_result)
    })
    .await
    .unwrap_or_else(|_| Err(LanguageServerError::FailedToParse))
}

/// `Sessions` is a collection of [Session]s, each of which represents a project
/// that has been opened in the users workspace.
pub(crate) struct Sessions(HashMap<PathBuf, Session>);

impl Sessions {
    fn init(&mut self, uri: &Url) -> Result<(), LanguageServerError> {
        let mut session = Session::new();
        let project_name = session.init(uri)?;
        self.insert(project_name, session);
        Ok(())
    }

    /// Constructs and returns a tuple of `(Url, &Session)` from a given workspace URI.
    /// The returned URL represents the temp directory workspace.
    pub(crate) fn uri_and_session_from_workspace(
        &self,
        workspace_uri: &Url,
    ) -> Result<(Url, &Session), LanguageServerError> {
        let session = self.url_to_session(workspace_uri)?;
        let uri = session.sync.workspace_to_temp_url(workspace_uri)?;
        Ok((uri, session))
    }

    pub(crate) fn uri_and_mut_session_from_workspace(
        &mut self,
        workspace_uri: &Url,
    ) -> Result<(Url, &mut Session), LanguageServerError> {
        let session = self.url_to_session_mut(workspace_uri)?;
        let uri = session.sync.workspace_to_temp_url(workspace_uri)?;
        Ok((uri, session))
    }

    fn url_to_session(&self, uri: &Url) -> Result<&Session, LanguageServerError> {
        let manifest_dir = get_manifest_dir_from_uri(&uri)?;
        let session = self
            .get(&manifest_dir)
            .ok_or(LanguageServerError::SessionNotFound)?;
        Ok(session)
    }

    fn url_to_session_mut(&mut self, uri: &Url) -> Result<&mut Session, LanguageServerError> {
        let manifest_dir = get_manifest_dir_from_uri(&uri)?;
        if self.get(&manifest_dir).is_none() {
            // If no session can be found, then we need to call init and insert a new session into the map
            self.init(uri)?;
        }
        let session = self
            .get_mut(&manifest_dir)
            .expect("no session found even though it was just inserted into the map");
        Ok(session)
    }
}

impl std::ops::Deref for Sessions {
    type Target = HashMap<PathBuf, Session>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Sessions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn get_manifest_dir_from_uri(uri: &Url) -> Result<PathBuf, LanguageServerError> {
    let path = PathBuf::from(uri.path());
    let manifest =
        PackageManifestFile::from_dir(&path).map_err(|_| DocumentError::ManifestFileNotFound {
            dir: path.to_string_lossy().to_string(),
        })?;

    // Strip Forc.toml from the path to get the manifest directory
    let manifest_dir = manifest
        .path()
        .parent()
        .ok_or(DirectoryError::ManifestDirNotFound)?
        .to_path_buf();

    Ok(manifest_dir)
}
