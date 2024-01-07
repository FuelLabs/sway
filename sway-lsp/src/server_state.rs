//! The context or environment in which the language server functions.

use crate::{
    config::{Config, Warnings},
    core::session::{self, Session},
    error::{DirectoryError, DocumentError, LanguageServerError},
    utils::debug,
    utils::keyword_docs::KeywordDocs,
};
use crossbeam_channel::{Sender, Receiver};
use dashmap::DashMap;
use forc_pkg::PackageManifestFile;
use lsp_types::{Diagnostic, Url};
use parking_lot::RwLock;
use std::{path::PathBuf, sync::{Arc, atomic::{AtomicBool, Ordering}}};
use tower_lsp::{jsonrpc, Client};

/// `ServerState` is the primary mutable state of the language server
pub struct ServerState {
    pub(crate) client: Option<Client>,
    pub(crate) config: Arc<RwLock<Config>>,
    pub(crate) keyword_docs: Arc<KeywordDocs>,
    pub(crate) sessions: Arc<Sessions>,
    pub(crate) retrigger_compilation: Arc<AtomicBool>,
    pub(crate) is_compiling: Arc<AtomicBool>,
    pub(crate) mpsc_tx: Sender<Shared>,
    pub(crate) mpsc_rx: Arc<Receiver<Shared>>,
    pub(crate) finished_compilation: Arc<tokio::sync::Notify>,
}

impl Default for ServerState {
    fn default() -> Self {
        let (mpsc_tx, mpsc_rx) = crossbeam_channel::bounded(1);

        let state = ServerState {
            client: None,
            config: Arc::new(RwLock::new(Default::default())),
            keyword_docs: Arc::new(KeywordDocs::new()),
            sessions: Arc::new(Sessions(DashMap::new())),
            retrigger_compilation: Arc::new(AtomicBool::new(false)),
            is_compiling: Arc::new(AtomicBool::new(false)),
            mpsc_tx,
            mpsc_rx: Arc::new(mpsc_rx),
            finished_compilation: Arc::new(tokio::sync::Notify::new()),
        };

        state.spawn_compilation_thread();
        state
    }
}

#[derive(Debug, Default)]
pub struct Shared {
    pub session: Option<Arc<Session>>,
    pub uri: Option<Url>,
    pub version: Option<i32>,
}

fn reset_compilation_state(
    is_compiling: Arc<AtomicBool>, 
    retrigger_compilation: Arc<AtomicBool>,
    finished_compilation: Arc<tokio::sync::Notify>
) {
    is_compiling.store(false, Ordering::Relaxed);
    retrigger_compilation.store(false, Ordering::Relaxed);
    eprintln!("THREAD | finished compilation, notifying waiters");
    finished_compilation.notify_waiters();
}

impl ServerState {
    pub fn new(client: Client) -> ServerState {
        eprintln!("ServerState::new");
        ServerState {
            client: Some(client),
            ..Default::default()
        }
    }

    pub fn spawn_compilation_thread(&self) {
        let is_compiling = self.is_compiling.clone();
        let retrigger_compilation = self.retrigger_compilation.clone();
        let finished_compilation = self.finished_compilation.clone();
        let rx = self.mpsc_rx.clone();
        std::thread::spawn(move || {
            while let Ok(shared) = rx.recv() {
                eprintln!("THREAD | received new compilation request");
                let uri = shared.uri.as_ref().unwrap().clone();
                let version = shared.version;
                let session = shared.session.as_ref().unwrap().clone();
                let mut engines_clone = session.engines.read().clone();

                // if let Some(version) = version {
                //     // Garbage collection is fairly expsensive so we only clear on every 10th keystroke.
                //     if version % 10 == 0 {
                //         // Call this on the engines clone so we don't clear types that are still in use
                //         // and might be needed in the case cancel compilation was triggered.
                //         if let Err(err) = session.garbage_collect(&mut engines_clone) {
                //             tracing::error!("Unable to perform garbage collection: {}", err.to_string());
                //         }
                //     }
                // }

                eprintln!("THREAD | starting parsing project: version: {:?}", version);
                match session::parse_project(&uri, &engines_clone, Some(retrigger_compilation.clone())) {
                    Ok(parse_result) => {
                        eprintln!("THREAD | engines_write: {:?}", version);
                        *session.engines.write() = engines_clone;
                        eprintln!("THREAD | success, about to write parse results: {:?}", version);
                        session.write_parse_result(parse_result);
                        reset_compilation_state(is_compiling.clone(), retrigger_compilation.clone(), finished_compilation.clone()); 
                    },
                    Err(err) => {
                        eprintln!("{:?}", err);
                        reset_compilation_state(is_compiling.clone(), retrigger_compilation.clone(), finished_compilation.clone());
                        continue;
                    },
                }
                eprintln!("THREAD | finished parsing project: version: {:?}", version);
            }
        });
    }

    /// Waits asynchronously for the `is_compiling` flag to become false.
    /// 
    /// This function checks the state of `is_compiling`, and if it's true,
    /// it awaits on a notification. Once notified, it checks again, repeating
    /// this process until `is_compiling` becomes false.
    pub async fn wait_for_parsing(&self) {
        loop {
            eprintln!("are we still compiling?");
            if !self.is_compiling.load(Ordering::Relaxed) {
                eprintln!("compilation is finished, lets check if there are pending compilation requests");
                if self.mpsc_rx.is_empty() {
                    eprintln!("no pending compilation work, safe to break");
                    break;
                } else {
                    eprintln!("there is pending compilation work, lets wait for it to finish");
                }
            } else {
                eprintln!("we are still compiling, lets wait to be notified");
            }
            self.finished_compilation.notified().await;
            eprintln!("we were notified, lets check if we are still compiling");
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

    pub(crate) async fn publish_diagnostics(
        &self,
        uri: Url,
        workspace_uri: Url,
        session: Arc<Session>,
    ) {
        let diagnostics = self.diagnostics(&uri, session.clone()).await;
        // Note: Even if the computed diagnostics vec is empty, we still have to push the empty Vec
        // in order to clear former diagnostics. Newly pushed diagnostics always replace previously pushed diagnostics.
        if let Some(client) = self.client.as_ref() {
            client
                .publish_diagnostics(
                    workspace_uri.clone(),
                    diagnostics,
                    None,
                )
                .await;
        }
    }

    async fn diagnostics(&self, uri: &Url, session: Arc<Session>) -> Vec<Diagnostic> {
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
                if let Some(diagnostics) = session.diagnostics.read().get(&PathBuf::from(uri.path())) {
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
}



// /// Runs parse_project in a blocking thread, because parsing is not async.
// async fn run_blocking_parse_project(
//     uri: Url,
//     version: Option<i32>,
//     session: Arc<Session>,
//     retrigger_compilation: Option<Arc<AtomicBool>>,
// ) -> Result<(), LanguageServerError> {
//     // Acquire a permit to parse the project. If there are none available, return false. This way,
//     // we avoid publishing the same diagnostics multiple times.
//     if session.parse_permits.try_acquire().is_err() {
//         return Err(LanguageServerError::UnableToAcquirePermit);
//     }
//     tokio::task::spawn_blocking(move || {
//         // Lock the diagnostics result to prevent multiple threads from parsing the project at the same time.
//         let _ = session.diagnostics.write();

//         if let Some(version) = version {
//             // Garbage collection is fairly expsensive so we only clear on every 10th keystroke.
//             if version % 10 == 0 {
//                 if let Err(err) = session.garbage_collect() {
//                     tracing::error!("Unable to perform garbage collection: {}", err.to_string());
//                 }
//             }
//         }
//         let now = std::time::Instant::now();
//         let engines_clone = session.engines.read().clone();
//         eprintln!("parse_project: engines_clone: {:?}", now.elapsed());

//         let now = std::time::Instant::now();
//         let parse_result = session::parse_project(&uri, &engines_clone, retrigger_compilation)?;
//         eprintln!("compilation_took: {:?}", now.elapsed());

//         let now = std::time::Instant::now();
//         *session.engines.write() = engines_clone;
//         eprintln!("parse_project: engines_write: {:?}", now.elapsed());

//         session.write_parse_result(parse_result);
//         Ok(())
//     })
//     .await
//     .unwrap_or_else(|_| Err(LanguageServerError::FailedToParse))
// }

/// `Sessions` is a collection of [Session]s, each of which represents a project
/// that has been opened in the users workspace.
pub(crate) struct Sessions(DashMap<PathBuf, Arc<Session>>);

impl Sessions {
    async fn init(&self, uri: &Url) -> Result<(), LanguageServerError> {
        let session = Arc::new(Session::new());
        let project_name = session.init(uri).await?;
        self.insert(project_name, session);
        Ok(())
    }

    /// Constructs and returns a tuple of `(Url, Arc<Session>)` from a given workspace URI.
    /// The returned URL represents the temp directory workspace.
    pub(crate) async fn uri_and_session_from_workspace(
        &self,
        workspace_uri: &Url,
    ) -> Result<(Url, Arc<Session>), LanguageServerError> {
        let session = self.url_to_session(workspace_uri).await?;
        let uri = session.sync.workspace_to_temp_url(workspace_uri)?;
        Ok((uri, session))
    }

    async fn url_to_session(&self, uri: &Url) -> Result<Arc<Session>, LanguageServerError> {
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
                self.init(uri).await?;
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
