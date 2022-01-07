use std::path::PathBuf;

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

    let mut ir = sway_ir::parser::parse(&input).unwrap();
    let main_fn = ir
        .functions
        .iter()
        .find_map(|(idx, fc)| if fc.name == "main" { Some(idx) } else { None })
        .unwrap();
    sway_ir::optimize::inline_all_function_calls(&mut ir, &sway_ir::function::Function(main_fn))
        .unwrap();
    let output = sway_ir::printer::to_string(&ir);

    if output != expected {
        println!("{}", prettydiff::diff_lines(&expected, &output));
    }
    assert_eq!(output, expected);
}

// -------------------------------------------------------------------------------------------------
