mod handlers;
mod state;
mod util;

use crate::{
    error::{self, AdapterError, Error},
    server::{state::ServerState, util::IdGenerator},
    types::{ExitCode, Instruction},
};
use dap::{
    events::{ExitedEventBody, OutputEventBody, StoppedEventBody},
    prelude::*,
    types::StartDebuggingRequestKind,
};
use forc_pkg::{
    manifest::GenericManifestFile,
    source::IPFSNode,
    {self, BuildProfile, Built, BuiltPackage, PackageManifestFile},
};
use forc_test::{
    execute::{DebugResult, TestExecutor},
    setup::TestSetup,
    BuiltTests,
};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufReader, BufWriter, Read, Write},
    process,
    sync::Arc,
};
use sway_core::BuildTarget;

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
    pub state: ServerState,
}

impl Default for DapServer {
    fn default() -> Self {
        Self::new(Box::new(std::io::stdin()), Box::new(std::io::stdout()))
    }
}

impl DapServer {
    /// Creates a new DAP server with custom input and output streams.
    ///
    /// # Arguments
    /// * `input` - Source of DAP protocol messages (usually stdin)
    /// * `output` - Destination for DAP protocol messages (usually stdout)
    pub fn new(input: Box<dyn Read>, output: Box<dyn Write>) -> Self {
        let server = Server::new(BufReader::new(input), BufWriter::new(output));
        DapServer {
            server,
            state: ServerState::default(),
            breakpoint_id_gen: IdGenerator::default(),
        }
    }

    /// Runs the debug server event loop, handling client requests until completion or error.
    pub fn start(&mut self) -> error::Result<()> {
        loop {
            let req = match self.server.poll_request()? {
                Some(req) => req,
                None => return Err(Error::AdapterError(AdapterError::MissingCommand)),
            };

            // Handle the request and send response
            let response = self.handle_request(req)?;
            self.server.respond(response)?;

            // Handle one-time initialization
            if !self.state.initialized_event_sent {
                let _ = self.server.send_event(Event::Initialized);
                self.state.initialized_event_sent = true;
            }

            // Handle launch after configuration is complete
            if self.should_launch() {
                self.state.started_debugging = true;
                match self.launch() {
                    Ok(true) => continue,
                    Ok(false) => self.exit(0), // The tests finished executing
                    Err(e) => {
                        self.error(format!("Launch error: {e:?}"));
                        self.exit(1);
                    }
                }
            }
        }
    }

    /// Processes a debug adapter request and generates appropriate response.
    fn handle_request(&mut self, req: Request) -> error::Result<Response> {
        let (result, exit_code) = self.handle_command(&req.command).into_tuple();
        let response = match result {
            Ok(rsp) => Ok(req.success(rsp)),
            Err(e) => {
                self.error(format!("{e:?}"));
                Ok(req.error(&format!("{e:?}")))
            }
        };
        if let Some(exit_code) = exit_code {
            self.exit(exit_code);
        }
        response
    }

    /// Handles a command and returns the result and exit code, if any.
    pub fn handle_command(&mut self, command: &Command) -> HandlerResult {
        match command {
            Command::Attach(_) => self.handle_attach(),
            Command::BreakpointLocations(ref args) => {
                self.handle_breakpoint_locations_command(args)
            }
            Command::ConfigurationDone => self.handle_configuration_done(),
            Command::Continue(_) => self.handle_continue(),
            Command::Disconnect(_) => HandlerResult::ok_with_exit(ResponseBody::Disconnect, 0),
            Command::Evaluate(args) => self.handle_evaluate(args),
            Command::Initialize(_) => self.handle_initialize(),
            Command::Launch(ref args) => self.handle_launch(args),
            Command::Next(_) => self.handle_next(),
            Command::Pause(_) => self.handle_pause(),
            Command::Restart(_) => self.handle_restart(),
            Command::Scopes(_) => self.handle_scopes(),
            Command::SetBreakpoints(ref args) => self.handle_set_breakpoints_command(args),
            Command::StackTrace(_) => self.handle_stack_trace_command(),
            Command::StepIn(_) => {
                self.error("This feature is not currently supported.".into());
                HandlerResult::ok(ResponseBody::StepIn)
            }
            Command::StepOut(_) => {
                self.error("This feature is not currently supported.".into());
                HandlerResult::ok(ResponseBody::StepOut)
            }
            Command::Terminate(_) => HandlerResult::ok_with_exit(ResponseBody::Terminate, 0),
            Command::TerminateThreads(_) => {
                HandlerResult::ok_with_exit(ResponseBody::TerminateThreads, 0)
            }
            Command::Threads => self.handle_threads(),
            Command::Variables(ref args) => self.handle_variables_command(args),
            _ => HandlerResult::err(AdapterError::UnhandledCommand {
                command: command.clone(),
            }),
        }
    }

    /// Checks whether debug session is ready to begin launching tests.
    fn should_launch(&self) -> bool {
        self.state.configuration_done
            && !self.state.started_debugging
            && matches!(self.state.mode, Some(StartDebuggingRequestKind::Launch))
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

    /// Logs test execution results in a cargo-test-like format, showing duration and gas usage for each test.
    fn log_test_results(&mut self) {
        if !self.state.executors.is_empty() {
            return;
        }
        let test_results = &self.state.test_results;
        let test_lines = test_results
            .iter()
            .map(|r| {
                let outcome = if r.passed() { "ok" } else { "failed" };
                format!(
                    "test {} ... {} ({}ms, {} gas)",
                    r.name,
                    outcome,
                    r.duration.as_millis(),
                    r.gas_used
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let passed = test_results.iter().filter(|r| r.passed()).count();
        let final_outcome = if passed == test_results.len() {
            "OK"
        } else {
            "FAILED"
        };

        self.log(format!(
            "{test_lines}\nResult: {final_outcome}. {passed} passed. {} failed.\n",
            test_results.len() - passed
        ));
    }

    /// Handles a `launch` request. Returns true if the server should continue running.
    pub fn launch(&mut self) -> Result<bool, AdapterError> {
        // Build tests for the given path.
        let (pkg_to_debug, test_setup) = self.build_tests()?;
        let entries = pkg_to_debug.bytecode.entries.iter().filter_map(|entry| {
            if let Some(test_entry) = entry.kind.test() {
                return Some((entry, test_entry));
            }
            None
        });

        // Construct a TestExecutor for each test and store it
        let executors: Vec<TestExecutor> = entries
            .filter_map(|(entry, test_entry)| {
                let offset = u32::try_from(entry.finalized.imm)
                    .expect("test instruction offset out of range");
                let name = entry.finalized.fn_name.clone();
                if test_entry.file_path.as_path() != self.state.program_path.as_path() {
                    return None;
                }

                TestExecutor::build(
                    &pkg_to_debug.bytecode.bytes,
                    offset,
                    test_setup.clone(),
                    test_entry,
                    name.clone(),
                )
                .ok()
            })
            .collect();
        self.state.init_executors(executors);

        // Start debugging
        self.start_debugging_tests(false)
    }

    /// Builds the tests at the given [PathBuf] and stores the source maps.
    pub fn build_tests(&mut self) -> Result<(BuiltPackage, TestSetup), AdapterError> {
        if let Some(pkg) = &self.state.built_package {
            if let Some(setup) = &self.state.test_setup {
                return Ok((pkg.clone(), setup.clone()));
            }
        }

        // 1. Build the packages
        let manifest_file = forc_pkg::manifest::ManifestFile::from_dir(&self.state.program_path)
            .map_err(|err| AdapterError::BuildFailed {
                reason: format!("read manifest file: {err:?}"),
            })?;
        let pkg_manifest: PackageManifestFile =
            manifest_file
                .clone()
                .try_into()
                .map_err(|err: anyhow::Error| AdapterError::BuildFailed {
                    reason: format!("package manifest: {err:?}"),
                })?;
        let member_manifests =
            manifest_file
                .member_manifests()
                .map_err(|err| AdapterError::BuildFailed {
                    reason: format!("member manifests: {err:?}"),
                })?;
        let lock_path = manifest_file
            .lock_path()
            .map_err(|err| AdapterError::BuildFailed {
                reason: format!("lock path: {err:?}"),
            })?;
        let build_plan = forc_pkg::BuildPlan::from_lock_and_manifests(
            &lock_path,
            &member_manifests,
            false,
            false,
            &IPFSNode::default(),
        )
        .map_err(|err| AdapterError::BuildFailed {
            reason: format!("build plan: {err:?}"),
        })?;

        let project_name = pkg_manifest.project_name();

        let outputs = std::iter::once(build_plan.find_member_index(project_name).ok_or(
            AdapterError::BuildFailed {
                reason: format!("find built project: {project_name}"),
            },
        )?)
        .collect();

        let built_packages = forc_pkg::build(
            &build_plan,
            BuildTarget::default(),
            &BuildProfile {
                optimization_level: sway_core::OptLevel::Opt0,
                include_tests: true,
                ..Default::default()
            },
            &outputs,
            &[],
            &[],
            None,
        )
        .map_err(|err| AdapterError::BuildFailed {
            reason: format!("build packages: {err:?}"),
        })?;

        // 2. Store the source maps and find debug package
        let pkg_to_debug = built_packages
            .iter()
            .find(|(_, pkg)| pkg.descriptor.manifest_file == pkg_manifest)
            .map(|(_, pkg)| pkg)
            .ok_or(AdapterError::BuildFailed {
                reason: format!("find package: {project_name}"),
            })?;

        self.state.source_map = pkg_to_debug.source_map.clone();

        // 3. Build the tests
        let built = Built::Package(Arc::from(pkg_to_debug.clone()));

        let built_tests = BuiltTests::from_built(built, &build_plan).map_err(|err| {
            AdapterError::BuildFailed {
                reason: format!("build tests: {err:?}"),
            }
        })?;

        let pkg_tests = match built_tests {
            BuiltTests::Package(pkg_tests) => pkg_tests,
            BuiltTests::Workspace(_) => {
                return Err(AdapterError::BuildFailed {
                    reason: "package tests: workspace tests not supported".into(),
                })
            }
        };
        let test_setup = pkg_tests.setup().map_err(|err| AdapterError::BuildFailed {
            reason: format!("test setup: {err:?}"),
        })?;
        self.state.built_package = Some(pkg_to_debug.clone());
        self.state.test_setup = Some(test_setup.clone());
        Ok((pkg_to_debug.clone(), test_setup))
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

/// Represents the result of a DAP handler operation, combining the response/error and an optional exit code
#[derive(Debug)]
pub struct HandlerResult {
    response: Result<ResponseBody, AdapterError>,
    exit_code: Option<ExitCode>,
}

impl HandlerResult {
    /// Creates a new successful result with no exit code
    pub fn ok(response: ResponseBody) -> Self {
        Self {
            response: Ok(response),
            exit_code: None,
        }
    }

    /// Creates a new successful result with an exit code
    pub fn ok_with_exit(response: ResponseBody, code: ExitCode) -> Self {
        Self {
            response: Ok(response),
            exit_code: Some(code),
        }
    }

    /// Creates a new error result with an exit code
    pub fn err_with_exit(error: AdapterError, code: ExitCode) -> Self {
        Self {
            response: Err(error),
            exit_code: Some(code),
        }
    }

    /// Creates a new error result with no exit code
    pub fn err(error: AdapterError) -> Self {
        Self {
            response: Err(error),
            exit_code: None,
        }
    }

    /// Deconstructs the result into its original tuple form
    pub fn into_tuple(self) -> (Result<ResponseBody, AdapterError>, Option<ExitCode>) {
        (self.response, self.exit_code)
    }
}
