mod handlers;
use dap::responses::*;
use dap::{events::OutputEventBody, types::Breakpoint};
use serde::{Deserialize, Serialize};
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
use dap::types::StartDebuggingRequestKind;
use forc_pkg::{
    self, manifest::ManifestFile, Built, BuiltPackage, PackageManifest, PackageManifestFile,
};
use thiserror::Error;
use rand::Rng;

#[derive(Error, Debug)]
enum AdapterError {
    #[error("Unhandled command")]
    UnhandledCommandError,

    #[error("Missing command")]
    MissingCommandError,

    #[error("Build failed")]
    BuildError,

    #[error("Test execution failed")]
    TestExecutionError {
        #[from]
        source: anyhow::Error,
    },
}

pub struct DapServer {
    server: Server<Stdin, Stdout>,
    source_map: SourceMap,
    mode: Option<StartDebuggingRequestKind>,
    breakpoints: Vec<Breakpoint>,
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
        }
    }

    pub fn start(&mut self) -> DynResult<()> {
        loop {
            match self.server.poll_request()? {
                Some(req) => {
                    let rsp = self.handle(req)?;
                    self.server.respond(rsp)?;
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
                // Set the breakpoints in the VM
                let mut rng = rand::thread_rng();
                let breakpoint = Breakpoint {
                    id: rng.gen(),
                    verified: true,
                    line: Some(args.line),
                    source: Some(args.source.clone()),
                    ..Default::default()
                };
                self.breakpoints.push(breakpoint);
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
            // Command::Completions(_) => todo!(),
            Command::ConfigurationDone => {
                self.log("configuration done req".into());
                // if let Some(StartDebuggingRequestKind::Launch) = self.mode {
                // let _ = self.handle_launch();
                // }
                Ok(ResponseBody::ConfigurationDone)
            }
            // Command::Continue(_) => todo!(),
            // Command::DataBreakpointInfo(_) => todo!(),
            // Command::Disassemble(_) => todo!(),
            Command::Disconnect(_) => process::exit(0),
            // Command::Evaluate(_) => todo!(),
            // Command::ExceptionInfo(_) => todo!(),
            // Command::Goto(_) => todo!(),
            // Command::GotoTargets(_) => todo!(),
            Command::Initialize(_) => Ok(ResponseBody::Initialize(types::Capabilities {
                // supports_function_breakpoints: Some(true),
                // supports_conditional_breakpoints: Some(true),
                // supports_hit_conditional_breakpoints: Some(true),
                // supports_goto_targets_request: Some(true),
                // supports_step_in_targets_request: Some(true),
                // support_suspend_debuggee: Some(true),
                // supports_data_breakpoints: Some(true),
                supports_breakpoint_locations_request: Some(true),
                supports_configuration_done_request: Some(true),
                ..Default::default()
            })),
            Command::Launch(_) => {
                self.mode = Some(StartDebuggingRequestKind::Launch);
                // let _ = self.handle_launch();
                Ok(ResponseBody::Launch)
            } //self.handle_launch(),
            // Command::LoadedSources => todo!(),
            // Command::Modules(_) => todo!(),
            // Command::Next(_) => todo!(),
            // Command::Pause(_) => todo!(),
            // Command::ReadMemory(_) => todo!(),
            // Command::Restart(_) => todo!(),
            // Command::RestartFrame(_) => todo!(),
            // Command::ReverseContinue(_) => todo!(),
            // Command::Scopes(_) => todo!(),
            Command::SetBreakpoints(ref args) => {
                let breakpoints = args
                    .breakpoints
                    .clone()
                    .unwrap_or_default()
                    .iter()
                    .map(|bp| {
                        let line = Some(bp.line);
                        types::Breakpoint {
                            verified: true,
                            line,
                            ..Default::default()
                        }
                    })
                    .collect::<Vec<_>>();
                self.breakpoints = breakpoints.clone();
                Ok(ResponseBody::SetBreakpoints(
                    responses::SetBreakpointsResponse { breakpoints },
                ))
            }
            Command::SetDataBreakpoints(_) => Ok(ResponseBody::SetDataBreakpoints(
                responses::SetDataBreakpointsResponse { breakpoints: self.breakpoints.clone() },
            )),
            Command::SetExceptionBreakpoints(_) => Ok(ResponseBody::SetExceptionBreakpoints(
                responses::SetExceptionBreakpointsResponse { breakpoints: None },
            )),
            // Command::SetExpression(_) => todo!(),
            Command::SetFunctionBreakpoints(_) => Ok(ResponseBody::SetFunctionBreakpoints(
                responses::SetFunctionBreakpointsResponse { breakpoints: self.breakpoints.clone() },
            )),
            Command::SetInstructionBreakpoints(_) => Ok(ResponseBody::SetInstructionBreakpoints(
                responses::SetInstructionBreakpointsResponse { breakpoints: self.breakpoints.clone() },
            )),
            // Command::SetVariable(_) => todo!(),
            // Command::Source(_) => todo!(),
            // Command::StackTrace(_) => todo!(),
            // Command::StepBack(_) => todo!(),
            // Command::StepIn(_) => todo!(),
            // Command::StepInTargets(_) => todo!(),
            // Command::StepOut(_) => todo!(),
            // Command::Terminate(_) => todo!(),
            // Command::TerminateThreads(_) => todo!(),
            Command::Threads => Ok(ResponseBody::Threads(responses::ThreadsResponse {
                threads: vec![types::Thread {
                    id: 1,
                    name: "main".into(),
                }],
            })),
            // Command::Variables(_) => todo!(),
            // Command::WriteMemory(_) => todo!(),
            // Command::Cancel(_) => todo!(),
            _ => Err(AdapterError::UnhandledCommandError),
        };

        self.log(format!("{:?}\n", rsp));

        match rsp {
            Ok(rsp) => Ok(req.success(rsp)),
            Err(e) => Ok(req.error(&format!("{:?}", e))),
        }
    }

    /// Log a message to the client's debugger console output.
    fn log(&mut self, output: String) {
        let _ = self.server.send_event(Event::Output(OutputEventBody {
            output,
            ..Default::default()
        }));
    }
}
