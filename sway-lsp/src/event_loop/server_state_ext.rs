use crate::{
    core::session::Session,
    event_loop::{main_loop::Task, task_pool::TaskPool},
    server_state::ServerState,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use lsp_types::{Diagnostic, Url};
use std::{sync::Arc, time::Instant};

// Enforces drop order
pub(crate) struct Handle<H, C> {
    pub(crate) handle: H,
    pub(crate) receiver: C,
}

pub(crate) type ReqHandler = fn(&mut ServerStateExt, lsp_server::Response);
pub(crate) type ReqQueue = lsp_server::ReqQueue<(String, Instant), ReqHandler>;

pub(crate) struct EventLoopState {
    sender: Sender<lsp_server::Message>,
    req_queue: ReqQueue,
    pub(crate) task_pool: Handle<TaskPool<Task>, Receiver<Task>>,
    pub(crate) shutdown_requested: bool,
}

pub struct ServerStateExt {
    pub(crate) state: ServerState,
    pub(crate) event_loop_state: EventLoopState,
}

impl EventLoopState {
    pub(crate) fn new(sender: Sender<lsp_server::Message>) -> Self {
        let task_pool = {
            let (sender, receiver) = unbounded();
            let handle = TaskPool::new_with_threads(sender, main_loop_num_threads());
            Handle { handle, receiver }
        };
        Self {
            sender,
            req_queue: ReqQueue::default(),
            task_pool,
            shutdown_requested: false,
        }
    }
}

fn main_loop_num_threads() -> usize {
    num_cpus::get_physical().try_into().unwrap_or(1)
}

impl ServerStateExt {
    pub fn new(
        client: Client,
        sender: crossbeam_channel::Sender<lsp_server::Message>,
    ) -> ServerStateExt {
        let state = ServerState::new(client);
        let event_loop_state = EventLoopState::new(sender);
        ServerStateExt {
            state,
            event_loop_state,
        }
    }

    pub(crate) fn send_request<R: lsp_types::request::Request>(
        &mut self,
        params: R::Params,
        handler: ReqHandler,
    ) {
        let request = self.event_loop_state.req_queue.outgoing.register(
            R::METHOD.to_string(),
            params,
            handler,
        );
        self.send(request.into());
    }

    pub(crate) fn complete_request(&mut self, response: lsp_server::Response) {
        let handler = self
            .event_loop_state
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

    pub(crate) fn publish_diagnostics(&self, uri: Url, diagnostics: Vec<Diagnostic>) {
        self.send_notification::<lsp_types::notification::PublishDiagnostics>(
            lsp_types::PublishDiagnosticsParams {
                uri,
                diagnostics,
                version: None,
            },
        );
    }

    pub(crate) fn register_request(
        &mut self,
        request: &lsp_server::Request,
        request_received: Instant,
    ) {
        self.event_loop_state.req_queue.incoming.register(
            request.id.clone(),
            (request.method.clone(), request_received),
        );
    }

    pub(crate) fn respond(&mut self, response: lsp_server::Response) {
        if let Some((method, start)) = self
            .event_loop_state
            .req_queue
            .incoming
            .complete(response.id.clone())
        {
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
        if let Some(response) = self.event_loop_state.req_queue.incoming.cancel(request_id) {
            self.send(response.into());
        }
    }

    pub(crate) fn is_completed(&self, request: &lsp_server::Request) -> bool {
        self.event_loop_state
            .req_queue
            .incoming
            .is_completed(&request.id)
    }

    fn send(&self, message: lsp_server::Message) {
        self.event_loop_state.sender.send(message).unwrap()
    }
}
