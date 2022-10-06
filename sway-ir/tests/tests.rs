use std::path::PathBuf;

use sway_ir::{optimize as opt, Context, Function};

// -------------------------------------------------------------------------------------------------
// Utility for finding test files and running FileCheck.  See actual pass invocations below.

fn run_tests<F: Fn(&str, &mut Context) -> bool>(sub_dir: &str, opt_fn: F) {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let dir: PathBuf = format!("{}/tests/{}", manifest_dir, sub_dir).into();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();

        let input_bytes = std::fs::read(&path).unwrap();
        let input = String::from_utf8_lossy(&input_bytes);

        let mut ir = sway_ir::parser::parse(&input).unwrap_or_else(|parse_err| {
            println!("{}: {parse_err}", path.display());
            panic!()
        });

        let first_line = input.split('\n').next().unwrap();

        // The tests should return true, indicating they made modifications.
        assert!(
            opt_fn(first_line, &mut ir),
            "Pass returned false (no changes made to {}).",
            path.display()
        );
        let ir = ir.verify().unwrap_or_else(|err| {
            println!("{err}");
            panic!();
        });

        let output = sway_ir::printer::to_string(&ir);

        let chkr = filecheck::CheckerBuilder::new()
            .text(&input)
            .unwrap()
            .finish();
        if chkr.is_empty() {
            println!("{output}");
            panic!("No filecheck directives found in test: {}", path.display());
        }

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
    run_tests("inline", |first_line, ir: &mut Context| {
        let mut words = first_line.split(' ').collect::<Vec<_>>();
        let params = if words.is_empty() || words.remove(0) != "//" {
            Vec::new()
        } else {
            words
        };

        let fn_idcs: Vec<_> = ir.functions.iter().map(|func| func.0).collect();

        if params.iter().any(|&p| p == "all") {
            // Just inline everything, replacing all CALL instructions.
            fn_idcs.into_iter().fold(false, |acc, fn_idx| {
                opt::inline_all_function_calls(ir, &Function(fn_idx)).unwrap() || acc
            })
        } else {
            // Get the parameters from the first line.  See the inline/README.md for details.  If
            // there aren't any found then there won't be any constraints and it'll be the
            // equivalent of asking to inline everything.
            let (max_blocks, max_instrs, max_stack) =
                params
                    .windows(2)
                    .fold(
                        (None, None, None),
                        |acc @ (b, i, s), param_and_arg| match param_and_arg[0] {
                            "blocks" => (param_and_arg[1].parse().ok(), i, s),
                            "instrs" => (b, param_and_arg[1].parse().ok(), s),
                            "stack" => (b, i, param_and_arg[1].parse().ok()),
                            _ => acc,
                        },
                    );

            fn_idcs.into_iter().fold(false, |acc, fn_idx| {
                opt::inline_some_function_calls(
                    ir,
                    &Function(fn_idx),
                    opt::is_small_fn(max_blocks, max_instrs, max_stack),
                )
                .unwrap()
                    || acc
            })
        }
    })
}

// -------------------------------------------------------------------------------------------------

// Clippy suggests using the map iterator below directly instead of collecting from it first, but
// if we try that then we have borrowing issues with `ir` which is used within the closure.
#[allow(clippy::needless_collect)]
#[test]
fn constants() {
    run_tests("constants", |_first_line, ir: &mut Context| {
        let fn_idcs: Vec<_> = ir.functions.iter().map(|func| func.0).collect();
        fn_idcs.into_iter().fold(false, |acc, fn_idx| {
            sway_ir::optimize::combine_constants(ir, &sway_ir::function::Function(fn_idx)).unwrap()
                || acc
        })
    })
}

// -------------------------------------------------------------------------------------------------

#[allow(clippy::needless_collect)]
#[test]
fn simplify_cfg() {
    run_tests("simplify_cfg", |_first_line, ir: &mut Context| {
        let fn_idcs: Vec<_> = ir.functions.iter().map(|func| func.0).collect();
        fn_idcs.into_iter().fold(false, |acc, fn_idx| {
            sway_ir::optimize::simplify_cfg(ir, &sway_ir::function::Function(fn_idx)).unwrap()
                || acc
        })
    })
}

// -------------------------------------------------------------------------------------------------

#[allow(clippy::needless_collect)]
#[test]
fn dce() {
    run_tests("dce", |_first_line, ir: &mut Context| {
        let fn_idcs: Vec<_> = ir.functions.iter().map(|func| func.0).collect();
        fn_idcs.into_iter().fold(false, |acc, fn_idx| {
            sway_ir::optimize::dce(ir, &sway_ir::function::Function(fn_idx)).unwrap() || acc
        })
    })
}

// -------------------------------------------------------------------------------------------------

#[test]
fn serialize() {
    // This isn't running a pass, it's just confirming that the IR can be loaded and printed, and
    // FileCheck can just confirm certain instructions came out OK.
    run_tests("serialize", |_, _: &mut Context| true)
}

// -------------------------------------------------------------------------------------------------
