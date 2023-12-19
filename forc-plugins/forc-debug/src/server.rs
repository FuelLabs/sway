use dap::events::OutputEventBody;
use dap::responses::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{BufReader, BufWriter, Stdin, Stdout},
    path::PathBuf,
    process,
    sync::Arc, cmp::min,
};
use sway_core::source_map::PathIndex;
use sway_types::{Span, span::Position};
// use sway_core::source_map::SourceMap;
use thiserror::Error;

use dap::prelude::*;

use crate::types::DynResult;

#[derive(Error, Debug)]
enum AdapterError {
    #[error("Unhandled command")]
    UnhandledCommandError,

    #[error("Missing command")]
    MissingCommandError,
}

pub struct DapServer {
    server: Server<Stdin, Stdout>,
    source_map: SourceMap,
}

type Line = i64;
type Instruction = u64;
type SourceMap = HashMap<PathBuf, HashMap<Line, Instruction>>;

impl DapServer {
    pub fn new() -> Self {
        let output = BufWriter::new(std::io::stdout());
        let input = BufReader::new(std::io::stdin());
        let server = Server::new(input, output);
        DapServer {
            server,
            source_map: Default::default(),
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
                self.log("attach!\n\n".into());
                // let compiled_program = args.additional_data.
                let program =
                    "/Users/sophiedankel/Development/sway-playground/projects/swaypad/src/main.sw";
                let dir = PathBuf::from(program);
                let manifest_file = forc_pkg::manifest::ManifestFile::from_dir(&dir)?;
                let mut member_manifests = manifest_file.member_manifests()?;
                let lock_path = manifest_file.lock_path()?;
                let build_plan = forc_pkg::BuildPlan::from_lock_and_manifests(
                    &lock_path,
                    &member_manifests,
                    false,
                    false,
                    Default::default(),
                )?;

                // self.log(format!("build plan!\n{:?}\n", build_plan));

                // let compiled = forc_pkg::check(&plan, Default::default(), false, true, Default::default())?;

                let outputs = std::iter::once(
                    build_plan
                        .find_member_index(
                            &member_manifests.first_entry().unwrap().get().project.name,
                        )
                        .unwrap(),
                )
                .collect();

                let built_packages = forc_pkg::build(
                    &build_plan,
                    Default::default(),
                    &Default::default(),
                    &outputs,
                )?;

                // self.log(format!("built!\n{:?}\n", built_packages));

                built_packages.iter().for_each(|built_package| {
                    let compiled_package = &built_package.1;
                    let source_map = &compiled_package.source_map;
                    let pretty = serde_json::to_string_pretty(source_map).unwrap();
                    fs::write("/Users/sophiedankel/Development/sway-playground/projects/swaypad/src/tmp.txt", pretty).expect("Unable to write file");

                    let paths = &source_map.paths;
                    // Cache the source code for every path in the map, since we'll need it later.
                    let source_code = paths.iter().filter_map(|path_buf| {
                      if let Ok(source) = fs::read_to_string(path_buf) {
                        return Some((path_buf, source));
                      } else {
                        None
                      }
                    }).collect::<HashMap<_, _>>();

                    source_map.map.iter().for_each(|(instruction, sm_span)| {
                        let path_buf: &PathBuf = paths.get(sm_span.path.0).unwrap();

                        if let Some(source_code) = source_code.get(path_buf) {
                          if let Some(start_pos) = Position::new(&source_code, sm_span.range.start) {    
                              let (line, _) = start_pos.line_col();
                              let (line, instruction) = (line as i64, *instruction as u64);

                              self.source_map.entry(path_buf.clone()).and_modify(|new_map| {

                              new_map.entry(line as i64).and_modify(|val| {
                                // Choose the first instruction that maps to this line
                                *val = min(instruction, *val);
                              }).or_insert(instruction);
                            }).or_insert(HashMap::from([(line, instruction)]));

                            } else {
                              self.log(format!("Couldn't get position: {:?} in file: {:?}", sm_span.range.start, path_buf));
                            }
                        } else {
                          self.log(format!("Couldn't read file: {:?}", path_buf));
                        }
                    });

                    self.log("Writing source map!\n\n".into());
                    let pretty = serde_json::to_string_pretty(&self.source_map.clone()).unwrap();
                    fs::write("/Users/sophiedankel/Development/sway-playground/projects/swaypad/src/tmp2.txt", pretty).expect("Unable to write file");
                });
                // Run forc test
                ResponseBody::Attach
            }
            Command::BreakpointLocations(_) => {
                // Set the breakpoints in the VM
                ResponseBody::BreakpointLocations(responses::BreakpointLocationsResponse {
                    breakpoints: vec![types::BreakpointLocation {
                        line: 2,
                        ..Default::default()
                    }],
                })
            }
            // Command::Completions(_) => todo!(),
            // Command::ConfigurationDone => todo!(),
            // Command::Continue(_) => todo!(),
            // Command::DataBreakpointInfo(_) => todo!(),
            // Command::Disassemble(_) => todo!(),
            Command::Disconnect(_) => process::exit(0),
            // Command::Evaluate(_) => todo!(),
            // Command::ExceptionInfo(_) => todo!(),
            // Command::Goto(_) => todo!(),
            // Command::GotoTargets(_) => todo!(),
            Command::Initialize(_) => ResponseBody::Initialize(types::Capabilities {
                supports_function_breakpoints: Some(true),
                supports_conditional_breakpoints: Some(true),
                supports_hit_conditional_breakpoints: Some(true),
                supports_goto_targets_request: Some(true),
                supports_step_in_targets_request: Some(true),
                support_suspend_debuggee: Some(true),
                supports_data_breakpoints: Some(true),
                supports_breakpoint_locations_request: Some(true),
                ..Default::default()
            }),
            // Command::Launch(_) => todo!(),
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
                    .collect();
                ResponseBody::SetBreakpoints(responses::SetBreakpointsResponse { breakpoints })
            }
            // Command::SetDataBreakpoints(_) => todo!(),
            // Command::SetExceptionBreakpoints(_) => todo!(),
            // Command::SetExpression(_) => todo!(),
            // Command::SetFunctionBreakpoints(_) => todo!(),
            // Command::SetInstructionBreakpoints(_) => todo!(),
            // Command::SetVariable(_) => todo!(),
            // Command::Source(_) => todo!(),
            // Command::StackTrace(_) => todo!(),
            // Command::StepBack(_) => todo!(),
            // Command::StepIn(_) => todo!(),
            // Command::StepInTargets(_) => todo!(),
            // Command::StepOut(_) => todo!(),
            // Command::Terminate(_) => todo!(),
            // Command::TerminateThreads(_) => todo!(),
            // Command::Threads => todo!(),
            // Command::Variables(_) => todo!(),
            // Command::WriteMemory(_) => todo!(),
            // Command::Cancel(_) => todo!(),
            _ => {
                return Err(Box::new(AdapterError::UnhandledCommandError));
            }
        };

        self.log(format!("{:?}\n", rsp));

        Ok(req.success(rsp))
    }

    /// Log a message to the client's debugger console output.
    fn log(&mut self, output: String) {
        let _ = self.server.send_event(Event::Output(OutputEventBody {
            output,
            ..Default::default()
        }));
    }
}
