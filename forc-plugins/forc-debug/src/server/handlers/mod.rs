use crate::{
    error::AdapterError,
    server::{
        AdditionalData, DapServer, HandlerResult, INSTRUCTIONS_VARIABLE_REF,
        REGISTERS_VARIABLE_REF, THREAD_ID,
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
    pub(crate) fn handle_attach(&mut self) -> HandlerResult {
        self.state.mode = Some(StartDebuggingRequestKind::Attach);
        self.error("This feature is not currently supported.".into());
        HandlerResult::ok_with_exit(ResponseBody::Attach, 0)
    }

    pub(crate) fn handle_initialize(&mut self) -> HandlerResult {
        HandlerResult::ok(ResponseBody::Initialize(types::Capabilities {
            supports_breakpoint_locations_request: Some(true),
            supports_configuration_done_request: Some(true),
            ..Default::default()
        }))
    }

    pub(crate) fn handle_configuration_done(&mut self) -> HandlerResult {
        self.state.configuration_done = true;
        HandlerResult::ok(ResponseBody::ConfigurationDone)
    }

    pub(crate) fn handle_launch(&mut self, args: &LaunchRequestArguments) -> HandlerResult {
        self.state.mode = Some(StartDebuggingRequestKind::Launch);
        if let Some(additional_data) = &args.additional_data {
            if let Ok(data) = serde_json::from_value::<AdditionalData>(additional_data.clone()) {
                self.state.program_path = PathBuf::from(data.program);
                return HandlerResult::ok(ResponseBody::Launch);
            }
        }
        HandlerResult::err_with_exit(AdapterError::MissingConfiguration, 1)
    }

    /// Handles a `next` request. Returns true if the server should continue running.
    pub(crate) fn handle_next(&mut self) -> HandlerResult {
        match self.continue_debugging_tests(true) {
            Ok(true) => HandlerResult::ok(ResponseBody::Next),
            Ok(false) => {
                // The tests finished executing
                HandlerResult::ok_with_exit(ResponseBody::Next, 0)
            }
            Err(e) => HandlerResult::err_with_exit(e, 1),
        }
    }

    /// Handles a `continue` request. Returns true if the server should continue running.
    pub(crate) fn handle_continue(&mut self) -> HandlerResult {
        match self.continue_debugging_tests(false) {
            Ok(true) => HandlerResult::ok(ResponseBody::Continue(responses::ContinueResponse {
                all_threads_continued: Some(true),
            })),
            Ok(false) => HandlerResult::ok_with_exit(
                ResponseBody::Continue(responses::ContinueResponse {
                    all_threads_continued: Some(true),
                }),
                0,
            ),
            Err(e) => HandlerResult::err_with_exit(e, 1),
        }
    }

    pub(crate) fn handle_evaluate(&mut self, args: &EvaluateArguments) -> HandlerResult {
        let result = match args.context {
            Some(types::EvaluateArgumentsContext::Variables) => args.expression.clone(),
            _ => "Evaluate expressions not supported in this context".into(),
        };
        HandlerResult::ok(ResponseBody::Evaluate(responses::EvaluateResponse {
            result,
            ..Default::default()
        }))
    }

    pub(crate) fn handle_pause(&mut self) -> HandlerResult {
        // TODO: interpreter pause function
        if let Some(executor) = self.state.executor() {
            executor.interpreter.set_single_stepping(true);
        }
        HandlerResult::ok(ResponseBody::Pause)
    }

    pub(crate) fn handle_restart(&mut self) -> HandlerResult {
        self.state.reset();
        HandlerResult::ok(ResponseBody::Restart)
    }

    pub(crate) fn handle_scopes(&mut self) -> HandlerResult {
        HandlerResult::ok(ResponseBody::Scopes(responses::ScopesResponse {
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
        }))
    }

    pub(crate) fn handle_threads(&mut self) -> HandlerResult {
        HandlerResult::ok(ResponseBody::Threads(responses::ThreadsResponse {
            threads: vec![types::Thread {
                id: THREAD_ID,
                name: "main".into(),
            }],
        }))
    }
}
