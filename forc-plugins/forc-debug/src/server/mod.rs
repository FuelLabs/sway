mod handlers;
use crate::names::register_name;
use dap::events::{ExitedEventBody, StoppedEventBody, ThreadEventBody};
use dap::responses::*;
use dap::{events::OutputEventBody, types::Breakpoint};
use forc_test::execute::TestExecutor;
use fuel_vm::fuel_asm::Word;
use serde::{Deserialize, Serialize};
use std::fmt::format;
use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    fs,
    io::{BufReader, BufWriter, Stdin, Stdout},
    path::PathBuf,
    process,
    sync::Arc,
};
use sway_core::source_map::PathIndex;
use sway_types::{span::Position, Span};
// use sway_core::source_map::SourceMap;
use crate::types::DynResult;
use dap::prelude::*;
use dap::types::{
    PresentationHint, Scope, Source, StartDebuggingRequestKind, Variable, VariablePresentationHint,
};
use forc_pkg::{
    self, manifest::ManifestFile, Built, BuiltPackage, PackageManifest, PackageManifestFile,
};
use forc_test::execute::DebugResult;
use fuel_core_client::client::schema::schema::__fields::Mutation::_set_breakpoint_arguments::breakpoint;
use fuel_vm::consts::VM_REGISTER_COUNT;
use rand::Rng;
use thiserror::Error;

pub const THREAD_ID: i64 = 0;
pub const REGISTERS_VARIABLE_REF: i64 = 1;

#[derive(Error, Debug)]
enum AdapterError {
    #[error("Unhandled command")]
    UnhandledCommandError,

    #[error("Missing command")]
    MissingCommandError,

    #[error("Missing configuration")]
    MissingConfigurationError,

    #[error("Missing source map")]
    MissingSourceMapError {
        pc: u64
    },

    #[error("Unknown breakpoint")]
    UnknownBreakpointError,

    #[error("Build failed")]
    BuildError,

    #[error("Test execution failed")]
    TestExecutionError {
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
    source_map: SourceMap,
    mode: Option<StartDebuggingRequestKind>,
    breakpoints: Vec<Breakpoint>,
    initialized_event_sent: bool,
    started_debugging: bool,
    configuration_done: bool,
    current_breakpoint_id: Option<i64>,
    program_path: Option<PathBuf>,
    executors: Vec<TestExecutor>,
    test_results: HashMap<String, forc_test::TestResult>,
}

pub type Line = i64;
pub type Instruction = u64;
pub type SourceMap = HashMap<PathBuf, HashMap<Line, Instruction>>;

impl DapServer {
    pub fn new() -> Self {
        let output = BufWriter::new(std::io::stdout());
        let input = BufReader::new(std::io::stdin());
        let server = Server::new(input, output);
        DapServer {
            server,
            source_map: Default::default(),
            mode: None,
            breakpoints: Default::default(),
            initialized_event_sent: false,
            started_debugging: false,
            configuration_done: false,
            current_breakpoint_id: None,
            program_path: None,
            executors: Default::default(),
            test_results: Default::default(),
        }
    }

    pub fn start(&mut self) -> DynResult<()> {
        loop {
            match self.server.poll_request()? {
                Some(req) => {
                    let rsp = self.handle(req)?;
                    self.server.respond(rsp)?;

                    if !self.initialized_event_sent {
                        let _ = self.server.send_event(Event::Initialized);
                        self.initialized_event_sent = true;
                    }
                    if self.configuration_done == true && self.started_debugging == false {
                        if let Some(StartDebuggingRequestKind::Launch) = self.mode {
                            if let Some(program_path) = &self.program_path {
                                self.started_debugging = true;
                                match self.handle_launch(program_path.clone()) {
                                    Ok(true) => {}
                                    Ok(false) => {
                                        // The tests finished executing
                                        self.exit(0);
                                    }
                                    Err(e) => {
                                        self.error(format!("launch error: {:?}", e));
                                        self.exit(1);
                                    }
                                }
                            }
                        }
                    }
                }
                None => return Err(Box::new(AdapterError::MissingCommandError)),
            };
        }
    }

    fn handle(&mut self, req: Request) -> DynResult<Response> {
        self.log(format!("{:?}\n", req));

        let rsp = match req.command {
            Command::Attach(_) => {
                self.mode = Some(StartDebuggingRequestKind::Attach);
                Ok(ResponseBody::Attach)
            }
            Command::BreakpointLocations(ref args) => {
                // Add this breakpoint if we don't already have it
                match self.breakpoints.iter().find(|bp| {
                    if let Some(source) = &bp.source {
                        if let Some(line) = bp.line {
                            if args.source.path == source.path && args.line == line {
                                return true;
                            }
                        }
                    }
                    false
                }) {
                    Some(_) => {}
                    None => {
                        self.log(format!("bp locations bp did not exist!\n\n"));
                    }
                }

                let breakpoints = self
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
            // Command::Cancel(_) => todo!(),
            Command::ConfigurationDone => {
                self.configuration_done = true;
                Ok(ResponseBody::ConfigurationDone)
            }
            Command::Continue(_) => {
                //  While there are more tests to execute, either start debugging or continue
                match self.handle_continue() {
                    Ok(true) => {}
                    Ok(false) => {
                        // The tests finished executing
                        self.exit(0);
                    }
                    Err(e) => {
                        self.error(format!("continue error: {:?}", e));
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
                self.mode = Some(StartDebuggingRequestKind::Launch);
                let data = serde_json::from_value::<AdditionalData>(
                    args.additional_data
                        .as_ref()
                        .ok_or(AdapterError::MissingConfigurationError)?
                        .clone(),
                )
                .map_err(|_| AdapterError::MissingConfigurationError)?;
                self.program_path = Some(PathBuf::from(data.program));
                Ok(ResponseBody::Launch)
            }
            Command::Scopes(ref args) => Ok(ResponseBody::Scopes(responses::ScopesResponse {
                scopes: vec![Scope {
                    name: "Registers".into(),
                    presentation_hint: Some(types::ScopePresentationhint::Registers),
                    variables_reference: REGISTERS_VARIABLE_REF,
                    named_variables: None,
                    indexed_variables: None,
                    expensive: false,
                    source: None,
                    line: None,
                    column: None,
                    end_line: None,
                    end_column: None,
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
                        match self.breakpoints.iter().find(|bp| {
                            if let Some(source) = &bp.source {
                                if let Some(line) = bp.line {
                                    if args.source.path == source.path && source_bp.line == line {
                                        return true;
                                    }
                                }
                            }
                            false
                        }) {
                            Some(existing_bp) => {
                                return existing_bp.clone();
                            }
                            None => {
                                let id = rng.gen_range(0..1000000);
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
                self.breakpoints = breakpoints.clone();
                Ok(ResponseBody::SetBreakpoints(
                    responses::SetBreakpointsResponse { breakpoints },
                ))
            }
            Command::SetDataBreakpoints(_) => Ok(ResponseBody::SetDataBreakpoints(
                responses::SetDataBreakpointsResponse {
                    breakpoints: self.breakpoints.clone(),
                },
            )),
            Command::SetInstructionBreakpoints(ref args) => {
                self.log(format!("set instruction breakpoints args: {:?}\n", args));
                Ok(ResponseBody::SetInstructionBreakpoints(
                    responses::SetInstructionBreakpointsResponse {
                        breakpoints: self.breakpoints.clone(),
                    },
                ))
            }
            Command::StackTrace(ref args) => {
                let executor = self.executors.get_mut(0).unwrap();

                // For now, we only return 1 stack frame.
                let stack_frames = self
                    .current_breakpoint_id
                    .map(|bp_id| {
                        self.breakpoints
                            .iter()
                            .find(|bp| bp.id == Some(bp_id))
                            .map(|bp| {
                                vec![types::StackFrame {
                                    id: 0,
                                    name: executor.name.clone().into(),
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
                    .flatten()
                    .unwrap_or_default();

                Ok(ResponseBody::StackTrace(responses::StackTraceResponse {
                    stack_frames,
                    total_frames: None,
                }))
            }
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
            Command::Variables(ref args) => {
                let variables = self
                    .executors.get_mut(0)
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
            _ => Err(AdapterError::UnhandledCommandError),
        };

        self.log(format!("{:?}\n", rsp));

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

    /// Sends the 'exited' event to the client and kills the server process.
    fn exit(&mut self, exit_code: i64) {
        let _ = self
            .server
            .send_event(Event::Exited(ExitedEventBody { exit_code }));
        process::exit(exit_code as i32);
    }

    /// Updates the breakpoints in the VM.
    fn update_vm_breakpoints(&mut self) {
        if let Some(executor) = self.executors.get_mut(0) {
            if let Some(program_path) = &self.program_path {
                // Divide by 4 to get the opcode offset rather than the program counter offset.
                let opcode_offset = (executor.test_offset as u64) / 4;

                self.breakpoints.iter().for_each(|bp| {
                    // When the breakpoint is applied, $is is added. We only need to provide the index of the instruction
                    // from the beginning of the script.
                    let opcode_index = *self
                        .source_map
                        .get(program_path)
                        .unwrap()
                        .get(&bp.line.unwrap())
                        .unwrap();
                    let bp = fuel_vm::state::Breakpoint::script(opcode_index + opcode_offset);

                    // TODO: set all breakpoints in the VM
                    executor.interpreter.set_breakpoint(bp);
                });
            }
        }
    }

    fn vm_pc_to_breakpoint_id(&mut self, pc: u64) -> Result<i64, AdapterError> {
        if let Some(executor) = self.executors.get_mut(0) {
            if let Some(program_path) = &self.program_path {
                if let Some(source_map) = &self.source_map.get(program_path) {
                    // Divide by 4 to get the opcode offset rather than the program counter offset.
                    let instruction_offset = (pc - executor.test_offset as u64) / 4; // TODO: fix offset for 2nd or 3rd test

                    let (line, _) = source_map
                        .iter()
                        .find(|(_, pc)| **pc == instruction_offset)
                        .ok_or(AdapterError::MissingSourceMapError { pc })?;

                    let breakpoint_id = self
                        .breakpoints
                        .iter()
                        .find(|bp| bp.line == Some(*line))
                        .ok_or(AdapterError::UnknownBreakpointError)?
                        .id
                        .ok_or(AdapterError::UnknownBreakpointError)?;

                    return Ok(breakpoint_id);
                }
            }
        }
        Err(AdapterError::UnknownBreakpointError)
    }

    fn send_stopped_event(&mut self, pc: u64) -> Result<bool, AdapterError> {
        let breakpoint_id = self.vm_pc_to_breakpoint_id(pc)?;
        self.current_breakpoint_id = Some(breakpoint_id);
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
}
