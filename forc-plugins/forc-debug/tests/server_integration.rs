use dap::{
    events::{Event, OutputEventBody},
    requests::{Command, LaunchRequestArguments, SetBreakpointsArguments, VariablesArguments},
    responses::ResponseBody,
    types::{OutputEventCategory, Source, SourceBreakpoint, StartDebuggingRequestKind, StoppedEventReason, Variable},
};
use forc_debug::server::{
    AdditionalData, DapServer, INSTRUCTIONS_VARIABLE_REF, REGISTERS_VARIABLE_REF,
};
use itertools::Itertools;
use std::{
    collections::BTreeMap, env, io::Write, path::PathBuf, sync::{Arc, Mutex}
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


#[test]
fn test_sourcemap_build() {
    let mut server = DapServer::new(
        Box::new(std::io::stdin()), 
        Box::new(std::io::sink())
    );

    let program_path = test_fixtures_dir().join("simple/src/main.sw");
    let source_str = program_path.to_string_lossy().to_string();

    // Initialize and set the program path
    server.handle_command(&Command::Initialize(Default::default()));
    server.state.program_path = PathBuf::from(&source_str);
    server.state.mode = Some(StartDebuggingRequestKind::Launch);

    // Explicitly build the tests
    server.build_tests().expect("Failed to build tests");

    // Now examine what was built
    let source_map = &server.state.source_map;
    let file_map = source_map.get(&PathBuf::from(&source_str))
        .expect("Should have source map for our file");
    
    // Print out all line mappings in order
    let mut lines: Vec<i64> = file_map.keys().cloned().collect();
    lines.sort();
    
    println!("\nComplete source map for {}:", source_str);
    for line in lines {
        let instructions = file_map.get(&line).unwrap();
        println!("Line {:2}: Instructions {:?}", line, instructions);
    }

    // Also print the source code with line numbers for reference
    let source_content = std::fs::read_to_string(&program_path).unwrap();
    println!("\nSource code with line numbers:");
    for (i, line) in source_content.lines().enumerate() {
        println!("{:2}: {}", i + 1, line);
    }
}


#[test]
fn test_sourcemap_patterns() {
    let mut server = DapServer::new(
        Box::new(std::io::stdin()), 
        Box::new(std::io::sink())
    );

    let program_path = test_fixtures_dir().join("simple/src/main.sw");
    let source_str = program_path.to_string_lossy().to_string();

    // Initialize and build
    server.handle_command(&Command::Initialize(Default::default()));
    server.state.program_path = PathBuf::from(&source_str);
    server.state.mode = Some(StartDebuggingRequestKind::Launch);
    server.build_tests().expect("Failed to build tests");

    let source_map = &server.state.source_map;
    let file_map = source_map.get(&PathBuf::from(&source_str))
        .expect("Should have source map for our file");

    // Analyze instruction groupings
    println!("\nInstruction groupings by function:");
    
    // Main function instructions (lines 3-8)
    println!("\nMain function:");
    for line in 3..=8 {
        if let Some(instructions) = file_map.get(&line) {
            println!("  Line {:2}: Instructions {:?}", line, instructions);
        }
    }

    // Helper function instructions (lines 11-16)
    println!("\nHelper function:");
    for line in 11..=16 {
        if let Some(instructions) = file_map.get(&line) {
            println!("  Line {:2}: Instructions {:?}", line, instructions);
        }
    }

    // Test function instructions
    let test_ranges = [(20..=24, "test_1"), (28..=32, "test_2"), (36..=40, "test_3")];
    
    for (range, name) in test_ranges.clone() {
        println!("\n{}:", name);
        for line in range {
            if let Some(instructions) = file_map.get(&line) {
                println!("  Line {:2}: Instructions {:?}", line, instructions);
            }
        }
    }

    // Verify consistent patterns within each group
    
    // 1. Main and helper functions should have similar instruction patterns
    let main_params = file_map.get(&3).expect("main params");
    let helper_params = file_map.get(&11).expect("helper params");
    assert_eq!(main_params.len(), helper_params.len(), 
               "Main and helper should have same number of param instructions");

    // 2. Test functions should have identical instruction patterns
    let test1_instructions: Vec<_> = (21..=24)
        .filter_map(|line| file_map.get(&line))
        .collect();
    let test2_instructions: Vec<_> = (29..=32)
        .filter_map(|line| file_map.get(&line))
        .collect();
    let test3_instructions: Vec<_> = (37..=40)
        .filter_map(|line| file_map.get(&line))
        .collect();

    // Each corresponding instruction set in the test functions should have the same size
    for i in 0..test1_instructions.len() {
        assert_eq!(
            test1_instructions[i].len(),
            test2_instructions[i].len(),
            "Test 1 and 2 instruction counts should match at step {}", i
        );
        assert_eq!(
            test2_instructions[i].len(),
            test3_instructions[i].len(),
            "Test 2 and 3 instruction counts should match at step {}", i
        );
    }

    // 3. Instructions within each function should be sequential
    let verify_sequential_range = |range: std::ops::RangeInclusive<i64>, name: &str| {
        let mut last_instr = None;
        for line in range {
            if let Some(instructions) = file_map.get(&line) {
                for &instr in instructions {
                    if let Some(last) = last_instr {
                        if instr < last {
                            println!("Warning: Non-sequential instruction in {}: {} -> {}", 
                                   name, last, instr);
                        }
                    }
                    last_instr = Some(instr);
                }
            }
        }
    };

    verify_sequential_range(3..=8, "main");
    verify_sequential_range(11..=16, "helper");
    for (range, name) in test_ranges {
        verify_sequential_range(range, name);
    }
}

#[test]
fn test_sourcemap_instruction_patterns() {
    let mut server = DapServer::new(
        Box::new(std::io::stdin()), 
        Box::new(std::io::sink())
    );

    let program_path = test_fixtures_dir().join("simple/src/main.sw");
    let source_str = program_path.to_string_lossy().to_string();

    // Initialize and build
    server.handle_command(&Command::Initialize(Default::default()));
    server.state.program_path = PathBuf::from(&source_str);
    server.state.mode = Some(StartDebuggingRequestKind::Launch);
    server.build_tests().expect("Failed to build tests");

    let source_map = &server.state.source_map;
    let file_map = source_map.get(&PathBuf::from(&source_str))
        .expect("Should have source map for our file");

    // 1. Verify main function instruction patterns
    let main_params = file_map.get(&3).expect("main params");
    assert_eq!(main_params.len(), 4, "Main function parameter setup uses 4 instructions");
    assert_eq!(main_params, &vec![159, 160, 163, 164]);

    let main_add = file_map.get(&4).expect("main addition");
    assert_eq!(main_add.len(), 2, "Addition operation uses 2 instructions");
    assert_eq!(main_add, &vec![166, 167]);

    // 2. Verify helper function has same pattern as main
    let helper_params = file_map.get(&11).expect("helper params");
    assert_eq!(helper_params.len(), 4, "Helper function parameter setup uses 4 instructions");
    assert_eq!(helper_params, &vec![262, 263, 266, 267]);

    let helper_add = file_map.get(&12).expect("helper addition");
    assert_eq!(helper_add.len(), 2, "Addition operation uses 2 instructions");
    assert_eq!(helper_add, &vec![269, 270]);

    // 3. Verify test function patterns
    struct TestPattern {
        let_hi: u64,
        let_hey: u64,
        helper_call: u64,
        assert: u64,
    }

    let test_patterns = [
        TestPattern { 
            let_hi: 68,      // test_1
            let_hey: 70,
            helper_call: 80,
            assert: 84,
        },
        TestPattern { 
            let_hi: 92,      // test_2
            let_hey: 94,
            helper_call: 104,
            assert: 108,
        },
        TestPattern { 
            let_hi: 116,     // test_3
            let_hey: 118,
            helper_call: 128,
            assert: 132,
        },
    ];

    // Verify each test function follows the pattern
    for (i, pattern) in test_patterns.iter().enumerate() {
        let test_num = i + 1;
        let base_line = 19 + (i as i64 * 8); // Tests are 8 lines apart

        assert_eq!(
            file_map.get(&(base_line + 2)).expect(&format!("test_{} let_hi", test_num)),
            &vec![pattern.let_hi]
        );
        assert_eq!(
            file_map.get(&(base_line + 3)).expect(&format!("test_{} let_hey", test_num)),
            &vec![pattern.let_hey]
        );
        assert_eq!(
            file_map.get(&(base_line + 4)).expect(&format!("test_{} helper_call", test_num)),
            &vec![pattern.helper_call]
        );
        assert_eq!(
            file_map.get(&(base_line + 5)).expect(&format!("test_{} assert", test_num)),
            &vec![pattern.assert]
        );

        // Verify instruction spacing pattern within each test
        if i < test_patterns.len() - 1 {
            let curr = pattern;
            let next = &test_patterns[i + 1];
            
            // Each test starts 24 instructions after the previous one
            assert_eq!(
                next.let_hi - curr.let_hi, 
                24, 
                "Tests should be 24 instructions apart"
            );
        }
    }
}

#[test]
fn test_compiler_sourcemap() {
    let mut server = DapServer::new(
        Box::new(std::io::stdin()), 
        Box::new(std::io::sink())
    );

    let program_path = test_fixtures_dir().join("simple/src/main.sw");
    let source_str = program_path.to_string_lossy().to_string();

    // Initialize and build
    server.handle_command(&Command::Initialize(Default::default()));
    server.state.program_path = PathBuf::from(&source_str);
    server.state.mode = Some(StartDebuggingRequestKind::Launch);
    server.build_tests().expect("Failed to build tests");

    // Examine the source map
    let source_map = &server.state.compiler_source_map.unwrap();
    
    println!("\nSource map contents:");
    println!("Paths:");
    for path in &source_map.paths {
        println!("  {}", path.display());
    }
    
    println!("\nDependency paths:");
    for path in &source_map.dependency_paths {
        println!("  {}", path.display());
    }
    
    println!("\nMapping entries:");
    for (pc, span) in &source_map.map {
        if let Some((path, range)) = source_map.addr_to_span(*pc) {
            println!("  Instruction {}: {}:{}:{}-{}:{}", 
                    pc,
                    path.display(),
                    range.start.line,
                    range.start.col,
                    range.end.line,
                    range.end.col);
        }
    }
}

#[test]
fn compare_original_sourcemap() {
    let mut server = DapServer::new(
        Box::new(std::io::stdin()), 
        Box::new(std::io::sink())
    );

    let program_path = test_fixtures_dir().join("simple/src/main.sw");
    let source_str = program_path.to_string_lossy().to_string();

    // Initialize and build
    server.handle_command(&Command::Initialize(Default::default()));
    server.state.program_path = PathBuf::from(&source_str);
    server.state.mode = Some(StartDebuggingRequestKind::Launch);
    server.build_tests().expect("Failed to build tests");

    // Looking at what we had in our original output:
    println!("\nOriginal source map output showed:");
    println!("Line  3: Instructions [159, 160, 163, 164]");
    println!("Line  4: Instructions [166, 167]");
    println!("Line  5: Instructions [170]");
    println!("Line  6: Instructions [175]");
    println!("Line 11: Instructions [262, 263, 266, 267]");
    println!("Line 12: Instructions [269, 270]");
    println!("Line 13: Instructions [273]");
    println!("Line 14: Instructions [278]");
    println!("Line 21: Instructions [68]");
    println!("Line 22: Instructions [70]");
    println!("Line 23: Instructions [80]");
    println!("Line 24: Instructions [84]");

    println!("\nNew compiler source map shows:");
    // Group instructions by line number for comparison
    let mut line_to_instructions: BTreeMap<i64, Vec<usize>> = BTreeMap::new();
    
    let source_map = &server.state.compiler_source_map.unwrap();
    for (pc, span) in source_map.clone().map {
        if let Some((path, range)) = source_map.addr_to_span(pc) {
            if path == program_path {
                line_to_instructions
                    .entry(range.start.line as i64)
                    .or_default()
                    .push(pc);
            }
        }
    }

    // Print in same format as original for comparison
    for (line, instructions) in &line_to_instructions {
        println!("Line {:2}: Instructions {:?}", line, instructions);
    }
}