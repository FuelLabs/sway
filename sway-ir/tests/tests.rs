use std::path::PathBuf;

use itertools::Itertools;
use sway_features::ExperimentalFeatures;
use sway_ir::{
    create_arg_demotion_pass, create_ccp_pass, create_const_demotion_pass,
    create_const_folding_pass, create_cse_pass, create_dce_pass, create_dom_fronts_pass,
    create_dominators_pass, create_escaped_symbols_pass, create_mem2reg_pass,
    create_memcpyopt_pass, create_misc_demotion_pass, create_postorder_pass,
    create_ret_demotion_pass, create_simplify_cfg_pass, metadata_to_inline, optimize as opt,
    register_known_passes, Context, Function, IrError, PassGroup, PassManager, Value, DCE_NAME,
    FN_DEDUP_DEBUG_PROFILE_NAME, FN_DEDUP_RELEASE_PROFILE_NAME, GLOBALS_DCE_NAME, MEM2REG_NAME,
    SROA_NAME,
};
use sway_types::SourceEngine;

// -------------------------------------------------------------------------------------------------
// Utility for finding test files and running FileCheck.  See actual pass invocations below.

fn run_tests<F: Fn(&str, &mut Context) -> bool>(sub_dir: &str, opt_fn: F) {
    let source_engine = SourceEngine::default();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let dir: PathBuf = format!("{manifest_dir}/tests/{sub_dir}").into();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();

        let input_bytes = std::fs::read(&path).unwrap();
        let input = String::from_utf8_lossy(&input_bytes);

        let experimental = ExperimentalFeatures {
            new_encoding: false,
            // TODO: Properly support experimental features in IR tests.
            ..Default::default()
        };

        let mut ir = sway_ir::parser::parse(&input, &source_engine, experimental).unwrap_or_else(
            |parse_err| {
                println!("{}: {parse_err}", path.display());
                panic!()
            },
        );

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

// Utility for finding test files and running IR verifier tests.
// Each test file must contain an IR code that is parsable,
// but does not pass IR verification.
// Each test file must contain exactly one `// error: ...` line
// that specifies the expected IR verification error.
fn run_ir_verifier_tests(sub_dir: &str) {
    let source_engine = SourceEngine::default();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let dir: PathBuf = format!("{manifest_dir}/tests/{sub_dir}").into();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();

        let input_bytes = std::fs::read(&path).unwrap();
        let input = String::from_utf8_lossy(&input_bytes);

        let expected_errors = input
            .lines()
            .filter(|line| line.starts_with("// error: "))
            .collect_vec();

        let expected_error = match expected_errors[..] {
            [] => {
                println!(
                    "--- IR verifier test does not contain the expected error: {}",
                    path.display()
                );
                println!("The expected error must be specified by using the `// error: ` comment.");
                println!("E.g., `// error: This is the expected error`");
                println!("There must be exactly one error specified in each IR verifier test.");
                panic!();
            }
            [err] => err.replace("// error: ", ""),
            _ => {
                println!(
                    "--- IR verifier test contains more then one expected error: {}",
                    path.display()
                );
                println!(
                    "There must be exactly one expected error specified in each IR verifier test."
                );
                println!("The specified expected errors were:");
                println!("{}", expected_errors.join("\n"));
                panic!();
            }
        };

        let parse_result =
            sway_ir::parser::parse(&input, &source_engine, ExperimentalFeatures::default());

        match parse_result {
            Ok(_) => {
                println!(
                    "--- Parsing and validating an IR verifier test passed without errors: {}",
                    path.display()
                );
                println!("The expected IR validation error was: {expected_error}");
                panic!();
            }
            Err(err @ IrError::ParseFailure(_, _)) => {
                println!(
                    "--- Parsing of an IR verifier test failed: {}",
                    path.display()
                );
                println!(
                    "IR verifier test must be parsable and result in an IR verification error."
                );
                println!("The parsing error was: {err}");
                panic!();
            }
            Err(err) => {
                let err = format!("{err}");
                if !err.contains(&expected_error) {
                    println!("--- IR verifier test failed: {}", path.display());
                    println!("The expected error was: {expected_error}");
                    println!("The actual IR verification error was: {err}");
                    panic!();
                }
            }
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

        let funcs = ir
            .module_iter()
            .flat_map(|module| module.function_iter(ir))
            .collect::<Vec<_>>();

        if params.iter().any(|&p| p == "all") {
            // Just inline everything, replacing all CALL instructions.
            funcs.into_iter().fold(false, |acc, func| {
                opt::inline_all_function_calls(ir, &func).unwrap() || acc
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

            funcs.into_iter().fold(false, |acc, func| {
                let predicate = |context: &Context, function: &Function, call_site: &Value| {
                    let attributed_inline =
                        metadata_to_inline(context, function.get_metadata(context));
                    match attributed_inline {
                        Some(opt::Inline::Never) => false,
                        Some(opt::Inline::Always) => true,
                        None => (opt::is_small_fn(max_blocks, max_instrs, max_stack))(
                            context, function, call_site,
                        ),
                    }
                };
                opt::inline_some_function_calls(ir, &func, predicate).unwrap() || acc
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
        let mut pass_mgr = PassManager::default();
        let mut pass_group = PassGroup::default();
        let pass = pass_mgr.register(create_const_folding_pass());
        pass_group.append_pass(pass);
        pass_mgr.run(ir, &pass_group).unwrap()
    })
}

// -------------------------------------------------------------------------------------------------

#[allow(clippy::needless_collect)]
#[test]
fn ccp() {
    run_tests("ccp", |_first_line, ir: &mut Context| {
        let mut pass_mgr = PassManager::default();
        let mut pass_group = PassGroup::default();
        pass_mgr.register(create_postorder_pass());
        pass_mgr.register(create_dominators_pass());
        let pass = pass_mgr.register(create_ccp_pass());
        pass_group.append_pass(pass);
        pass_mgr.run(ir, &pass_group).unwrap()
    })
}

// -------------------------------------------------------------------------------------------------

#[allow(clippy::needless_collect)]
#[test]
fn simplify_cfg() {
    run_tests("simplify_cfg", |_first_line, ir: &mut Context| {
        let mut pass_mgr = PassManager::default();
        let mut pass_group = PassGroup::default();
        let pass = pass_mgr.register(create_simplify_cfg_pass());
        pass_group.append_pass(pass);
        pass_mgr.run(ir, &pass_group).unwrap()
    })
}

// -------------------------------------------------------------------------------------------------

#[allow(clippy::needless_collect)]
#[test]
fn dce() {
    run_tests("dce", |_first_line, ir: &mut Context| {
        let mut pass_mgr = PassManager::default();
        let mut pass_group = PassGroup::default();
        pass_mgr.register(create_escaped_symbols_pass());
        let pass = pass_mgr.register(create_dce_pass());
        pass_group.append_pass(pass);
        // TODO: needed by dce_dead_constant_assignment test. Improve DCE and remove this.
        // https://github.com/FuelLabs/sway/issues/7262
        pass_group.append_pass(pass);
        pass_mgr.run(ir, &pass_group).unwrap()
    })
}

// -------------------------------------------------------------------------------------------------

#[allow(clippy::needless_collect)]
#[test]
fn cse() {
    run_tests("cse", |_first_line, ir: &mut Context| {
        let mut pass_mgr = PassManager::default();
        let mut pass_group = PassGroup::default();
        pass_mgr.register(create_postorder_pass());
        pass_mgr.register(create_dominators_pass());
        let pass = pass_mgr.register(create_cse_pass());
        pass_group.append_pass(pass);
        pass_mgr.run(ir, &pass_group).unwrap()
    })
}

// -------------------------------------------------------------------------------------------------

#[allow(clippy::needless_collect)]
#[test]
fn mem2reg() {
    run_tests("mem2reg", |_first_line, ir: &mut Context| {
        let mut pass_mgr = PassManager::default();
        let mut pass_group = PassGroup::default();
        pass_mgr.register(create_postorder_pass());
        pass_mgr.register(create_dominators_pass());
        pass_mgr.register(create_dom_fronts_pass());
        let pass = pass_mgr.register(create_mem2reg_pass());
        pass_group.append_pass(pass);
        pass_mgr.run(ir, &pass_group).unwrap()
    })
}

// -------------------------------------------------------------------------------------------------

#[allow(clippy::needless_collect)]
#[test]
fn demote_arg() {
    run_tests("demote_arg", |_first_line, ir: &mut Context| {
        let mut pass_mgr = PassManager::default();
        let mut pass_group = PassGroup::default();
        let pass = pass_mgr.register(create_arg_demotion_pass());
        pass_group.append_pass(pass);
        pass_mgr.run(ir, &pass_group).unwrap()
    })
}

// -------------------------------------------------------------------------------------------------

#[allow(clippy::needless_collect)]
#[test]
fn demote_const() {
    run_tests("demote_const", |_first_line, ir: &mut Context| {
        let mut pass_mgr = PassManager::default();
        let mut pass_group = PassGroup::default();
        let pass = pass_mgr.register(create_const_demotion_pass());
        pass_group.append_pass(pass);
        pass_mgr.run(ir, &pass_group).unwrap()
    })
}

// -------------------------------------------------------------------------------------------------

#[allow(clippy::needless_collect)]
#[test]
fn demote_ret() {
    run_tests("demote_ret", |_first_line, ir: &mut Context| {
        let mut pass_mgr = PassManager::default();
        let mut pass_group = PassGroup::default();
        let pass = pass_mgr.register(create_ret_demotion_pass());
        pass_group.append_pass(pass);
        pass_mgr.run(ir, &pass_group).unwrap()
    })
}

// -------------------------------------------------------------------------------------------------

#[allow(clippy::needless_collect)]
#[test]
fn demote_misc() {
    run_tests("demote_misc", |_first_line, ir: &mut Context| {
        let mut pass_mgr = PassManager::default();
        let mut pass_group = PassGroup::default();
        let pass = pass_mgr.register(create_misc_demotion_pass());
        pass_group.append_pass(pass);
        pass_mgr.run(ir, &pass_group).unwrap()
    })
}

// -------------------------------------------------------------------------------------------------

#[allow(clippy::needless_collect)]
#[test]
fn memcpyopt() {
    run_tests("memcpyopt", |_first_line, ir: &mut Context| {
        let mut pass_mgr = PassManager::default();
        let mut pass_group = PassGroup::default();
        pass_mgr.register(create_escaped_symbols_pass());
        let pass = pass_mgr.register(create_memcpyopt_pass());
        pass_group.append_pass(pass);
        pass_mgr.run(ir, &pass_group).unwrap()
    })
}

// -------------------------------------------------------------------------------------------------

#[allow(clippy::needless_collect)]
#[test]
fn sroa() {
    run_tests("sroa", |_first_line, ir: &mut Context| {
        let mut pass_mgr = PassManager::default();
        let mut pass_group = PassGroup::default();
        register_known_passes(&mut pass_mgr);
        pass_group.append_pass(SROA_NAME);
        pass_group.append_pass(MEM2REG_NAME);
        pass_group.append_pass(DCE_NAME);
        pass_mgr.run(ir, &pass_group).unwrap()
    })
}

// -------------------------------------------------------------------------------------------------

#[allow(clippy::needless_collect)]
#[test]
fn globals_dce() {
    run_tests("globals_dce", |_first_line, ir: &mut Context| {
        let mut pass_mgr = PassManager::default();
        let mut pass_group = PassGroup::default();
        register_known_passes(&mut pass_mgr);
        pass_group.append_pass(GLOBALS_DCE_NAME);
        pass_mgr.run(ir, &pass_group).unwrap()
    })
}

// -------------------------------------------------------------------------------------------------

#[allow(clippy::needless_collect)]
#[test]
fn fndedup_debug() {
    run_tests("fn_dedup/debug", |_first_line, ir: &mut Context| {
        let mut pass_mgr = PassManager::default();
        let mut pass_group = PassGroup::default();
        register_known_passes(&mut pass_mgr);
        pass_group.append_pass(FN_DEDUP_DEBUG_PROFILE_NAME);
        pass_group.append_pass(GLOBALS_DCE_NAME);
        pass_mgr.run(ir, &pass_group).unwrap()
    })
}

#[allow(clippy::needless_collect)]
#[test]
fn fndedup_release() {
    run_tests("fn_dedup/release", |_first_line, ir: &mut Context| {
        let mut pass_mgr = PassManager::default();
        let mut pass_group = PassGroup::default();
        register_known_passes(&mut pass_mgr);
        pass_group.append_pass(FN_DEDUP_RELEASE_PROFILE_NAME);
        pass_group.append_pass(GLOBALS_DCE_NAME);
        pass_mgr.run(ir, &pass_group).unwrap()
    })
}

#[test]
fn verify() {
    run_ir_verifier_tests("verify")
}

// -------------------------------------------------------------------------------------------------
#[test]
fn serialize() {
    // This isn't running a pass, it's just confirming that the IR can be loaded and printed, and
    // FileCheck can just confirm certain instructions came out OK.
    run_tests("serialize", |_, _: &mut Context| true)
}

// -------------------------------------------------------------------------------------------------
