mod error;
mod handlers;
mod state;
mod util;

use self::error::AdapterError;
use self::state::ServerState;
use self::util::IdGenerator;
use crate::types::DynResult;
use crate::types::Instruction;
use dap::events::OutputEventBody;
use dap::events::{ExitedEventBody, StoppedEventBody};
use dap::prelude::*;
use dap::types::{Scope, StartDebuggingRequestKind};
use forc_test::execute::DebugResult;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::{
    io::{BufReader, BufWriter},
    path::PathBuf,
    process,
};

pub const THREAD_ID: i64 = 0;
pub const REGISTERS_VARIABLE_REF: i64 = 1;
pub const INSTRUCTIONS_VARIABLE_REF: i64 = 2;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdditionalData {
    pub program: String,
}

/// This struct is a stateful representation of a Debug Adapter Protocol (DAP) server. It holds everything
/// needed to implement (DAP)[https://microsoft.github.io/debug-adapter-protocol/].
///
/// It is responsible for handling requests and sending responses and events to the client. It manages
/// the state of the server and the underlying VM instances used for debugging sway tests. It builds sway code
/// and generates source maps for debugging. It also manages the test setup and reports results back to the client.
pub struct DapServer {
    /// The DAP server transport.
    server: Server<Box<dyn Read>, Box<dyn Write>>,
    /// Used to generate unique breakpoint IDs.
    breakpoint_id_gen: IdGenerator,
    /// The server state.
    state: ServerState,
}

impl Default for DapServer {
    fn default() -> Self {
        Self::new(Box::new(std::io::stdin()), Box::new(std::io::stdout()))
    }
}

impl DapServer {
    pub fn new(input: Box<dyn Read>, output: Box<dyn Write>) -> Self {
        let server = Server::new(BufReader::new(input), BufWriter::new(output));
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
                    let rsp = self.handle_request(req)?;
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

    fn handle_request(&mut self, req: Request) -> DynResult<Response> {
        let command = req.command.clone();
        let (result, exit_code) = self.handle_command(command);
        let response = match result {
            Ok(rsp) => Ok(req.success(rsp)),
            Err(e) => {
                self.error(format!("{:?}", e));
                Ok(req.error(&format!("{:?}", e)))
            }
        };
        if let Some(exit_code) = exit_code {
            self.exit(exit_code)
        }
        response
    }

    /// Handles a command and returns the result and exit code, if any.
    pub fn handle_command(
        &mut self,
        command: Command,
    ) -> (Result<ResponseBody, AdapterError>, Option<i64>) {
        match command {
            Command::Attach(_) => {
                self.state.mode = Some(StartDebuggingRequestKind::Attach);
                self.error("This feature is not currently supported.".into());
                (Ok(ResponseBody::Attach), Some(0))
            }
            Command::BreakpointLocations(ref args) => {
                match self.handle_breakpoint_locations(args) {
                    Ok(breakpoints) => (
                        Ok(ResponseBody::BreakpointLocations(
                            responses::BreakpointLocationsResponse { breakpoints },
                        )),
                        None,
                    ),
                    Err(e) => (Err(e), None),
                }
            }
            Command::ConfigurationDone => {
                self.state.configuration_done = true;
                (Ok(ResponseBody::ConfigurationDone), None)
            }
            Command::Continue(_) => match self.handle_continue() {
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
            },
            Command::Disconnect(_) => (Ok(ResponseBody::Disconnect), Some(0)),
            Command::Evaluate(args) => {
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
            Command::Initialize(_) => (
                Ok(ResponseBody::Initialize(types::Capabilities {
                    supports_breakpoint_locations_request: Some(true),
                    supports_configuration_done_request: Some(true),
                    ..Default::default()
                })),
                None,
            ),
            Command::Launch(ref args) => {
                self.state.mode = Some(StartDebuggingRequestKind::Launch);
                if let Some(additional_data) = &args.additional_data {
                    if let Ok(data) =
                        serde_json::from_value::<AdditionalData>(additional_data.clone())
                    {
                        self.state.program_path = PathBuf::from(data.program);
                        return (Ok(ResponseBody::Launch), None);
                    }
                }
                (Err(AdapterError::MissingConfiguration), Some(1))
            }
            Command::Next(_) => {
                match self.handle_next() {
                    Ok(true) => (Ok(ResponseBody::Next), None),
                    Ok(false) => {
                        // The tests finished executing
                        (Ok(ResponseBody::Next), Some(0))
                    }
                    Err(e) => (Err(e), Some(1)),
                }
            }
            Command::Pause(_) => {
                // TODO: interpreter pause function
                if let Some(executor) = self.state.executor() {
                    executor.interpreter.set_single_stepping(true);
                }
                (Ok(ResponseBody::Pause), None)
            }
            Command::Restart(_) => {
                self.state.reset();
                (Ok(ResponseBody::Restart), None)
            }
            Command::Scopes(_) => (
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
            ),
            Command::SetBreakpoints(ref args) => match self.handle_set_breakpoints(args) {
                Ok(breakpoints) => (
                    Ok(ResponseBody::SetBreakpoints(
                        responses::SetBreakpointsResponse { breakpoints },
                    )),
                    None,
                ),
                Err(e) => (Err(e), None),
            },
            Command::StackTrace(_) => match self.handle_stack_trace() {
                Ok(stack_frames) => (
                    Ok(ResponseBody::StackTrace(responses::StackTraceResponse {
                        stack_frames,
                        total_frames: None,
                    })),
                    None,
                ),
                Err(e) => (Err(e), None),
            },
            Command::StepIn(_) => {
                self.error("This feature is not currently supported.".into());
                (Ok(ResponseBody::StepIn), None)
            }
            Command::StepOut(_) => {
                self.error("This feature is not currently supported.".into());
                (Ok(ResponseBody::StepOut), None)
            }
            Command::Terminate(_) => (Ok(ResponseBody::Terminate), Some(0)),
            Command::TerminateThreads(_) => (Ok(ResponseBody::TerminateThreads), Some(0)),
            Command::Threads => (
                Ok(ResponseBody::Threads(responses::ThreadsResponse {
                    threads: vec![types::Thread {
                        id: THREAD_ID,
                        name: "main".into(),
                    }],
                })),
                None,
            ),
            Command::Variables(ref args) => match self.handle_variables(args) {
                Ok(variables) => (
                    Ok(ResponseBody::Variables(responses::VariablesResponse {
                        variables,
                    })),
                    None,
                ),
                Err(e) => (Err(e), None),
            },
            _ => (Err(AdapterError::UnhandledCommand { command }), None),
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
        if !self.state.executors.is_empty() {
            return;
        }

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

    fn stop(&mut self, pc: Instruction) -> Result<bool, AdapterError> {
        let (hit_breakpoint_ids, reason) =
            if let Ok(breakpoint_id) = self.state.vm_pc_to_breakpoint_id(pc) {
                self.state.stopped_on_breakpoint_id = Some(breakpoint_id);
                (
                    Some(vec![breakpoint_id]),
                    types::StoppedEventReason::Breakpoint,
                )
            } else {
                self.state.stopped_on_breakpoint_id = None;
                (None, types::StoppedEventReason::Step)
            };

        let _ = self.server.send_event(Event::Stopped(StoppedEventBody {
            reason,
            hit_breakpoint_ids,
            description: None,
            thread_id: Some(THREAD_ID),
            preserve_focus_hint: None,
            text: None,
            all_threads_stopped: None,
        }));
        Ok(true)
    }

    /// Starts debugging all tests.
    /// `single_stepping` indicates whether the VM should break after one instruction.
    ///
    /// Returns true if it has stopped on a breakpoint or false if all tests have finished.
    fn start_debugging_tests(&mut self, single_stepping: bool) -> Result<bool, AdapterError> {
        self.state.update_vm_breakpoints();

        while let Some(executor) = self.state.executors.first_mut() {
            executor.interpreter.set_single_stepping(single_stepping);
            match executor.start_debugging()? {
                DebugResult::TestComplete(result) => {
                    self.state.test_complete(result);
                }
                DebugResult::Breakpoint(pc) => {
                    executor.interpreter.set_single_stepping(false);
                    return self.stop(pc);
                }
            };
        }
        self.log_test_results();
        Ok(false)
    }

    /// Continues debugging the current test and starts the next one if no breakpoint is hit.
    /// `single_stepping` indicates whether the VM should break after one instruction.
    ///
    /// Returns true if it has stopped on a breakpoint or false if all tests have finished.
    fn continue_debugging_tests(&mut self, single_stepping: bool) -> Result<bool, AdapterError> {
        self.state.update_vm_breakpoints();

        if let Some(executor) = self.state.executors.first_mut() {
            executor.interpreter.set_single_stepping(single_stepping);
            match executor.continue_debugging()? {
                DebugResult::TestComplete(result) => {
                    self.state.test_complete(result);
                    // The current test has finished, but there could be more tests to run. Start debugging the
                    // remaining tests.
                    return self.start_debugging_tests(single_stepping);
                }
                DebugResult::Breakpoint(pc) => {
                    executor.interpreter.set_single_stepping(false);
                    return self.stop(pc);
                }
            }
        }
        self.log_test_results();
        Ok(false)
    }
}
