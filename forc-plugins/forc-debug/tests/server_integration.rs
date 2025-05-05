use dap::{
    events::{Event, OutputEventBody},
    requests::{Command, LaunchRequestArguments, SetBreakpointsArguments, VariablesArguments},
    responses::ResponseBody,
    types::{
        OutputEventCategory, Source, SourceBreakpoint, StartDebuggingRequestKind,
        StoppedEventReason, Variable,
    },
};
use forc_debug::server::{
    AdditionalData, DapServer, INSTRUCTIONS_VARIABLE_REF, REGISTERS_VARIABLE_REF,
};
use std::{
    collections::BTreeMap,
    env,
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
};

pub fn sway_workspace_dir() -> PathBuf {
    env::current_dir().unwrap().parent().unwrap().to_path_buf()
}

pub fn test_fixtures_dir() -> PathBuf {
    env::current_dir().unwrap().join("tests/fixtures")
}

#[derive(Debug, Default, Clone)]
/// A simple struct to capture event output from the server for testing purposes.
struct EventCapture {
    pub output: Arc<Mutex<String>>,
}

impl Write for EventCapture {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut lock = self.output.lock().unwrap();
        lock.push_str(&String::from_utf8_lossy(buf));
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl EventCapture {
    pub fn take_event(&self) -> Option<Event> {
        let mut lock = self.output.lock().unwrap();
        while !lock.is_empty() {
            let cloned = lock.clone();
            let (line, rest) = cloned.split_once('\n')?;
            *lock = rest.to_string();
            if let Ok(event) = serde_json::from_str::<Event>(line) {
                return Some(event);
            }
        }
        None
    }
}

#[test]
fn test_server_attach_mode() {
    let output_capture = EventCapture::default();
    let input = Box::new(std::io::stdin());
    let output = Box::new(output_capture.clone());
    let mut server = DapServer::new(input, output);

    // Initialize request
    let (result, exit_code) = server
        .handle_command(&Command::Initialize(Default::default()))
        .into_tuple();
    assert!(matches!(result, Ok(ResponseBody::Initialize(_))));
    assert!(exit_code.is_none());

    // Attach request
    let (result, exit_code) = server
        .handle_command(&Command::Attach(Default::default()))
        .into_tuple();
    assert!(matches!(result, Ok(ResponseBody::Attach)));
    assert_eq!(exit_code, Some(0));
    assert_not_supported_event(output_capture.take_event());
}

#[test]
fn test_server_launch_mode() {
    let output_capture = EventCapture::default();
    let input = Box::new(std::io::stdin());
    let output = Box::new(output_capture.clone());
    let mut server = DapServer::new(input, output);

    let program_path = test_fixtures_dir().join("simple/src/main.sw");
    let source_str = program_path.to_string_lossy().to_string();

    // Initialize request
    let (result, exit_code) = server
        .handle_command(&Command::Initialize(Default::default()))
        .into_tuple();
    assert!(matches!(result, Ok(ResponseBody::Initialize(_))));
    assert!(exit_code.is_none());

    // Launch request
    let additional_data = serde_json::to_value(AdditionalData {
        program: source_str.clone(),
    })
    .unwrap();
    let (result, exit_code) = server
        .handle_command(&Command::Launch(LaunchRequestArguments {
            additional_data: Some(additional_data),
            ..Default::default()
        }))
        .into_tuple();
    assert!(matches!(result, Ok(ResponseBody::Launch)));
    assert!(exit_code.is_none());

    // Set Breakpoints
    let (result, exit_code) = server
        .handle_command(&Command::SetBreakpoints(SetBreakpointsArguments {
            source: Source {
                path: Some(source_str.clone()),
                ..Default::default()
            },
            breakpoints: Some(vec![
                SourceBreakpoint {
                    line: 21,
                    ..Default::default()
                },
                SourceBreakpoint {
                    line: 30,
                    ..Default::default()
                },
                SourceBreakpoint {
                    line: 39,
                    ..Default::default()
                },
            ]),
            ..Default::default()
        }))
        .into_tuple();
    match result.expect("set breakpoints result") {
        ResponseBody::SetBreakpoints(res) => {
            assert!(res.breakpoints.len() == 3);
        }
        other => panic!("Expected SetBreakpoints response, got {:?}", other),
    }
    assert!(exit_code.is_none());

    // Configuration Done request
    let (result, exit_code) = server
        .handle_command(&Command::ConfigurationDone)
        .into_tuple();
    assert!(matches!(result, Ok(ResponseBody::ConfigurationDone)));
    assert!(exit_code.is_none());

    // Launch, should hit first breakpoint
    let keep_running = server.launch().expect("launched without error");
    assert!(keep_running);
    assert_stopped_breakpoint_event(output_capture.take_event(), 0);

    // Threads request
    let (result, exit_code) = server.handle_command(&Command::Threads).into_tuple();
    match result.expect("threads result") {
        ResponseBody::Threads(res) => {
            assert_eq!(res.threads.len(), 1);
        }
        other => panic!("Expected Threads response, got {:?}", other),
    }
    assert!(exit_code.is_none());

    // Stack Trace request
    let (result, exit_code) = server
        .handle_command(&Command::StackTrace(Default::default()))
        .into_tuple();
    match result.expect("stack trace result") {
        ResponseBody::StackTrace(res) => {
            assert_eq!(res.stack_frames.len(), 1);
        }
        other => panic!("Expected StackTrace response, got {:?}", other),
    }
    assert!(exit_code.is_none());

    // Scopes request
    let (result, exit_code) = server
        .handle_command(&Command::Scopes(Default::default()))
        .into_tuple();
    match result.expect("scopes result") {
        ResponseBody::Scopes(res) => {
            assert_eq!(res.scopes.len(), 2);
        }
        other => panic!("Expected Scopes response, got {:?}", other),
    }
    assert!(exit_code.is_none());

    // Variables request - registers
    let (result, exit_code) = server
        .handle_command(&Command::Variables(VariablesArguments {
            variables_reference: REGISTERS_VARIABLE_REF,
            ..Default::default()
        }))
        .into_tuple();
    match result.expect("registers variables result") {
        ResponseBody::Variables(res) => {
            assert_eq!(res.variables.len(), 64);
        }
        other => panic!("Expected Variables response, got {:?}", other),
    }
    assert!(exit_code.is_none());

    // Variables request - VM instructions
    let (result, exit_code) = server
        .handle_command(&Command::Variables(VariablesArguments {
            variables_reference: INSTRUCTIONS_VARIABLE_REF,
            ..Default::default()
        }))
        .into_tuple();
    match result.expect("instructions variables result") {
        ResponseBody::Variables(res) => {
            let expected = vec![
                ("Opcode", "SW"),
                ("rA", "reg59"),
                ("rB", "one"),
                ("imm", "0x1"),
            ];
            assert_variables_eq(expected, res.variables);
        }
        other => panic!("Expected Variables response, got {:?}", other),
    }
    assert!(exit_code.is_none());

    // Next request
    let (result, exit_code) = server
        .handle_command(&Command::Next(Default::default()))
        .into_tuple();
    assert!(result.is_ok());
    assert!(exit_code.is_none());
    assert_stopped_next_event(output_capture.take_event());

    // Step In request
    let (result, exit_code) = server
        .handle_command(&Command::StepIn(Default::default()))
        .into_tuple();
    assert!(result.is_ok());
    assert!(exit_code.is_none());
    assert_not_supported_event(output_capture.take_event());

    // Step Out request
    let (result, exit_code) = server
        .handle_command(&Command::StepOut(Default::default()))
        .into_tuple();
    assert!(result.is_ok());
    assert!(exit_code.is_none());
    assert_not_supported_event(output_capture.take_event());

    // Continue request, should hit 2nd breakpoint
    let (result, exit_code) = server
        .handle_command(&Command::Continue(Default::default()))
        .into_tuple();
    assert!(result.is_ok());
    assert!(exit_code.is_none());
    assert_stopped_breakpoint_event(output_capture.take_event(), 1);

    // Continue request, should hit 3rd breakpoint
    let (result, exit_code) = server
        .handle_command(&Command::Continue(Default::default()))
        .into_tuple();
    assert!(result.is_ok());
    assert!(exit_code.is_none());
    assert_stopped_breakpoint_event(output_capture.take_event(), 2);

    // Continue request, should exit cleanly
    let (result, exit_code) = server
        .handle_command(&Command::Continue(Default::default()))
        .into_tuple();
    assert!(result.is_ok());
    assert_eq!(exit_code, Some(0));

    // Test results should be logged
    let body = assert_output_event_body(output_capture.take_event());
    assert!(body.category.is_none());
    assert!(body.output.contains("test test_1 ... ok"));
    assert!(body.output.contains("test test_2 ... ok"));
    assert!(body.output.contains("test test_3 ... ok"));
    assert!(body.output.contains("Result: OK. 3 passed. 0 failed"));
}

#[test]
fn test_sourcemap_build() {
    let mut server = DapServer::new(Box::new(std::io::stdin()), Box::new(std::io::sink()));

    let program_path = test_fixtures_dir().join("simple/src/main.sw");

    // Initialize and set the program path
    server.handle_command(&Command::Initialize(Default::default()));
    server.state.program_path = program_path.clone();
    server.state.mode = Some(StartDebuggingRequestKind::Launch);

    // Explicitly build the tests
    server.build_tests().expect("Failed to build tests");

    // Group instructions by line number
    let mut line_to_instructions: BTreeMap<i64, Vec<usize>> = BTreeMap::new();
    let source_map = &server.state.source_map;

    for pc in source_map.map.keys() {
        if let Some((path, range)) = source_map.addr_to_span(*pc) {
            if path == program_path {
                line_to_instructions
                    .entry(range.start.line as i64)
                    .or_default()
                    .push(*pc);
            }
        }
    }

    // Verify essential source locations are mapped correctly
    let key_locations = [
        // Main function and its contents
        (3, 2, "main function parameters"), // Should have 2 instructions
        (4, 2, "addition operation"),       // Should have 2 instructions (add operation)
        // Helper function and its contents
        (11, 2, "helper function parameters"), // Should have 2 instructions
        (12, 2, "helper addition operation"),  // Should have 2 instructions
        // Test functions (identical patterns)
        (21, 1, "test_1 first line"),  // Each test line should have
        (22, 1, "test_1 second line"), // exactly one instruction
        (23, 1, "test_1 helper call"),
        (24, 1, "test_1 assertion"),
    ];

    for (line, expected_count, description) in key_locations {
        let instructions = line_to_instructions
            .get(&line)
            .unwrap_or_else(|| panic!("Missing mapping for line {}: {}", line, description));
        assert_eq!(
            instructions.len(),
            expected_count,
            "Line {} ({}): Expected {} instructions, found {:?}",
            line,
            description,
            expected_count,
            instructions
        );
    }
}

/// Asserts that the given event is a Stopped event with a breakpoint reason and the given breakpoint ID.
fn assert_stopped_breakpoint_event(event: Option<Event>, breakpoint_id: i64) {
    match event.expect("received event") {
        Event::Stopped(body) => {
            assert!(matches!(body.reason, StoppedEventReason::Breakpoint));
            assert_eq!(body.hit_breakpoint_ids, Some(vec![breakpoint_id]));
        }
        other => panic!("Expected Stopped event, got {:?}", other),
    };
}

/// Asserts that the given event is a Stopped event with the right reason and no breakpoint ID.
fn assert_stopped_next_event(event: Option<Event>) {
    match event.expect("received event") {
        Event::Stopped(body) => {
            assert!(matches!(body.reason, StoppedEventReason::Step));
            assert_eq!(body.hit_breakpoint_ids, None);
        }
        other => panic!("Expected Stopped event, got {:?}", other),
    };
}

fn assert_output_event_body(event: Option<Event>) -> OutputEventBody {
    match event.expect("received event") {
        Event::Output(body) => body,
        other => panic!("Expected Output event, got {:?}", other),
    }
}

fn assert_not_supported_event(event: Option<Event>) {
    let body = assert_output_event_body(event);
    assert_eq!(body.output, "This feature is not currently supported.");
    assert!(matches!(body.category, Some(OutputEventCategory::Stderr)));
}

/// Asserts that the given variables match the expected (name, value) pairs.
fn assert_variables_eq(expected: Vec<(&str, &str)>, actual: Vec<Variable>) {
    assert_eq!(actual.len(), expected.len());
    for (i, (name, value)) in expected.iter().enumerate() {
        assert_eq!(actual[i].name, *name);
        assert_eq!(actual[i].value, *value);
    }
}
