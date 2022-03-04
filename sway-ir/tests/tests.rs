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
                // Run the tests!
                //
                // We currently choose a single pass based on the test file name, but eventually we
                // should use a comment within the test file to invoke `FileCheck`.

                println!("--- TESTING: {}", path.display());
                let path_str = path.file_name().unwrap().to_string_lossy();
                if path_str.starts_with("inline") {
                    test_inline(path);
                } else if path_str.starts_with("constants") {
                    test_constants(path);
                } else {
                    panic!(
                        "File which doesn't match valid passes: {:?}",
                        path.file_name().unwrap_or(path.as_os_str())
                    );
                }
            }
            Some("out_ir") => (),
            _ => panic!(
                "File with invalid extension in tests dir: {:?}",
                path.file_name().unwrap_or(path.as_os_str())
            ),
        }
    }
}

// -------------------------------------------------------------------------------------------------

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

fn test_constants(mut path: PathBuf) {
    let input_bytes = std::fs::read(&path).unwrap();
    let input = String::from_utf8_lossy(&input_bytes);

    path.set_extension("out_ir");

    let expected_bytes = std::fs::read(&path).unwrap();
    let expected = String::from_utf8_lossy(&expected_bytes);

    let mut ir = sway_ir::parser::parse(&input).unwrap();

    let fn_idcs: Vec<_> = ir.functions.iter().map(|func| func.0).collect();
    for fn_idx in fn_idcs {
        sway_ir::optimize::combine_constants(&mut ir, &sway_ir::function::Function(fn_idx))
            .unwrap();
    }
    let output = sway_ir::printer::to_string(&ir);

    if output != expected {
        println!("{}", prettydiff::diff_lines(&expected, &output));
    }
    assert_eq!(output, expected);
}

// -------------------------------------------------------------------------------------------------
