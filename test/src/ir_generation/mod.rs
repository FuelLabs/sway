use std::{
    fs,
    ops::Not,
    path::{Path, PathBuf},
};

use anyhow::Result;
use colored::Colorize;
use sway_core::{
    compile_ir_context_to_finalized_asm, compile_to_ast,
    ir_generation::compile_program,
    namespace::{self, Package},
    BuildBackend, BuildConfig, BuildTarget, Engines, OptLevel, PanicOccurrences,
    PanickingCallOccurrences,
};
use sway_error::handler::Handler;

use sway_features::ExperimentalFeatures;
use sway_ir::{
    create_fn_inline_pass, register_known_passes, Backtrace, PassGroup, PassManager,
    ARG_DEMOTION_NAME, CONST_DEMOTION_NAME, DCE_NAME, MEMCPYOPT_NAME, MISC_DEMOTION_NAME,
    RET_DEMOTION_NAME,
};
use sway_types::ProgramId;

use crate::RunConfig;

enum Checker {
    Ir,
    Asm,
    OptimizedIr { passes: Vec<String> },
}

impl Checker {
    /// Builds and configures checkers based on file comments. Every check between checkers directive
    /// are collected into the last started checker, "::check-ir::" being the default at the start
    /// of the file.
    /// Example:
    ///
    /// ```sway
    /// // ::check-ir::
    /// // ::check-ir-optimized::
    /// // ::check-ir-asm::
    /// ```
    ///
    /// # ::check-ir-optimized::
    ///
    /// Optimized IR checker can be configured with `pass: <PASSNAME or o1>`. When
    /// `o1` is chosen, all the configured passes are chosen automatically.
    ///
    /// ```sway
    /// // ::check-ir-optimized::
    /// // pass: o1
    /// ```
    pub fn new(input: impl AsRef<str>) -> Vec<(Checker, Option<filecheck::Checker>)> {
        let input = input.as_ref();

        let mut checkers: Vec<(Checker, String)> = vec![(Checker::Ir, "".to_string())];

        for line in input.lines() {
            if line.contains("::check-ir::") && !matches!(checkers.last(), Some((Checker::Ir, _))) {
                checkers.push((Checker::Ir, "".to_string()));
            }

            if line.contains("::check-asm::") {
                checkers.push((Checker::Asm, "".to_string()));
            }

            if line.contains("::check-ir-optimized::") {
                checkers.push((Checker::OptimizedIr { passes: vec![] }, "".to_string()));
            }

            if let Some(pass) = line.strip_prefix("// pass: ") {
                if let Some((Checker::OptimizedIr { passes }, _)) = checkers.last_mut() {
                    passes.push(pass.trim().to_string());
                }
            }

            if line.starts_with("//") {
                let s = checkers.last_mut().unwrap();
                s.1.push_str(line);
                s.1.push('\n');
            }
        }

        let mut new_checkers = vec![];

        for (k, v) in checkers {
            let ir_checker = filecheck::CheckerBuilder::new()
                .text(&v)
                .unwrap()
                .finish()
                .is_empty()
                .not()
                .then(|| {
                    filecheck::CheckerBuilder::new()
                        .text(
                            "regex: VAL=\\bv\\d+\\b\n\
                         regex: ID=[_[:alpha:]][_0-9[:alpha:]]*\n\
                         regex: MD=!\\d+\n",
                        )
                        .unwrap()
                        .text(&v)
                        .unwrap()
                        .finish()
                });
            new_checkers.push((k, ir_checker));
        }

        new_checkers
    }
}

/// Will print `filecheck` report using colors: normal lines will be dimmed,
/// matches will be green and misses will be red.
fn pretty_print_error_report(error: &str) {
    let mut stash = vec![];

    let mut lines = error.lines().peekable();
    while let Some(current) = lines.next() {
        if current.starts_with("> ") {
            match lines.peek() {
                Some(next) if next.contains("^~") => {
                    stash.push(current);
                }
                _ => println!("{}", current.bright_black()),
            }
        } else if current.starts_with("Matched") && current.contains("not: ") {
            for line in stash.drain(..) {
                if line.contains("^~") {
                    println!("{}", line.red())
                } else {
                    println!("{}", line.bold())
                }
            }
            println!("{}", current.red())
        } else if current.starts_with("Matched") {
            for line in stash.drain(..) {
                if line.contains("^~") {
                    println!("{}", line.green())
                } else {
                    println!("{}", line.bold())
                }
            }
            println!("{current}")
        } else if current.starts_with("Define") {
            println!("{current}")
        } else if current.starts_with("Missed") && current.contains("check: ") {
            for line in stash.drain(..) {
                if line.contains("^~") {
                    println!("{}", line.red())
                } else {
                    println!("{}", line.bold())
                }
            }
            println!("{}", current.red())
        } else if current.starts_with("Missed") && current.contains("not: ") {
            for line in stash.drain(..) {
                if line.contains("^~") {
                    println!("{}", line.green())
                } else {
                    println!("{}", line.bold())
                }
            }
            println!("{current}")
        } else {
            stash.push(current);
        }
    }
}

pub(super) async fn run(
    filter_regex: Option<&regex::Regex>,
    verbose: bool,
    run_config: &RunConfig,
) -> Result<()> {
    // Create new initial namespace for every test by reusing the precompiled
    // standard libraries. The namespace, thus its root module, must have the
    // name set.
    const PACKAGE_NAME: &str = "test_lib";
    let core_lib_name = sway_types::Ident::new_no_span(PACKAGE_NAME.to_string());

    // Compile std library and reuse it when compiling tests.
    let engines = Engines::default();
    let build_target = BuildTarget::default();
    let std_package = compile_std(build_target, &engines, run_config);

    // Find all the tests.
    let all_tests = discover_test_files();
    let total_test_count = all_tests.len();
    let mut run_test_count = 0;
    all_tests
        .into_iter()
        .filter_map(|path|  {
            // Filter against the regex.
            if path.to_str()
                .and_then(|path_str| filter_regex.map(|regex| regex.is_match(path_str)))
                .unwrap_or(true)  {
                // Read entire file.
                let input_bytes = fs::read(&path).expect("Read entire Sway source.");
                let input = String::from_utf8_lossy(&input_bytes);

                let checkers = Checker::new(&input);

                let mut optimisation_inline = false;
                let mut target_fuelvm = false;

                if let Some(first_line) = input.lines().next() {
                    optimisation_inline = first_line.contains("optimisation-inline");
                    target_fuelvm = first_line.contains("target-fuelvm");
                }

                Some((
                    path,
                    input_bytes,
                    checkers,
                    optimisation_inline,
                    target_fuelvm,
                ))
            } else {
                None
            }
        })
        .for_each(
            |(path, sway_str, checkers, optimisation_inline, target_fuelvm)| {
                let test_file_name = path.file_name().unwrap().to_string_lossy().to_string();
                tracing::info!("Testing {} ...", test_file_name.bold());

                let experimental = ExperimentalFeatures {
                    new_encoding: false, // IR tests still need encoding v1 off.
                    // TODO: Properly support experimental features in IR tests.
                    ..Default::default()
                };

                // TODO: Properly support backtrace build option in IR tests.
                let backtrace = Backtrace::default();

                // Compile to AST.  We need to provide a faux build config otherwise the IR will have
                // no span metadata.
                let bld_cfg = sway_core::BuildConfig::root_from_file_name_and_manifest_path(
                    path.clone(),
                    PathBuf::from("/"),
                    build_target,
                    sway_core::DbgGeneration::Full,
                );

                // Include unit tests in the build.
                let bld_cfg = bld_cfg.with_include_tests(true);

                let sway_str = String::from_utf8_lossy(&sway_str);
                let handler = Handler::default();
                let mut initial_namespace = Package::new(core_lib_name.clone(), None, ProgramId::new(0), false);
                initial_namespace.add_external("std".to_owned(), std_package.clone());
                let compile_res = compile_to_ast(
                    &handler,
                    &engines,
                    sway_str.as_ref().into(),
                    initial_namespace,
                    Some(&bld_cfg),
                    PACKAGE_NAME,
                    None,
                    experimental
                );
                let (errors, _warnings, _infos) = handler.consume();
                if !errors.is_empty() {
                    panic!(
                        "Failed to compile test {}:\n{}",
                        path.display(),
                        errors
                            .iter()
                            .map(|err| err.to_string())
                            .collect::<Vec<_>>()
                            .as_slice()
                            .join("\n")
                    );
                }
                let programs = compile_res
                    .expect("there were no errors, so there should be a program");

                if verbose {
                    println!("Declaration Engine");
                    println!("-----------------------");
                    println!("{}", engines.de().pretty_print(&engines));
                }

                let typed_program = programs.typed.as_ref().unwrap();

                // Compile to IR.
                let include_tests = true;
                let mut panic_occurrences = PanicOccurrences::default();
                let mut panicking_call_occurrences = PanickingCallOccurrences::default();
                let mut ir = compile_program(typed_program, &mut panic_occurrences, &mut panicking_call_occurrences, include_tests, &engines, experimental, backtrace)
                    .unwrap_or_else(|e| {
                        use sway_types::span::Spanned;
                        let e = e[0].clone();
                        let span = e.span();
                        panic!(
                            "Failed to compile test {}:\nError \"{e}\" at {}:{}\nCode: \"{}\"",
                            path.display(),
                            span.start(),
                            span.end(),
                            span.as_str()
                        );
                    });
                    ir.verify()
                    .unwrap_or_else(|err| {
                        panic!("IR verification failed for test {}:\n{err}", path.display());
                    });

                // Perform Fuel target specific passes if requested.
                if target_fuelvm {
                    // Manually run the FuelVM target passes.  This will be encapsulated into an
                    // official `PassGroup` eventually.
                    let mut pass_mgr = PassManager::default();
                    let mut pass_group = PassGroup::default();
                    register_known_passes(&mut pass_mgr);
                    pass_group.append_pass(CONST_DEMOTION_NAME);
                    pass_group.append_pass(ARG_DEMOTION_NAME);
                    pass_group.append_pass(RET_DEMOTION_NAME);
                    pass_group.append_pass(MISC_DEMOTION_NAME);
                    pass_group.append_pass(MEMCPYOPT_NAME);
                    pass_group.append_pass(DCE_NAME);
                    if pass_mgr.run(&mut ir, &pass_group).is_err() {
                        panic!(
                            "Failed to compile test {}:\n{}",
                            path.display(),
                                errors
                                .iter()
                                .map(|err| err.to_string())
                                .collect::<Vec<_>>()
                                .as_slice()
                                .join("\n")
                        );
                    }
                }

                let ir_output = sway_ir::printer::to_string(&ir);

                for (k, checker) in checkers {
                    match (k, checker) {
                        (Checker::Ir, Some(checker)) => {
                            match checker.explain(&ir_output, filecheck::NO_VARIABLES)
                            {
                                Ok((success, error)) if !success || verbose => {
                                    if !success || verbose {
                                        println!("{}", "::check-ir::".bold());
                                        pretty_print_error_report(&error);
                                    }
                                    if !success {
                                        panic!("check-ir filecheck failed. See above.");
                                    }
                                }
                                Err(e) => {
                                    panic!("check-ir filecheck directive error: {e}");
                                }
                                _ => (),
                            };
                        }
                        (Checker::Ir, None) => {
                            panic!(
                                "IR test for {test_file_name} is missing mandatory FileCheck directives.\n\n\
                                Here's the IR output:\n{ir_output}",
                            );
                        }
                        (Checker::OptimizedIr { passes }, Some(checker)) => {
                            if passes.is_empty() {
                                panic!("No optimization passes were specified for ::check-ir-optimized::. Use `// pass: <PASSNAME>` in the very next line.");
                            }

                            let mut group = PassGroup::default();
                            for pass in passes {
                                if pass == "o1" {
                                    group = sway_ir::create_o1_pass_group();
                                } else {
                                    // pass needs a 'static str
                                    let pass = Box::leak(Box::new(pass));
                                    group.append_pass(pass.as_str());
                                }
                            }

                            let mut pass_mgr = PassManager::default();
                            register_known_passes(&mut pass_mgr);

                            // Parse the IR again avoiding mutating the original ir
                            let mut ir = sway_ir::parser::parse(
                                &ir_output,
                                 engines.se(),
                                 experimental,
                                 backtrace,
                                )
                                .unwrap_or_else(|e| panic!("{}: {e}\n{ir_output}", path.display()));

                            let _ = pass_mgr.run(&mut ir, &group);
                            let ir_output = sway_ir::printer::to_string(&ir);

                            match checker.explain(&ir_output, filecheck::NO_VARIABLES)
                            {
                                Ok((success, error)) if !success || verbose  => {
                                    if !success || verbose {
                                        println!("{}", "::check-ir-optimized::".bold());
                                        pretty_print_error_report(&error);
                                    }
                                    if !success {
                                        panic!("check-ir-optimized filecheck failed. See above.");
                                    }
                                }
                                Err(e) => {
                                    panic!("check-ir-optimized filecheck directive error: {e}");
                                }
                                _ => (),
                            };
                        }
                        (Checker::Asm, Some(checker)) => {
                            if optimisation_inline {
                                let mut pass_mgr = PassManager::default();
                                let mut pmgr_config = PassGroup::default();
                                let inline = pass_mgr.register(create_fn_inline_pass());
                                pmgr_config.append_pass(inline);
                                let inline_res = pass_mgr.run(&mut ir, &pmgr_config);
                                if inline_res.is_err() {
                                    panic!(
                                        "Failed to compile test {}:\n{}",
                                        path.display(),
                                            errors
                                            .iter()
                                            .map(|err| err.to_string())
                                            .collect::<Vec<_>>()
                                            .as_slice()
                                            .join("\n")
                                    );
                                }
                            }

                            // Compile to ASM.
                            let handler = Handler::default();
                            let asm_result = compile_ir_context_to_finalized_asm(
                                &handler,
                                &ir,
                    Some(&BuildConfig::dummy_for_asm_generation().with_optimization_level(OptLevel::Opt1))
                            );
                            let (errors, _warnings, _infos) = handler.consume();

                            if asm_result.is_err() || !errors.is_empty() {
                                println!("Errors when compiling {test_file_name} IR to ASM:\n");
                                for e in errors {
                                    println!("{e}\n");
                                }
                                panic!();
                            };

                            let asm_output = asm_result
                                .map(|asm| format!("{asm}"))
                                .expect("Failed to stringify ASM for {test_file_name}.");

                            if checker.is_empty() {
                                panic!(
                                    "ASM test for {} has the '::check-asm::' marker \
                                but is missing directives.\n\
                                Please either remove the marker or add some.\n\n\
                                Here's the ASM output:\n{asm_output}",
                                    path.file_name().unwrap().to_string_lossy()
                                );
                            }

                            // Do ASM checks.
                            match checker.explain(&asm_output, filecheck::NO_VARIABLES) {
                                Ok((success, error)) => {
                                    if !success || verbose {
                                        println!("{}", "::check-asm::".bold());
                                        pretty_print_error_report(&error);
                                    }
                                    if !success {
                                        panic!("check-asm filecheck for {test_file_name} failed. See above.");
                                    }
                                }
                                Err(e) => {
                                    panic!("check-asm filecheck directive errors for {test_file_name}: {e}");
                                }
                            };
                        }
                        (_, _) => {
                            todo!("Unknown checker");
                        }
                    }
                }

                // Parse the IR again, and print it yet again to make sure that IR de/serialisation works.
                let parsed_ir = sway_ir::parser::parse(&ir_output, engines.se(), experimental, backtrace)
                    .unwrap_or_else(|e| panic!("{}: {e}\n{ir_output}", path.display()));
                let parsed_ir_output = sway_ir::printer::to_string(&parsed_ir);
                if ir_output != parsed_ir_output {
                    println!("Deserialized IR:");
                    tracing::error!("{}", prettydiff::diff_lines(&ir_output, &parsed_ir_output));
                    panic!("{} failed IR (de)serialization.", path.display());
                }

                run_test_count += 1;
            },
        );

    if run_test_count == 0 {
        tracing::warn!(
            "No IR generation tests were run. Regex filter \"{}\" filtered out all {} tests.",
            filter_regex
                .map(|regex| regex.to_string())
                .unwrap_or_default(),
            total_test_count,
        );
    } else {
        tracing::info!("_________________________________");
        tracing::info!(
            "IR tests result: {}. {} total, {} passed; {} failed; {} disabled",
            "ok".green().bold(),
            total_test_count,
            run_test_count,
            0,
            total_test_count - run_test_count
        );
    }
    // TODO: Make this return an Err once the panics above are converted to an error
    Ok(())
}

fn discover_test_files() -> Vec<PathBuf> {
    fn recursive_search(path: &Path, test_files: &mut Vec<PathBuf>) {
        if path.is_dir() {
            for entry in fs::read_dir(path).unwrap() {
                recursive_search(&entry.unwrap().path(), test_files);
            }
        } else if path.is_file() && path.extension().map(|ext| ext == "sw").unwrap_or(false) {
            test_files.push(path.to_path_buf());
        }
    }

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let tests_root_dir = format!("{manifest_dir}/src/ir_generation/tests");

    let mut test_files = Vec::new();
    recursive_search(&PathBuf::from(tests_root_dir), &mut test_files);
    test_files
}

fn compile_std(
    build_target: BuildTarget,
    engines: &Engines,
    run_config: &RunConfig,
) -> namespace::Package {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let libstd_root_dir = format!("{manifest_dir}/../sway-lib-std");

    let check_cmd = forc::cli::CheckCommand {
        build_target,
        backend: BuildBackend::default(),
        path: Some(libstd_root_dir),
        offline_mode: true,
        terse_mode: true,
        disable_tests: false,
        locked: false,
        ipfs_node: None,
        experimental: run_config.experimental.clone(),
        dump_impls: None,
    };

    let res = match forc::test::forc_check::check(check_cmd, engines) {
        Ok(res) => res,
        Err(err) => {
            panic!("Failed to compile sway-lib-std for IR tests: {err:?}")
        }
    };

    match res.0 {
        Some(typed_program) => typed_program.namespace.current_package_ref().clone(),
        _ => {
            let (errors, _warnings, _infos) = res.1.consume();
            for err in errors {
                println!("{err:?}");
            }
            panic!("Failed to compile sway-lib-std for IR tests.");
        }
    }
}
