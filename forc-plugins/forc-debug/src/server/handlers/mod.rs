use crate::{
    error::AdapterError,
    server::{
        AdditionalData, DapServer, INSTRUCTIONS_VARIABLE_REF, REGISTERS_VARIABLE_REF, THREAD_ID,
    },
};
use dap::{
    prelude::*,
    types::{Scope, StartDebuggingRequestKind},
};
use requests::{EvaluateArguments, LaunchRequestArguments};
use std::path::PathBuf;

pub(crate) mod handle_breakpoint_locations;
pub(crate) mod handle_set_breakpoints;
pub(crate) mod handle_stack_trace;
pub(crate) mod handle_variables;

impl DapServer {
    pub(crate) fn handle_attach(&mut self) -> (Result<ResponseBody, AdapterError>, Option<i64>) {
        self.state.mode = Some(StartDebuggingRequestKind::Attach);
        self.error("This feature is not currently supported.".into());
        (Ok(ResponseBody::Attach), Some(0))
    }

    pub(crate) fn handle_initialize(
        &mut self,
    ) -> (Result<ResponseBody, AdapterError>, Option<i64>) {
        (
            Ok(ResponseBody::Initialize(types::Capabilities {
                supports_breakpoint_locations_request: Some(true),
                supports_configuration_done_request: Some(true),
                ..Default::default()
            })),
            None,
        )
    }

    pub(crate) fn handle_configuration_done(
        &mut self,
    ) -> (Result<ResponseBody, AdapterError>, Option<i64>) {
        self.state.configuration_done = true;
        (Ok(ResponseBody::ConfigurationDone), None)
    }

    pub(crate) fn handle_launch(
        &mut self,
        args: &LaunchRequestArguments,
    ) -> (Result<ResponseBody, AdapterError>, Option<i64>) {
        self.state.mode = Some(StartDebuggingRequestKind::Launch);
        if let Some(additional_data) = &args.additional_data {
            if let Ok(data) = serde_json::from_value::<AdditionalData>(additional_data.clone()) {
                self.state.program_path = PathBuf::from(data.program);
                return (Ok(ResponseBody::Launch), None);
            }
        }
        (Err(AdapterError::MissingConfiguration), Some(1))
    }

    /// Handles a `next` request. Returns true if the server should continue running.
    pub(crate) fn handle_next(&mut self) -> (Result<ResponseBody, AdapterError>, Option<i64>) {
        match self.continue_debugging_tests(true) {
            Ok(true) => (Ok(ResponseBody::Next), None),
            Ok(false) => {
                // The tests finished executing
                (Ok(ResponseBody::Next), Some(0))
            }
            Err(e) => (Err(e), Some(1)),
        }
    }

    /// Handles a `continue` request. Returns true if the server should continue running.
    pub(crate) fn handle_continue(&mut self) -> (Result<ResponseBody, AdapterError>, Option<i64>) {
        match self.continue_debugging_tests(false) {
            Ok(true) => (
                Ok(ResponseBody::Continue(responses::ContinueResponse {
                    all_threads_continued: Some(true),
                })),
                None,
            ),
            Ok(false) => (
                Ok(ResponseBody::Continue(responses::ContinueResponse {
                    all_threads_continued: Some(true),
                })),
                Some(0),
            ),
            Err(e) => (Err(e), Some(1)),
        }
    }

    pub(crate) fn handle_evaluate(
        &mut self,
        args: &EvaluateArguments,
    ) -> (Result<ResponseBody, AdapterError>, Option<i64>) {
        let result = match args.context {
            Some(types::EvaluateArgumentsContext::Variables) => args.expression.clone(),
            _ => "Evaluate expressions not supported in this context".into(),
        };
        (
            Ok(ResponseBody::Evaluate(responses::EvaluateResponse {
                result,
                ..Default::default()
            })),
            None,
        )
    }

    pub(crate) fn handle_pause(&mut self) -> (Result<ResponseBody, AdapterError>, Option<i64>) {
        // TODO: interpreter pause function
        if let Some(executor) = self.state.executor() {
            executor.interpreter.set_single_stepping(true);
        }
        (Ok(ResponseBody::Pause), None)
    }

    pub(crate) fn handle_restart(&mut self) -> (Result<ResponseBody, AdapterError>, Option<i64>) {
        self.state.reset();
        (Ok(ResponseBody::Restart), None)
    }

    pub(crate) fn handle_scopes(&mut self) -> (Result<ResponseBody, AdapterError>, Option<i64>) {
        (
            Ok(ResponseBody::Scopes(responses::ScopesResponse {
                scopes: vec![
                    Scope {
                        name: "Current VM Instruction".into(),
                        presentation_hint: Some(types::ScopePresentationhint::Registers),
                        variables_reference: INSTRUCTIONS_VARIABLE_REF,
                        ..Default::default()
                    },
                    Scope {
                        name: "Registers".into(),
                        presentation_hint: Some(types::ScopePresentationhint::Registers),
                        variables_reference: REGISTERS_VARIABLE_REF,
                        ..Default::default()
                    },
                ],
            })),
            None,
        )
    }

    pub(crate) fn handle_threads(&mut self) -> (Result<ResponseBody, AdapterError>, Option<i64>) {
        (
            Ok(ResponseBody::Threads(responses::ThreadsResponse {
                threads: vec![types::Thread {
                    id: THREAD_ID,
                    name: "main".into(),
                }],
            })),
            None,
        )
    }
}
