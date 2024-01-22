mod handlers;
mod state;

use self::state::ServerState;
use crate::names::register_name;
use crate::types::DynResult;
use dap::events::OutputEventBody;
use dap::events::{ExitedEventBody, StoppedEventBody};
use dap::prelude::*;
use dap::types::{Scope, StartDebuggingRequestKind, Variable};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{
    io::{BufReader, BufWriter, Stdin, Stdout},
    path::PathBuf,
    process,
};
use thiserror::Error;

pub const THREAD_ID: i64 = 0;
pub const REGISTERS_VARIABLE_REF: i64 = 1;

#[derive(Error, Debug)]
pub(crate) enum AdapterError {
    #[error("Unhandled command")]
    UnhandledCommand { command: Command },

    #[error("Missing command")]
    MissingCommand,

    #[error("Missing configuration")]
    MissingConfiguration,

    #[error("Missing source map")]
    MissingSourceMap { pc: u64 },

    #[error("Unknown breakpoint")]
    UnknownBreakpoint,

    #[error("Build failed")]
    BuildFailed { phase: String },

    #[error("Test execution failed")]
    TestExecutionFailed {
        #[from]
        source: anyhow::Error,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdditionalData {
    name: String,
    program: String,
    request: String,
}

pub struct DapServer {
    server: Server<Stdin, Stdout>,
    state: ServerState,
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
                            if let Some(program_path) = &self.state.program_path {
                                self.state.started_debugging = true;
                                match self.handle_launch(program_path.clone()) {
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
                }
                None => return Err(Box::new(AdapterError::MissingCommand)),
            };
        }
    }

    fn handle(&mut self, req: Request) -> DynResult<Response> {
        let command = req.command.clone();

        let rsp = match command {
            Command::Attach(_) => {
                self.state.mode = Some(StartDebuggingRequestKind::Attach);
                Ok(ResponseBody::Attach)
            }
            Command::BreakpointLocations(ref args) => {
                let breakpoints = self
                    .state
                    .breakpoints
                    .iter()
                    .filter_map(|bp| {
                        if let Some(source) = &bp.source {
                            if let Some(line) = bp.line {
                                if args.source.path == source.path {
                                    return Some(types::BreakpointLocation {
                                        line,
                                        ..Default::default()
                                    });
                                }
                            }
                        }
                        None
                    })
                    .collect();

                Ok(ResponseBody::BreakpointLocations(
                    responses::BreakpointLocationsResponse { breakpoints },
                ))
            }
            Command::ConfigurationDone => {
                self.state.configuration_done = true;
                Ok(ResponseBody::ConfigurationDone)
            }
            Command::Continue(_) => {
                match self.handle_continue() {
                    Ok(true) => {}
                    Ok(false) => {
                        // The tests finished executing
                        self.exit(0);
                    }
                    Err(e) => {
                        self.error(format!("Continue error: {:?}", e));
                        self.exit(1);
                    }
                }
                Ok(ResponseBody::Continue(responses::ContinueResponse {
                    all_threads_continued: Some(true),
                }))
            }
            Command::Disconnect(_) => {
                self.exit(0);
                Ok(ResponseBody::Disconnect)
            }

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
                self.state.program_path = Some(PathBuf::from(data.program));
                Ok(ResponseBody::Launch)
            }
            Command::Next(_) => {
                match self.handle_next() {
                    Ok(true) => {}
                    Ok(false) => {
                        // The tests finished executing
                        self.exit(0);
                    }
                    Err(e) => {
                        self.error(format!("Next error: {:?}", e));
                        self.exit(1);
                    }
                }
                Ok(ResponseBody::Next)
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
                scopes: vec![Scope {
                    name: "Registers".into(),
                    presentation_hint: Some(types::ScopePresentationhint::Registers),
                    variables_reference: REGISTERS_VARIABLE_REF,
                    ..Default::default()
                }],
            })),
            Command::SetBreakpoints(ref args) => {
                let mut rng = rand::thread_rng();
                let breakpoints = args
                    .breakpoints
                    .clone()
                    .unwrap_or_default()
                    .iter()
                    .map(|source_bp| {
                        match self.state.breakpoints.iter().find(|bp| {
                            if let Some(source) = &bp.source {
                                if let Some(line) = bp.line {
                                    if args.source.path == source.path && source_bp.line == line {
                                        return true;
                                    }
                                }
                            }
                            false
                        }) {
                            Some(existing_bp) => existing_bp.clone(),

                            None => {
                                let id = rng.gen_range(0..1000000); // TODO: unique
                                types::Breakpoint {
                                    id: Some(id),
                                    verified: true,
                                    line: Some(source_bp.line),
                                    source: Some(args.source.clone()),
                                    ..Default::default()
                                }
                            }
                        }
                    })
                    .collect::<Vec<_>>();
                self.state.breakpoints = breakpoints.clone();
                Ok(ResponseBody::SetBreakpoints(
                    responses::SetBreakpointsResponse { breakpoints },
                ))
            }
            Command::StackTrace(_) => {
                let executor = self.state.executors.first().unwrap(); // TODO

                // For now, we only return 1 stack frame.
                let stack_frames = self
                    .state
                    .current_breakpoint_id
                    .and_then(|bp_id| {
                        self.state
                            .breakpoints
                            .iter()
                            .find(|bp| bp.id == Some(bp_id))
                            .map(|bp| {
                                vec![types::StackFrame {
                                    id: 0,
                                    name: executor.name.clone(),
                                    source: bp.source.clone(),
                                    line: bp.line.unwrap(),
                                    column: 0,
                                    presentation_hint: Some(
                                        types::StackFramePresentationhint::Normal,
                                    ),
                                    ..Default::default()
                                }]
                            })
                    })
                    .unwrap_or_default();

                Ok(ResponseBody::StackTrace(responses::StackTraceResponse {
                    stack_frames,
                    total_frames: None,
                }))
            }
            Command::StepIn(_) => Ok(ResponseBody::StepIn),
            Command::StepOut(_) => Ok(ResponseBody::StepOut),
            Command::Terminate(_) => {
                self.exit(0);
                Ok(ResponseBody::Terminate)
            }
            Command::TerminateThreads(_) => {
                self.exit(0);
                Ok(ResponseBody::TerminateThreads)
            }

            Command::Threads => Ok(ResponseBody::Threads(responses::ThreadsResponse {
                threads: vec![types::Thread {
                    id: THREAD_ID,
                    name: "main".into(),
                }],
            })),
            Command::Variables(_) => {
                let variables = self
                    .state
                    .executor()
                    .as_ref()
                    .map(|executor| {
                        let mut i = 0;
                        executor
                            .interpreter
                            .registers()
                            .iter()
                            .map(|value| {
                                let variable = Variable {
                                    name: register_name(i),
                                    value: format!("{:<8}", value),
                                    type_field: None,
                                    presentation_hint: None,
                                    evaluate_name: None,
                                    variables_reference: REGISTERS_VARIABLE_REF,
                                    named_variables: None,
                                    indexed_variables: None,
                                    memory_reference: None,
                                };
                                i += 1;
                                variable
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                Ok(ResponseBody::Variables(responses::VariablesResponse {
                    variables,
                }))
            }
            _ => Err(AdapterError::UnhandledCommand { command }),
        };

        match rsp {
            Ok(rsp) => Ok(req.success(rsp)),
            Err(e) => Ok(req.error(&format!("{:?}", e))),
        }
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
        self.state.current_breakpoint_id = Some(breakpoint_id);
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
            self.state.current_breakpoint_id = Some(breakpoint_id);
            Some(vec![breakpoint_id])
        } else {
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
