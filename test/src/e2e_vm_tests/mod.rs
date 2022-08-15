// Please take a look in test_programs/README.md for details on how these tests work.

mod harness;

use forc_util::init_tracing_subscriber;
use fuel_vm::prelude::*;
use regex::Regex;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(PartialEq)]
enum TestCategory {
    Compiles,
    FailsToCompile,
    Runs,
    RunsWithContract,
    Disabled,
}

#[derive(Debug)]
enum TestResult {
    Result(Word),
    Return(u64),
    ReturnData(Bytes32),
    Revert(u64),
}

struct TestDescription {
    name: String,
    category: TestCategory,
    expected_result: Option<TestResult>,
    contract_paths: Vec<String>,
    validate_abi: bool,
    validate_abi_flat: bool,
    validate_storage_slots: bool,
    checker: filecheck::Checker,
}

pub fn run(locked: bool, filter_regex: Option<&regex::Regex>) {
    init_tracing_subscriber();

    let configured_tests = discover_test_configs().unwrap_or_else(|e| {
        panic!("Discovering tests {e}");
    });

    let total_number_of_tests = configured_tests.len();
    let mut number_of_tests_executed = 0;
    let mut number_of_disabled_tests = 0;

    let mut deployed_contracts = HashMap::<String, ContractId>::new();

    for TestDescription {
        name,
        category,
        expected_result,
        contract_paths,
        validate_abi,
        validate_abi_flat,
        validate_storage_slots,
        checker,
    } in configured_tests
    {
        if !filter_regex
            .map(|regex| regex.is_match(&name))
            .unwrap_or(true)
        {
            continue;
        }

        match category {
            TestCategory::Runs => {
                let res = match expected_result {
                    Some(TestResult::Return(v)) => ProgramState::Return(v),
                    Some(TestResult::ReturnData(bytes)) => ProgramState::ReturnData(bytes),
                    Some(TestResult::Revert(v)) => ProgramState::Revert(v),

                    _ => panic!(
                        "For {name}:\n\
                        Invalid expected result for a 'runs' test: {expected_result:?}."
                    ),
                };

                let result = crate::e2e_vm_tests::harness::runs_in_vm(&name, locked);
                assert_eq!(result.0, res);
                if validate_abi {
                    assert!(crate::e2e_vm_tests::harness::test_json_abi(&name, &result.1).is_ok());
                }
                if validate_abi_flat {
                    assert!(crate::e2e_vm_tests::harness::test_json_abi_flat(&name, &result.1).is_ok());
                }
                number_of_tests_executed += 1;
            }

            TestCategory::Compiles => {
                let (result, output) =
                    crate::e2e_vm_tests::harness::compile_and_capture_output(&name, locked);

                assert!(result.is_ok());
                check_file_checker(checker, &name, &output);

                let compiled = result.unwrap();
                if validate_abi {
                    assert!(crate::e2e_vm_tests::harness::test_json_abi(&name, &compiled).is_ok());
                }
                if validate_abi_flat {
                    assert!(crate::e2e_vm_tests::harness::test_json_abi_flat(&name, &compiled).is_ok());
                }
                if validate_storage_slots {
                    assert!(crate::e2e_vm_tests::harness::test_json_storage_slots(
                        &name, &compiled
                    )
                    .is_ok());
                }
                number_of_tests_executed += 1;
            }

            TestCategory::FailsToCompile => {
                let (result, output) =
                    crate::e2e_vm_tests::harness::compile_and_capture_output(&name, locked);
                match result {
                    Ok(_) => {
                        panic!("For {name}:\nFailing test did not fail.");
                    }
                    Err(_) => {
                        check_file_checker(checker, &name, &output);
                    }
                }
                number_of_tests_executed += 1;
            }

            TestCategory::RunsWithContract => {
                let val = if let Some(TestResult::Result(val)) = expected_result {
                    val
                } else {
                    panic!(
                        "For {name}:\nExpecting a 'result' action for a 'run_on_node' test, \
                        found: {expected_result:?}."
                    )
                };

                if contract_paths.is_empty() {
                    panic!(
                        "For {name}\n\
                        One or more ontract paths are required for 'run_on_node' tests."
                    );
                }

                let contract_ids = contract_paths
                    .into_iter()
                    .map(|contract_path| {
                        *deployed_contracts
                            .entry(contract_path.clone())
                            .or_insert_with(|| {
                                harness::deploy_contract(contract_path.as_str(), locked)
                            })
                    })
                    .collect::<Vec<_>>();

                let result = harness::runs_on_node(&name, locked, &contract_ids);
                assert!(result.iter().all(|res| !matches!(
                    res,
                    fuel_tx::Receipt::Revert { .. } | fuel_tx::Receipt::Panic { .. }
                )));
                assert!(
                    result.len() >= 2
                        && matches!(result[result.len() - 2], fuel_tx::Receipt::Return { .. })
                        && result[result.len() - 2].val().unwrap() == val
                );

                number_of_tests_executed += 1;
            }

            TestCategory::Disabled => {
                number_of_disabled_tests += 1;
            }
        }
    }

    if number_of_tests_executed == 0 {
        tracing::info!(
            "No E2E tests were run. Regex filter \"{}\" filtered out all {} tests.",
            filter_regex
                .map(|regex| regex.to_string())
                .unwrap_or_default(),
            total_number_of_tests
        );
    } else {
        tracing::info!("_________________________________\nTests passed.");
        tracing::info!(
            "Ran {number_of_tests_executed} \
            out of {total_number_of_tests} E2E tests \
            ({number_of_disabled_tests} disabled)."
        );
    }
}

fn discover_test_configs() -> Result<Vec<TestDescription>, String> {
    fn recursive_search(path: &Path, configs: &mut Vec<TestDescription>) -> Result<(), String> {
        let wrap_err = |e| {
            let relative_path = path
                .iter()
                .skip_while(|part| part.to_string_lossy() != "test_programs")
                .skip(1)
                .collect::<PathBuf>();
            format!("{}: {}", relative_path.display(), e)
        };
        if path.is_dir() {
            for entry in std::fs::read_dir(path).unwrap() {
                recursive_search(&entry.unwrap().path(), configs)?;
            }
        } else if path.is_file() && path.file_name().map(|f| f == "test.toml").unwrap_or(false) {
            configs.push(parse_test_toml(path).map_err(wrap_err)?);
        }
        Ok(())
    }

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let tests_root_dir = format!("{manifest_dir}/src/e2e_vm_tests/test_programs");

    let mut configs = Vec::new();
    recursive_search(&PathBuf::from(tests_root_dir), &mut configs)?;
    Ok(configs)
}

const DIRECTIVE_RX: &str = r"(?m)^\s*#\s*(\w+):\s+(.*)$";

fn build_file_checker(content: &str) -> Result<filecheck::Checker, String> {
    let mut checker = filecheck::CheckerBuilder::new();

    // Parse the file and check for unknown FileCheck directives.
    let re = Regex::new(DIRECTIVE_RX).unwrap();
    for cap in re.captures_iter(content) {
        if let Ok(false) = checker.directive(&cap[0]) {
            return Err(format!("Unknown FileCheck directive: {}", &cap[1]));
        }
    }

    Ok(checker.finish())
}

/// This functions gets passed the previously built FileCheck-based file checker,
/// along with the output of the compilation, and checks the output for the
/// FileCheck directives that were found in the test.toml file, panicking
/// if the checking fails.
fn check_file_checker(checker: filecheck::Checker, name: &String, output: &str) {
    match checker.explain(output, filecheck::NO_VARIABLES) {
        Ok((success, report)) if !success => {
            panic!("For {name}:\nFilecheck failed:\n{report}");
        }
        Err(e) => {
            panic!("For {name}:\nFilecheck directive error: {e}");
        }
        _ => (),
    }
}

fn parse_test_toml(path: &Path) -> Result<TestDescription, String> {
    let toml_content_str = std::fs::read_to_string(path).map_err(|e| e.to_string())?;

    let checker = build_file_checker(&toml_content_str)?;

    let toml_content = toml_content_str
        .parse::<toml::Value>()
        .map_err(|e| e.to_string())?;

    if !toml_content.is_table() {
        return Err("Malformed test description.".to_owned());
    }

    let category = toml_content
        .get("category")
        .ok_or_else(|| "Missing mandatory 'category' entry.".to_owned())
        .and_then(|category_val| match category_val.as_str() {
            Some("run") => Ok(TestCategory::Runs),
            Some("run_on_node") => Ok(TestCategory::RunsWithContract),
            Some("fail") => Ok(TestCategory::FailsToCompile),
            Some("compile") => Ok(TestCategory::Compiles),
            Some("disabled") => Ok(TestCategory::Disabled),
            None => Err(format!(
                "Malformed category '{category_val}', should be a string."
            )),
            Some(other) => Err(format!("Unknown category '{}'.", other,)),
        })?;

    // Abort early if we find a FailsToCompile test without any Checker directives.
    if category == TestCategory::FailsToCompile && checker.is_empty() {
        return Err("'fail' tests must contain some FileCheck verification directives.".to_owned());
    }

    let expected_result = match &category {
        TestCategory::Runs | TestCategory::RunsWithContract => {
            Some(get_expected_result(&toml_content)?)
        }
        TestCategory::Compiles | TestCategory::FailsToCompile | TestCategory::Disabled => None,
    };

    let contract_paths = match toml_content.get("contracts") {
        None => Vec::new(),
        Some(contracts) => contracts
            .as_array()
            .ok_or_else(|| "Contracts must be an array of strings.".to_owned())
            .and_then(|vals| {
                vals.iter()
                    .map(|val| {
                        val.as_str()
                            .ok_or_else(|| "Contracts must be path strings.".to_owned())
                            .map(|path_str| path_str.to_owned())
                    })
                    .collect::<Result<Vec<_>, _>>()
            })?,
    };

    let validate_abi = toml_content
        .get("validate_abi")
        .map(|v| v.as_bool().unwrap_or(false))
        .unwrap_or(false);
    
    let validate_abi_flat = toml_content
        .get("validate_abi_flat")
        .map(|v| v.as_bool().unwrap_or(false))
        .unwrap_or(false);

    let validate_storage_slots = toml_content
        .get("validate_storage_slots")
        .map(|v| v.as_bool().unwrap_or(false))
        .unwrap_or(false);

    // We need to adjust the path to start relative to `test_programs`.
    let name = path
        .iter()
        .skip_while(|part| part.to_string_lossy() != "test_programs")
        .skip(1)
        .collect::<PathBuf>();

    // And it needs to chop off the `test.toml` and convert to a String.
    let name = name
        .parent()
        .unwrap()
        .to_str()
        .map(|s| s.to_owned())
        .unwrap();

    Ok(TestDescription {
        name,
        category,
        expected_result,
        contract_paths,
        validate_abi,
        validate_abi_flat,
        validate_storage_slots,
        checker,
    })
}

fn get_expected_result(toml_content: &toml::Value) -> Result<TestResult, String> {
    fn get_action_value(
        action: &toml::Value,
        expected_value: &toml::Value,
    ) -> Result<TestResult, String> {
        match (action.as_str(), expected_value) {
            // A simple integer value.
            (Some("return"), toml::Value::Integer(v)) => Ok(TestResult::Return(*v as u64)),

            // Also a simple integer value, but is a result from a contract call.
            (Some("result"), toml::Value::Integer(v)) => Ok(TestResult::Result(*v as Word)),

            // A bytes32 value.
            (Some("return_data"), toml::Value::Array(ary)) => ary
                .iter()
                .map(|byte_val| {
                    byte_val.as_integer().ok_or_else(|| {
                        format!(
                            "Return data must only contain integer values; \
                                                    found {byte_val}."
                        )
                    })
                })
                .collect::<Result<Vec<_>, _>>()
                .and_then(|bytes| {
                    if bytes.iter().any(|byte| *byte < 0 || *byte > 255) {
                        Err("Return data byte values must be less than 256.".to_owned())
                    } else if bytes.len() != 32 {
                        Err(format!(
                            "Return data must be a 32 byte array; \
                                                found {} values.",
                            bytes.len()
                        ))
                    } else {
                        Ok(bytes.iter().map(|byte| *byte as u8).collect())
                    }
                })
                .map(|bytes: Vec<u8>| {
                    let fixed_byte_array =
                        bytes
                            .iter()
                            .enumerate()
                            .fold([0_u8; 32], |mut ary, (idx, byte)| {
                                ary[idx] = *byte;
                                ary
                            });
                    TestResult::ReturnData(Bytes32::from(fixed_byte_array))
                }),

            // Revert with a specific code.
            (Some("revert"), toml::Value::Integer(v)) => Ok(TestResult::Revert(*v as u64)),

            _otherwise => Err(format!("Malformed action value: {action} {expected_value}")),
        }
    }

    toml_content
        .get("expected_result")
        .ok_or_else(|| "Could not find mandatory 'expected_result' entry.".to_owned())
        .and_then(|expected_result_table| {
            expected_result_table
                .get("action")
                .ok_or_else(|| {
                    "Could not find mandatory 'action' field in 'expected_result' entry.".to_owned()
                })
                .and_then(|action| {
                    expected_result_table
                        .get("value")
                        .ok_or_else(|| {
                            "Could not find mandatory 'value' field in 'expected_result' entry."
                                .to_owned()
                        })
                        .and_then(|expected_value| get_action_value(action, expected_value))
                })
        })
}
