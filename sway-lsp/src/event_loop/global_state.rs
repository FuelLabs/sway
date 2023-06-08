//! The context or environment in which the language server functions. In our
//! server implementation this is know as the `WorldState`.
//!
//! Each tick provides an immutable snapshot of the state as `WorldSnapshot`.

use crate::{
    config::{Config, Warnings},
    core::session::Session,
    error::{DirectoryError, DocumentError, LanguageServerError},
    event_loop::{main_loop::Task, task_pool::TaskPool},
    utils::debug,
    utils::keyword_docs::KeywordDocs,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use dashmap::DashMap;
use forc_pkg::PackageManifestFile;
use lsp_types::Url;
use std::{path::PathBuf, sync::Arc, time::Instant};

// Enforces drop order
pub(crate) struct Handle<H, C> {
    pub(crate) handle: H,
    pub(crate) receiver: C,
}

pub(crate) type ReqHandler = fn(&mut GlobalState, lsp_server::Response);
pub(crate) type ReqQueue = lsp_server::ReqQueue<(String, Instant), ReqHandler>;

/// `GlobalState` is the primary mutable state of the language server
///
/// The most interesting components are `vfs`, which stores a consistent
/// snapshot of the file systems, and `analysis_host`, which stores our
/// incremental salsa database.
///
/// Note that this struct has more than one impl in various modules!
pub(crate) struct GlobalState {
    sender: Sender<lsp_server::Message>,
    req_queue: ReqQueue,

    pub(crate) task_pool: Handle<TaskPool<Task>, Receiver<Task>>,

    // server
    pub(crate) config: Arc<Config>,
    pub(crate) keyword_docs: Arc<KeywordDocs>,
    pub(crate) sessions: Arc<Sessions>,

    // status
    pub(crate) shutdown_requested: bool,
}

/// An immutable snapshot of the world's state at a point in time.
pub(crate) struct GlobalStateSnapshot {
    pub(crate) config: Arc<Config>,
    pub(crate) keyword_docs: Arc<KeywordDocs>,
    pub(crate) sessions: Arc<Sessions>,
}

impl std::panic::UnwindSafe for GlobalStateSnapshot {}

struct Sessions(DashMap<PathBuf, Arc<Session>>);

impl Sessions {
    fn init(&self, uri: &Url) -> Result<(), LanguageServerError> {
        let session = Arc::new(Session::new());
        let project_name = session.init(uri)?;
        self.insert(project_name, session);
        Ok(())
    }

    pub(crate) fn get_uri_and_session(
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

impl GlobalState {
    pub(crate) fn new(sender: Sender<lsp_server::Message>, config: Config) -> GlobalState {
        let task_pool = {
            let (sender, receiver) = unbounded();
            let handle = TaskPool::new_with_threads(sender, config.main_loop_num_threads());
            Handle { handle, receiver }
        };
        let sessions = Arc::new(DashMap::new());
        let config = Arc::new(config);
        let keyword_docs = Arc::new(KeywordDocs::new());
        GlobalState {
            sender,
            req_queue: ReqQueue::default(),
            task_pool,
            config,
            keyword_docs,
            sessions,
            shutdown_requested: false,
        }
    }

    pub(crate) fn snapshot(&self) -> GlobalStateSnapshot {
        GlobalStateSnapshot {
            config: Arc::clone(&self.config),
            keyword_docs: Arc::clone(&self.keyword_docs),
            sessions: Arc::clone(&self.sessions),
        }
    }

    pub(crate) fn send_request<R: lsp_types::request::Request>(
        &mut self,
        params: R::Params,
        handler: ReqHandler,
    ) {
        let request = self
            .req_queue
            .outgoing
            .register(R::METHOD.to_string(), params, handler);
        self.send(request.into());
    }

    pub(crate) fn complete_request(&mut self, response: lsp_server::Response) {
        let handler = self
            .req_queue
            .outgoing
            .complete(response.id.clone())
            .expect("received response for unknown request");
        handler(self, response)
    }

    pub(crate) fn send_notification<N: lsp_types::notification::Notification>(
        &self,
        params: N::Params,
    ) {
        let not = lsp_server::Notification::new(N::METHOD.to_string(), params);
        self.send(not.into());
    }

    pub(crate) fn register_request(
        &mut self,
        request: &lsp_server::Request,
        request_received: Instant,
    ) {
        self.req_queue.incoming.register(
            request.id.clone(),
            (request.method.clone(), request_received),
        );
    }

    pub(crate) fn respond(&mut self, response: lsp_server::Response) {
        if let Some((method, start)) = self.req_queue.incoming.complete(response.id.clone()) {
            if let Some(err) = &response.error {
                if err.message.starts_with("server panicked") {
                    tracing::error!("{}, check the log", err.message);
                }
            }

            let duration = start.elapsed();
            tracing::debug!(
                "handled {} - ({}) in {:0.2?}",
                method,
                response.id,
                duration
            );
            self.send(response.into());
        }
    }

    pub(crate) fn cancel(&mut self, request_id: lsp_server::RequestId) {
        if let Some(response) = self.req_queue.incoming.cancel(request_id) {
            self.send(response.into());
        }
    }

    pub(crate) fn is_completed(&self, request: &lsp_server::Request) -> bool {
        self.req_queue.incoming.is_completed(&request.id)
    }

    fn send(&self, message: lsp_server::Message) {
        self.sender.send(message).unwrap()
    }
}

impl GlobalState {
    fn publish_diagnostics(&self, uri: &Url, workspace_uri: &Url, session: Arc<Session>) {
        let diagnostics_res = {
            let mut diagnostics_to_publish = vec![];
            let tokens = session.token_map().tokens_for_file(uri);
            match &self.config.debug.show_collected_tokens_as_warnings {
                // If collected_tokens_as_warnings is Parsed or Typed,
                // take over the normal error and warning display behavior
                // and instead show the either the parsed or typed tokens as warnings.
                // This is useful for debugging the lsp parser.
                Warnings::Parsed => diagnostics_to_publish
                    .extend(debug::generate_warnings_for_parsed_tokens(tokens)),
                Warnings::Typed => {
                    diagnostics_to_publish.extend(debug::generate_warnings_for_typed_tokens(tokens))
                }
                Warnings::Default => {}
            }
            let diagnostics = session.wait_for_parsing();
            if self.config.diagnostic.show_warnings {
                diagnostics_to_publish.extend(diagnostics.warnings);
            }
            if self.config.diagnostic.show_errors {
                diagnostics_to_publish.extend(diagnostics.errors);
            }
            diagnostics_to_publish
        };

        // Note: Even if the computed diagnostics vec is empty, we still have to push the empty Vec
        // in order to clear former diagnostics. Newly pushed diagnostics always replace previously pushed diagnostics.
        self.client
            .publish_diagnostics(workspace_uri.clone(), diagnostics_res, None)
            .await;
    }

    pub(crate) fn parse_project(&self, uri: Url, workspace_uri: Url, session: Arc<Session>) {
        let should_publish = run_blocking_parse_project(uri.clone(), session.clone()).await;
        if should_publish {
            self.publish_diagnostics(&uri, &workspace_uri, session);
        }
    }
}

/// Runs parse_project in a blocking thread, because parsing is not async.
async fn run_blocking_parse_project(uri: Url, session: Arc<Session>) -> bool {
    task::spawn_blocking(move || match session.parse_project(&uri) {
        Ok(should_publish) => should_publish,
        Err(err) => {
            tracing::error!("{}", err);
            matches!(err, LanguageServerError::FailedToParse)
        }
    })
    .await
    .unwrap_or_default()
}
