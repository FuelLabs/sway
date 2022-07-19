use std::path::PathBuf;

// -------------------------------------------------------------------------------------------------
// Utility for finding test files and running FileCheck.  See actual pass invocations below.

fn run_tests<F: Fn(&mut sway_ir::Context) -> bool>(sub_dir: &str, opt_fn: F) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let dir: PathBuf = format!("{}/tests/{}", manifest_dir, sub_dir).into();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();

        let input_bytes = std::fs::read(&path).unwrap();
        let input = String::from_utf8_lossy(&input_bytes);

        let chkr = filecheck::CheckerBuilder::new()
            .text(&input)
            .unwrap()
            .finish();
        assert!(
            !chkr.is_empty(),
            "No filecheck directives found in test: {}",
            path.display()
        );

        let mut ir = match sway_ir::parser::parse(&input) {
            Ok(ir) => ir,
            Err(parse_err) => {
                println!("{parse_err}");
                panic!()
            }
        };

        assert!(opt_fn(&mut ir));

        let output = sway_ir::printer::to_string(&ir);

        match chkr.explain(&output, filecheck::NO_VARIABLES) {
            Ok((success, report)) if !success => {
                println!("--- FILECHECK FAILED FOR {}", path.display());
                println!("{report}");
                panic!()
            }
            Err(e) => {
                panic!("filecheck directive error while checking: {e}");
            }
            _ => (),
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[test]
fn inline() {
    run_tests("inline", |ir: &mut sway_ir::Context| {
        let main_fn = ir
            .functions
            .iter()
            .find_map(|(idx, fc)| if fc.name == "main" { Some(idx) } else { None })
            .unwrap();
        sway_ir::optimize::inline_all_function_calls(ir, &sway_ir::function::Function(main_fn))
            .unwrap()
    })
}

// -------------------------------------------------------------------------------------------------

// Clippy suggests using the map iterator below directly instead of collecting from it first, but
// if we try that then we have borrowing issues with `ir` which is used within the closure.
#[allow(clippy::needless_collect)]
#[test]
fn constants() {
    run_tests("constants", |ir: &mut sway_ir::Context| {
        let fn_idcs: Vec<_> = ir.functions.iter().map(|func| func.0).collect();
        fn_idcs.into_iter().all(|fn_idx| {
            sway_ir::optimize::combine_constants(ir, &sway_ir::function::Function(fn_idx)).unwrap()
        })
    })
}

// -------------------------------------------------------------------------------------------------

#[test]
fn serialize() {
    // This isn't running a pass, it's just confirming that the IR can be loaded and printed, and
    // FileCheck can just confirm certain instructions came out OK.
    run_tests("serialize", |_: &mut sway_ir::Context| true)
}

// -------------------------------------------------------------------------------------------------
