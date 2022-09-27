use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use sway_core::{compile_ir_to_asm, compile_to_ast, ir_generation::compile_program, namespace};

pub(super) fn run(filter_regex: Option<&regex::Regex>) {
    // Compile core library and reuse it when compiling tests.
    let core_lib = compile_core();

    // Find all the tests.
    let all_tests = discover_test_files();
    let total_test_count = all_tests.len();
    let mut run_test_count = 0;
    all_tests
        .into_iter()
        .filter(|path| {
            // Filter against the regex.
            path.to_str()
                .and_then(|path_str| filter_regex.map(|regex| regex.is_match(path_str)))
                .unwrap_or(true)
        })
        .map(|path| {
            // Read entire file.
            let input_bytes = fs::read(&path).expect("Read entire Sway source.");
            let input = String::from_utf8_lossy(&input_bytes);

            // Split into Sway, FileCheck of IR, FileCheck of ASM.
            //
            // - Search for the optional boundaries.  If they exist, delimited by special tags,
            // then they mark the boundaries for their checks.  If the IR delimiter is missing then
            // it's assumed to be from the start of the file.  The ASM checks themselves are
            // entirely optional.
            let ir_checks_begin_offs = input.find("::check-ir::").unwrap_or(0);
            let asm_checks_begin_offs = input.find("::check-asm::");

            let ir_checks_end_offs = match asm_checks_begin_offs {
                Some(asm_offs) if asm_offs > ir_checks_begin_offs => asm_offs,
                _otherwise => input.len(),
            };

            // This is slightly convoluted.  We want to build the checker from the text, but also
            // provide some builtin regexes for VAL, ID and MD.  If the checker is empty after
            // parsing the test source then it has no checks which is invalid (and below we
            // helpfully print out the IR so some checks can be authored).  But if we add the
            // regexes first then it can't be empty and there's no other simple way to tell.
            // Ideally we'd be able get it from the result of `CheckerBuilder::text()` or to get a
            // count of the found directives (and check they're greater than 3).
            //
            // So instead it builds a temporary checker, tests if it's empty and sets it to None if
            // so.  Otherwise it's discarded and we build another one with the regexes provided.
            use std::ops::Not;
            let ir_checker = filecheck::CheckerBuilder::new()
                .text(&input[ir_checks_begin_offs..ir_checks_end_offs])
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
                        .text(&input[ir_checks_begin_offs..ir_checks_end_offs])
                        .unwrap()
                        .finish()
                });

            let asm_checker = asm_checks_begin_offs.map(|begin_offs| {
                let end_offs = if ir_checks_begin_offs > begin_offs {
                    ir_checks_begin_offs
                } else {
                    input.len()
                };
                filecheck::CheckerBuilder::new()
                    .text(&input[begin_offs..end_offs])
                    .unwrap()
                    .finish()
            });

            (path, input_bytes, ir_checker, asm_checker)
        })
        .for_each(|(path, sway_str, ir_checker, opt_asm_checker)| {
            let test_file_name = path.file_name().unwrap().to_string_lossy();
            tracing::info!("* {test_file_name}");

            // Compile to AST.  We need to provide a faux build config otherwise the IR will have
            // no span metdata.
            let bld_cfg = sway_core::BuildConfig::root_from_file_name_and_manifest_path(
                path.clone(),
                PathBuf::from("/"),
            );
            let sway_str = String::from_utf8_lossy(&sway_str);
            let typed_res = compile_to_ast(Arc::from(sway_str), core_lib.clone(), Some(&bld_cfg));
            if !typed_res.errors.is_empty() {
                panic!(
                    "Failed to compile test {}:\n{}",
                    path.display(),
                    typed_res
                        .errors
                        .iter()
                        .map(|err| err.to_string())
                        .collect::<Vec<_>>()
                        .as_slice()
                        .join("\n")
                );
            }
            let typed_program = typed_res
                .value
                .expect("there were no errors, so there should be a program");

            // Compile to IR.
            let mut ir = compile_program(typed_program)
                .unwrap_or_else(|e| {
                    panic!("Failed to compile test {}:\n{e}", path.display());
                })
                .verify()
                .unwrap_or_else(|err| {
                    panic!("IR verification failed for test {}:\n{err}", path.display());
                });
            let ir_output = sway_ir::printer::to_string(&ir);

            if ir_checker.is_none() {
                panic!(
                    "IR test for {test_file_name} is missing mandatory FileCheck directives.\n\n\
                    Here's the IR output:\n{ir_output}",
                );
            }

            // Do IR checks.
            match ir_checker
                .unwrap()
                .explain(&ir_output, filecheck::NO_VARIABLES)
            {
                Ok((success, report)) if !success => {
                    panic!("IR filecheck failed:\n{report}");
                }
                Err(e) => {
                    panic!("IR filecheck directive error: {e}");
                }
                _ => (),
            };

            if let Some(asm_checker) = opt_asm_checker {
                // Compile to ASM. Need to inline function calls beforehand.
                let entry_point_functions: Vec<::sway_ir::Function> = ir
                    .functions
                    .iter()
                    .filter_map(|(idx, fc)| {
                        if fc.name == "main" || fc.selector.is_some() {
                            Some(::sway_ir::function::Function(idx))
                        } else {
                            None
                        }
                    })
                    .collect();

                for function in entry_point_functions {
                    assert!(
                        sway_ir::optimize::inline_all_function_calls(&mut ir, &function).is_ok()
                    );
                }

                let asm_result = compile_ir_to_asm(&ir, None);
                assert!(
                    asm_result.is_ok(),
                    "Errors when compiling {test_file_name} IR to ASM."
                );

                let asm_output = asm_result
                    .value
                    .map(|asm| format!("{asm}"))
                    .expect("Failed to stringify ASM for {test_file_name}.");

                if asm_checker.is_empty() {
                    panic!(
                        "ASM test for {} has the '::check-asm::' marker \
                        but is missing directives.\n\
                        Please either remove the marker or add some.\n\n\
                        Here's the ASM output:\n{asm_output}",
                        path.file_name().unwrap().to_string_lossy()
                    );
                }

                // Do ASM checks.
                match asm_checker.explain(&asm_output, filecheck::NO_VARIABLES) {
                    Ok((success, report)) if !success => {
                        panic!("ASM filecheck for {test_file_name}failed:\n{report}");
                    }
                    Err(e) => {
                        panic!("ASM filecheck directive errors for {test_file_name}: {e}");
                    }
                    _ => (),
                };
            }

            // Parse the IR again, and print it yet again to make sure that IR de/serialisation works.
            let parsed_ir = sway_ir::parser::parse(&ir_output)
                .unwrap_or_else(|e| panic!("{}: {}", path.display(), e));
            let parsed_ir_output = sway_ir::printer::to_string(&parsed_ir);
            if ir_output != parsed_ir_output {
                tracing::error!("{}", prettydiff::diff_lines(&ir_output, &parsed_ir_output));
                panic!("{} failed IR (de)serialization.", path.display());
            }

            run_test_count += 1;
        });

    if run_test_count == 0 {
        tracing::warn!(
            "No IR generation tests were run. Regex filter \"{}\" filtered out all {} tests.",
            filter_regex
                .map(|regex| regex.to_string())
                .unwrap_or_default(),
            total_test_count,
        );
    } else {
        tracing::info!("Ran {run_test_count} out of {total_test_count} IR generation tests.");
    }
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

fn compile_core() -> namespace::Module {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let libcore_root_dir = format!("{manifest_dir}/../sway-lib-core");

    let check_cmd = forc::cli::CheckCommand {
        path: Some(libcore_root_dir),
        offline_mode: true,
        terse_mode: true,
        locked: false,
    };

    let res = forc::test::forc_check::check(check_cmd)
        .expect("Failed to compile sway-lib-core for IR tests.");

    match res.value {
        Some(typed_program) if res.is_ok() => {
            // Create a module for core and copy the compiled modules into it.  Unfortunately we
            // can't get mutable access to move them out so they're cloned.
            let core_module = typed_program.root.namespace.submodules().into_iter().fold(
                namespace::Module::default(),
                |mut core_mod, (name, sub_mod)| {
                    core_mod.insert_submodule(name.clone(), sub_mod.clone());
                    core_mod
                },
            );

            // Create a module for std and insert the core module.
            let mut std_module = namespace::Module::default();
            std_module.insert_submodule("core".to_owned(), core_module);
            std_module
        }
        _ => panic!("Failed to compile sway-lib-core for IR tests."),
    }
}
