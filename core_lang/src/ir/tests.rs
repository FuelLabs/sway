use std::path::PathBuf;

use crate::{
    control_flow_analysis::{ControlFlowGraph, Graph},
    parser::{HllParser, Rule},
    semantic_analysis::{TreeType, TypedParseTree},
};
use pest::Parser;

use super::parser;
use super::printer;

// -------------------------------------------------------------------------------------------------

#[test]
fn sway_to_ir_tests() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let dir: PathBuf = format!("{}/tests/sway_to_ir", manifest_dir).into();
    for entry in std::fs::read_dir(dir).unwrap() {
        // We're only interested in the `.sw` files here.
        let path = entry.unwrap().path();
        match path.extension().unwrap().to_str() {
            Some("sw") => {
                //
                // Run the tests!
                //
                test_sway_to_ir(path);
            }
            Some("ir") => (),
            _ => panic!(
                "File with invalid extension in tests dir: {:?}",
                path.file_name().unwrap_or(path.as_os_str())
            ),
        }
    }
}

fn test_sway_to_ir(mut path: PathBuf) {
    let input_bytes = std::fs::read(&path).unwrap();
    let input = String::from_utf8_lossy(&input_bytes);

    path.set_extension("ir");

    let expected_bytes = std::fs::read(&path).unwrap();
    let expected = String::from_utf8_lossy(&expected_bytes);

    let typed_ast = parse_to_typed_ast(&input);
    let ir = super::compile_ast(typed_ast).unwrap();
    let output = printer::to_string(&ir);

    if output != expected {
        println!("{}", prettydiff::diff_lines(&expected, &output));
    }
    assert_eq!(output, expected);
}

// -------------------------------------------------------------------------------------------------

#[test]
fn ir_printer_parser_tests() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let dir: PathBuf = format!("{}/tests/sway_to_ir", manifest_dir).into();
    for entry in std::fs::read_dir(dir).unwrap() {
        // We're only interested in the `.ir` files here.
        let path = entry.unwrap().path();
        match path.extension().unwrap().to_str() {
            Some("ir") => {
                //
                // Run the tests!
                //
                test_printer_parser(path);
            }
            Some("sw") => (),
            _ => panic!(
                "File with invalid extension in tests dir: {:?}",
                path.file_name().unwrap_or(path.as_os_str())
            ),
        }
    }
}

fn test_printer_parser(path: PathBuf) {
    let input_bytes = std::fs::read(&path).unwrap();
    let input = String::from_utf8_lossy(&input_bytes);

    let parsed_ctx = match parser::parse(&input) {
        Ok(p) => p,
        Err(e) => {
            println!("{}: {}", path.display(), e);
            panic!();
        }
    };
    let printed = printer::to_string(&parsed_ctx);
    if printed != input {
        println!("{}", prettydiff::diff_lines(&input, &printed));
    }
    assert_eq!(input, printed);
}

// -------------------------------------------------------------------------------------------------

#[test]
fn ir_to_ir_tests() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let dir: PathBuf = format!("{}/tests/ir_to_ir", manifest_dir).into();
    for entry in std::fs::read_dir(dir).unwrap() {
        // We're only interested in the `.in_ir` files here.
        let path = entry.unwrap().path();
        match path.extension().unwrap().to_str() {
            Some("in_ir") => {
                //
                // Run the tests!  We're only testing the inliner at this stage.  Eventually we'll
                // test other transforms which will be specified by either the test file name or
                // perhaps a comment within.
                //
                test_inline(path);
            }
            Some("out_ir") => (),
            _ => panic!(
                "File with invalid extension in tests dir: {:?}",
                path.file_name().unwrap_or(path.as_os_str())
            ),
        }
    }
}

fn test_inline(mut path: PathBuf) {
    let input_bytes = std::fs::read(&path).unwrap();
    let input = String::from_utf8_lossy(&input_bytes);

    path.set_extension("out_ir");

    let expected_bytes = std::fs::read(&path).unwrap();
    let expected = String::from_utf8_lossy(&expected_bytes);

    let mut ir = parser::parse(&input).unwrap();
    let main_fn = ir
        .functions
        .iter()
        .find_map(|(idx, fc)| if fc.name == "main" { Some(idx) } else { None })
        .unwrap();
    super::optimise::inline_all_function_calls(&mut ir, &super::function::Function(main_fn))
        .unwrap();
    let output = printer::to_string(&ir);

    if output != expected {
        println!("{}", prettydiff::diff_lines(&expected, &output));
    }
    assert_eq!(output, expected);
}

// -------------------------------------------------------------------------------------------------

fn parse_to_typed_ast(input: &str) -> TypedParseTree {
    let mut parsed = HllParser::parse(Rule::program, input).expect("parse_tree");

    let mut warnings = vec![];
    let mut errors = vec![];
    let parse_tree = crate::parse_root_from_pairs(parsed.next().unwrap().into_inner(), None)
        .unwrap(&mut warnings, &mut errors);

    let mut dead_code_graph = ControlFlowGraph {
        graph: Graph::new(),
        entry_points: vec![],
        namespace: Default::default(),
    };
    let build_config = crate::build_config::BuildConfig {
        file_name: std::sync::Arc::new("test.sw".into()),
        dir_of_code: std::sync::Arc::new("tests".into()),
        manifest_path: std::sync::Arc::new(".".into()),
        print_intermediate_asm: false,
        print_finalized_asm: false,
    };
    TypedParseTree::type_check(
        parse_tree.script_ast.expect("script_ast"),
        Default::default(),
        TreeType::Script,
        &build_config,
        &mut dead_code_graph,
        &mut std::collections::HashMap::new(),
    )
    .unwrap(&mut warnings, &mut errors)
}

// -------------------------------------------------------------------------------------------------
