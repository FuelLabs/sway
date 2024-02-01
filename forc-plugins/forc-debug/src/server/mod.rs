mod error;
mod handlers;
mod state;
mod util;

use self::error::AdapterError;
use self::state::ServerState;
use self::util::IdGenerator;
use crate::types::DynResult;
use dap::events::OutputEventBody;
use dap::events::{ExitedEventBody, StoppedEventBody};
use dap::prelude::*;
use dap::types::{Scope, StartDebuggingRequestKind};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufReader, BufWriter, Stdin, Stdout},
    path::PathBuf,
    process,
};

pub const THREAD_ID: i64 = 0;
pub const REGISTERS_VARIABLE_REF: i64 = 1;
pub const INSTRUCTIONS_VARIABLE_REF: i64 = 2;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdditionalData {
    name: String,
    program: String,
    request: String,
}

pub struct DapServer {
    server: Server<Stdin, Stdout>,
    state: ServerState,
    breakpoint_id_gen: IdGenerator,
}

impl Default for DapServer {
    fn default() -> Self {
        Self::new()
    }
}

impl DapServer {
    pub fn new() -> Self {
        let output = BufWriter::new(std::io::stdout());
        let input = BufReader::new(std::io::stdin());
        let server = Server::new(input, output);
        DapServer {
            server,
            state: Default::default(),
            breakpoint_id_gen: Default::default(),
        }
    }

    pub fn start(&mut self) -> DynResult<()> {
        loop {
            match self.server.poll_request()? {
                Some(req) => {
                    let rsp = self.handle(req)?;
                    self.server.respond(rsp)?;

                    if !self.state.initialized_event_sent {
                        let _ = self.server.send_event(Event::Initialized);
                        self.state.initialized_event_sent = true;
                    }
                    if self.state.configuration_done && !self.state.started_debugging {
                        if let Some(StartDebuggingRequestKind::Launch) = self.state.mode {
                            self.state.started_debugging = true;
                            match self.handle_launch() {
                                Ok(true) => {}
                                Ok(false) => {
                                    // The tests finished executing
                                    self.exit(0);
                                }
                                Err(e) => {
                                    self.error(format!("Launch error: {:?}", e));
                                    self.exit(1);
                                }
                            }
                        }
                    }
                }
                None => return Err(Box::new(AdapterError::MissingCommand)),
            };
        }
    }

    fn handle(&mut self, req: Request) -> DynResult<Response> {
        let command = req.command.clone();
        let mut exit_code = None;

        let rsp = match command {
            Command::Attach(_) => {
                self.state.mode = Some(StartDebuggingRequestKind::Attach);
                Ok(ResponseBody::Attach)
            }
            Command::BreakpointLocations(ref args) => {
                match self.handle_breakpoint_locations(args) {
                    Ok(breakpoints) => Ok(ResponseBody::BreakpointLocations(
                        responses::BreakpointLocationsResponse { breakpoints },
                    )),
                    Err(e) => Err(e),
                }
            }
            Command::ConfigurationDone => {
                self.state.configuration_done = true;
                Ok(ResponseBody::ConfigurationDone)
            }
            Command::Continue(_) => match self.handle_continue() {
                Ok(still_running) => {
                    if !still_running {
                        exit_code = Some(0);
                    }
                    Ok(ResponseBody::Continue(responses::ContinueResponse {
                        all_threads_continued: Some(true),
                    }))
                }
                Err(e) => {
                    exit_code = Some(1);
                    Err(e)
                }
            },
            Command::Disconnect(_) => {
                exit_code = Some(0);
                Ok(ResponseBody::Disconnect)
            }
            Command::Evaluate(_) => Ok(ResponseBody::Evaluate(responses::EvaluateResponse {
                result: "Evaluate expressions not supported".into(),
                ..Default::default()
            })),
            Command::Initialize(_) => Ok(ResponseBody::Initialize(types::Capabilities {
                supports_breakpoint_locations_request: Some(true),
                supports_configuration_done_request: Some(true),
                ..Default::default()
            })),
            Command::Launch(ref args) => {
                self.state.mode = Some(StartDebuggingRequestKind::Launch);
                let data = serde_json::from_value::<AdditionalData>(
                    args.additional_data
                        .as_ref()
                        .ok_or(AdapterError::MissingConfiguration)?
                        .clone(),
                )
                .map_err(|_| AdapterError::MissingConfiguration)?;
                self.state.program_path = PathBuf::from(data.program);
                Ok(ResponseBody::Launch)
            }
            Command::Next(_) => {
                match self.handle_next() {
                    Ok(true) => Ok(ResponseBody::Next),
                    Ok(false) => {
                        // The tests finished executing
                        exit_code = Some(0);
                        Ok(ResponseBody::Next)
                    }
                    Err(e) => {
                        exit_code = Some(1);
                        Err(e)
                    }
                }
            }
            Command::Pause(_) => {
                // TODO: interpreter pause function
                if let Some(executor) = self.state.executor() {
                    executor.interpreter.set_single_stepping(true);
                }
                Ok(ResponseBody::Pause)
            }
            Command::Restart(_) => {
                self.state.reset();
                Ok(ResponseBody::Restart)
            }
            Command::Scopes(_) => Ok(ResponseBody::Scopes(responses::ScopesResponse {
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
            Command::SetBreakpoints(ref args) => match self.handle_set_breakpoints(args) {
                Ok(breakpoints) => Ok(ResponseBody::SetBreakpoints(
                    responses::SetBreakpointsResponse { breakpoints },
                )),
                Err(e) => Err(e),
            },
            Command::StackTrace(_) => match self.handle_stack_trace() {
                Ok(stack_frames) => Ok(ResponseBody::StackTrace(responses::StackTraceResponse {
                    stack_frames,
                    total_frames: None,
                })),
                Err(e) => Err(e),
            },
            Command::StepIn(_) => Ok(ResponseBody::StepIn),
            Command::StepOut(_) => Ok(ResponseBody::StepOut),
            Command::Terminate(_) => {
                exit_code = Some(0);
                Ok(ResponseBody::Terminate)
            }
            Command::TerminateThreads(_) => {
                exit_code = Some(0);
                Ok(ResponseBody::TerminateThreads)
            }

            Command::Threads => Ok(ResponseBody::Threads(responses::ThreadsResponse {
                threads: vec![types::Thread {
                    id: THREAD_ID,
                    name: "main".into(),
                }],
            })),
            Command::Variables(ref args) => match self.handle_variables(args) {
                Ok(variables) => Ok(ResponseBody::Variables(responses::VariablesResponse {
                    variables,
                })),
                Err(e) => Err(e),
            },
            _ => Err(AdapterError::UnhandledCommand { command }),
        };

        let result = match rsp {
            Ok(rsp) => Ok(req.success(rsp)),
            Err(e) => {
                self.error(format!("{:?}", e));
                Ok(req.error(&format!("{:?}", e)))
            }
        };
        if let Some(exit_code) = exit_code {
            self.exit(exit_code)
        }
        result
    }

    /// Logs a message to the client's debugger console output.
    fn log(&mut self, output: String) {
        let _ = self.server.send_event(Event::Output(OutputEventBody {
            output,
            ..Default::default()
        }));
    }

    /// Logs an error message to the client's debugger console output.
    fn error(&mut self, output: String) {
        let _ = self.server.send_event(Event::Output(OutputEventBody {
            output,
            category: Some(types::OutputEventCategory::Stderr),
            ..Default::default()
        }));
    }

    fn log_test_results(&mut self) {
        let results = self
            .state
            .test_results
            .iter()
            .map(|result| {
                let outcome = match result.passed() {
                    true => "ok",
                    false => "failed",
                };

                format!(
                    "test {} ... {} ({}ms, {} gas)",
                    result.name,
                    outcome,
                    result.duration.as_millis(),
                    result.gas_used
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let final_outcome = match self.state.test_results.iter().any(|r| !r.passed()) {
            true => "FAILED",
            false => "OK",
        };
        let passed = self
            .state
            .test_results
            .iter()
            .filter(|r| r.passed())
            .count();
        let failed = self
            .state
            .test_results
            .iter()
            .filter(|r| !r.passed())
            .count();
        self.log(format!(
            "{}\nResult: {}. {} passed. {} failed.\n",
            results, final_outcome, passed, failed
        ));
    }

    /// Sends the 'exited' event to the client and kills the server process.
    fn exit(&mut self, exit_code: i64) {
        let _ = self
            .server
            .send_event(Event::Exited(ExitedEventBody { exit_code }));
        process::exit(exit_code as i32);
    }

    fn stop_on_breakpoint(&mut self, pc: u64) -> Result<bool, AdapterError> {
        let breakpoint_id = self.state.vm_pc_to_breakpoint_id(pc)?;
        self.state.stopped_on_breakpoint_id = Some(breakpoint_id);
        let _ = self.server.send_event(Event::Stopped(StoppedEventBody {
            reason: types::StoppedEventReason::Breakpoint,
            hit_breakpoint_ids: Some(vec![breakpoint_id]),
            description: None,
            thread_id: Some(THREAD_ID),
            preserve_focus_hint: None,
            text: None,
            all_threads_stopped: None,
        }));
        Ok(true)
    }

    fn stop_on_step(&mut self, pc: u64) -> Result<bool, AdapterError> {
        let hit_breakpoint_ids = if let Ok(breakpoint_id) = self.state.vm_pc_to_breakpoint_id(pc) {
            self.state.stopped_on_breakpoint_id = Some(breakpoint_id);
            Some(vec![breakpoint_id])
        } else {
            self.state.stopped_on_breakpoint_id = None;
            None
        };

        let _ = self.server.send_event(Event::Stopped(StoppedEventBody {
            reason: types::StoppedEventReason::Step,
            hit_breakpoint_ids,
            description: None,
            thread_id: Some(THREAD_ID),
            preserve_focus_hint: None,
            text: None,
            all_threads_stopped: None,
        }));
        Ok(true)
    }
}
