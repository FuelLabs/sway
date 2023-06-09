//! The main loop of `sway-lsp` responsible for dispatching LSP
//! requests/replies and notifications back to the client.
//!
//! Heavily inspired by the `main_loop` function in `rust-analyzer`.
use crate::{
    config::Config,
    event_loop::{
        dispatch::{NotificationDispatcher, RequestDispatcher},
        global_state::GlobalState,
        Result,
    },
    lsp_ext,
};
use crossbeam_channel::{select, Receiver};
use lsp_server::{Connection, Notification, Request};
use lsp_types::notification::Notification as _;
use std::{fmt, time::Instant};

pub fn run(config: Config, connection: Connection) -> Result<()> {
    tracing::info!("initial config: {:#?}", config);
    GlobalState::new(connection.sender, config).run(connection.receiver)
}

enum Event {
    Lsp(lsp_server::Message),
    Task(Task),
}

#[derive(Debug)]
pub(crate) enum Task {
    Response(lsp_server::Response),
    Retry(lsp_server::Request),
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let debug_non_verbose = |not: &Notification, f: &mut fmt::Formatter<'_>| {
            f.debug_struct("Notification")
                .field("method", &not.method)
                .finish()
        };

        match self {
            Event::Lsp(lsp_server::Message::Notification(not)) => {
                if notification_is::<lsp_types::notification::DidOpenTextDocument>(not)
                    || notification_is::<lsp_types::notification::DidChangeTextDocument>(not)
                {
                    return debug_non_verbose(not, f);
                }
            }
            Event::Task(Task::Response(resp)) => {
                return f
                    .debug_struct("Response")
                    .field("id", &resp.id)
                    .field("error", &resp.error)
                    .finish();
            }
            _ => (),
        }
        match self {
            Event::Lsp(it) => fmt::Debug::fmt(it, f),
            Event::Task(it) => fmt::Debug::fmt(it, f),
        }
    }
}

pub(crate) fn notification_is<N: lsp_types::notification::Notification>(
    notification: &Notification,
) -> bool {
    notification.method == N::METHOD
}

impl GlobalState {
    fn run(mut self, inbox: Receiver<lsp_server::Message>) -> Result<()> {
        while let Some(event) = self.next_event(&inbox) {
            if matches!(
                &event,
                Event::Lsp(lsp_server::Message::Notification(Notification { method, .. }))
                if method == lsp_types::notification::Exit::METHOD
            ) {
                return Ok(());
            }
            self.handle_event(event)?;
        }

        Err("client exited without proper shutdown sequence".into())
    }

    fn next_event(&self, inbox: &Receiver<lsp_server::Message>) -> Option<Event> {
        select! {
            recv(inbox) -> msg =>
                msg.ok().map(Event::Lsp),

            recv(self.task_pool.receiver) -> task =>
                Some(Event::Task(task.unwrap())),
        }
    }

    fn handle_event(&mut self, event: Event) -> Result<()> {
        let loop_start = Instant::now();

        let event_dbg_msg = format!("{event:?}");
        tracing::debug!("{:?} handle_event({})", loop_start, event_dbg_msg);
        if tracing::enabled!(tracing::Level::INFO) {
            let task_queue_len = self.task_pool.handle.len();
            if task_queue_len > 0 {
                tracing::info!("task queue len: {}", task_queue_len);
            }
        }

        match event {
            Event::Lsp(msg) => match msg {
                lsp_server::Message::Request(req) => self.on_new_request(loop_start, req),
                lsp_server::Message::Notification(not) => self.on_notification(not)?,
                lsp_server::Message::Response(resp) => self.complete_request(resp),
            },
            Event::Task(task) => {
                self.handle_task(task);
                // Coalesce multiple task events into one loop turn
                while let Ok(task) = self.task_pool.receiver.try_recv() {
                    self.handle_task(task);
                }
            }
        }
        Ok(())
    }

    fn handle_task(&mut self, task: Task) {
        match task {
            Task::Response(response) => self.respond(response),
            // Only retry requests that haven't been cancelled. Otherwise we do unnecessary work.
            Task::Retry(req) if !self.is_completed(&req) => self.on_request(req),
            Task::Retry(_) => (),
        }
    }

    /// Registers and handles a request. This should only be called once per incoming request.
    fn on_new_request(&mut self, request_received: Instant, req: Request) {
        self.register_request(&req, request_received);
        self.on_request(req);
    }

    /// Handles a request.
    fn on_request(&mut self, req: Request) {
        let mut dispatcher = RequestDispatcher {
            req: Some(req),
            global_state: self,
        };
        dispatcher.on_sync_mut::<lsp_types::request::Shutdown>(|s, ()| {
            s.shutdown_requested = true;
            tracing::info!("Shutting Down the Sway Language Server");

            let _ = s.sessions.iter().map(|item| {
                let session = item.value();
                session.shutdown();
            });
            Ok(())
        });

        match &mut dispatcher {
            RequestDispatcher {
                req: Some(req),
                global_state: this,
            } if this.shutdown_requested => {
                this.respond(lsp_server::Response::new_err(
                    req.id.clone(),
                    lsp_server::ErrorCode::InvalidRequest as i32,
                    "Shutdown already requested.".to_owned(),
                ));
                return;
            }
            _ => (),
        }

        use crate::handlers::request as handlers;

        dispatcher
            // Request handlers that must run on the main thread
            // because they mutate GlobalState:
            //.on_sync_mut::<lsp_ext::ReloadWorkspace>(handlers::handle_workspace_reload)
            // Request handlers which are related to the user typing
            // are run on the main thread to reduce latency:
            // .on_sync::<lsp_ext::OnEnter>(handlers::handle_on_enter)
            // We can’t run latency-sensitive request handlers which do semantic
            // analysis on the main thread because that would block other
            // requests. Instead, we run these request handlers on higher priority
            // threads in the threadpool.
            .on_latency_sensitive::<lsp_types::request::Completion>(handlers::handle_completion)
            .on_latency_sensitive::<lsp_types::request::SemanticTokensFullRequest>(
                handlers::handle_semantic_tokens_full,
            )
            // Formatting is not caused by the user typing,
            // but it does qualify as latency-sensitive
            // because a delay before formatting is applied
            // can be confusing for the user.
            .on_latency_sensitive::<lsp_types::request::Formatting>(handlers::handle_formatting)
            // All other request handlers
            .on::<lsp_types::request::HoverRequest>(handlers::handle_hover)
            .on::<lsp_types::request::PrepareRenameRequest>(handlers::handle_prepare_rename)
            .on::<lsp_types::request::Rename>(handlers::handle_rename)
            .on::<lsp_types::request::DocumentSymbolRequest>(handlers::handle_document_symbol)
            .on::<lsp_types::request::GotoDefinition>(handlers::handle_goto_definition)
            .on::<lsp_types::request::DocumentHighlightRequest>(handlers::handle_document_highlight)
            .on::<lsp_types::request::CodeLensRequest>(handlers::handle_code_lens)
            .on::<lsp_types::request::CodeActionRequest>(handlers::handle_code_action)
            .on::<lsp_ext::ShowAst>(handlers::handle_show_ast)
            .on_no_retry::<lsp_types::request::InlayHintRequest>(handlers::handle_inlay_hints)
            .finish();
    }

    /// Handles an incoming notification.
    fn on_notification(&mut self, not: Notification) -> Result<()> {
        use crate::handlers::notification as handlers;
        use lsp_types::notification as notifs;

        NotificationDispatcher {
            not: Some(not),
            global_state: self,
        }
        .on::<notifs::Cancel>(handlers::handle_cancel)?
        .on::<notifs::DidOpenTextDocument>(handlers::handle_did_open_text_document)?
        .on::<notifs::DidChangeTextDocument>(handlers::handle_did_change_text_document)?
        .on::<notifs::DidSaveTextDocument>(handlers::handle_did_save_text_document)?
        .on::<notifs::DidChangeWatchedFiles>(handlers::handle_did_change_watched_files)?
        .finish();
        Ok(())
    }
}
