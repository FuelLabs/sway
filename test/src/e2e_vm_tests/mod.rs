// Please take a look in test_programs/README.md for details on how these tests work.

mod harness;
mod util;

use crate::e2e_vm_tests::harness::run_and_capture_output;
use crate::{FilterConfig, RunConfig};

use anyhow::{anyhow, bail, Result};
use assert_matches::assert_matches;
use colored::*;
use core::fmt;
use fuel_vm::fuel_tx;
use fuel_vm::prelude::*;
use regex::Regex;
use std::io::stdout;
use std::io::Write;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::Mutex;
use tracing::Instrument;

use self::util::VecExt;

#[derive(PartialEq, Debug)]
enum TestCategory {
    Compiles,
    FailsToCompile,
    Runs,
    RunsWithContract,
    UnitTestsPass,
    Disabled,
}

#[derive(PartialEq)]
enum TestResult {
    Result(Word),
    Return(u64),
    ReturnData(Vec<u8>),
    Revert(u64),
}

impl fmt::Debug for TestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestResult::Result(result) => write!(f, "Result({})", result),
            TestResult::Return(code) => write!(f, "Return({})", code),
            TestResult::ReturnData(data) => write!(f, "ReturnData(0x{})", hex::encode(data)),
            TestResult::Revert(code) => write!(f, "Revert({})", code),
        }
    }
}

struct TestDescription {
    name: String,
    category: TestCategory,
    script_data: Option<Vec<u8>>,
    expected_result: Option<TestResult>,
    contract_paths: Vec<String>,
    validate_abi: bool,
    validate_storage_slots: bool,
    checker: filecheck::Checker,
}

#[derive(Clone)]
struct TestContext {
    run_config: RunConfig,
    deployed_contracts: Arc<Mutex<HashMap<String, ContractId>>>,
}

impl TestContext {
    async fn deploy_contract(&self, contract_path: String) -> Result<ContractId> {
        let mut deployed_contracts = self.deployed_contracts.lock().await;
        Ok(
            if let Some(contract_id) = deployed_contracts.get(&contract_path) {
                *contract_id
            } else {
                let contract_id =
                    harness::deploy_contract(contract_path.as_str(), &self.run_config).await?;
                deployed_contracts.insert(contract_path, contract_id);
                contract_id
            },
        )
    }
    async fn run(&self, test: TestDescription, output: &mut String) -> Result<()> {
        let context = self;
        let TestDescription {
            name,
            category,
            script_data,
            expected_result,
            contract_paths,
            validate_abi,
            validate_storage_slots,
            checker,
        } = test;

        match category {
            TestCategory::Runs => {
                let res = match expected_result {
                    Some(TestResult::Return(_))
                    | Some(TestResult::ReturnData(_))
                    | Some(TestResult::Revert(_)) => expected_result.unwrap(),

                    _ => panic!(
                        "For {name}:\n\
                        Invalid expected result for a 'runs' test: {expected_result:?}."
                    ),
                };

                let (result, out) = run_and_capture_output(|| {
                    harness::compile_to_bytes(&name, &context.run_config)
                })
                .await;
                *output = out;

                let compiled = result?;

                let compiled = match compiled {
                    forc_pkg::Built::Package(built_pkg) => *built_pkg,
                    forc_pkg::Built::Workspace(_) => {
                        panic!("workspaces are not supported in the test suite yet")
                    }
                };

                let (state, receipts, pkg) = harness::runs_in_vm(compiled, script_data)?;
                let result = match state {
                    ProgramState::Return(v) => TestResult::Return(v),
                    ProgramState::ReturnData(digest) => {
                        // Find the ReturnData receipt matching the digest
                        let receipt = receipts
                            .iter()
                            .find(|r| r.digest() == Some(&digest))
                            .unwrap();
                        // Get the data from the receipt
                        let data = receipt.data().unwrap().to_vec();
                        TestResult::ReturnData(data)
                    }
                    ProgramState::Revert(v) => TestResult::Revert(v),
                };
                if result != res {
                    Err(anyhow::Error::msg(format!(
                        "expected: {:?}\nactual: {:?}",
                        res, result
                    )))
                } else {
                    if validate_abi {
                        let (result, out) = run_and_capture_output(|| async {
                            harness::test_json_abi(&name, &pkg)
                        })
                        .await;
                        result?;
                        output.push_str(&out);
                    }
                    Ok(())
                }
            }

            TestCategory::Compiles => {
                let (result, out) = run_and_capture_output(|| {
                    harness::compile_to_bytes(&name, &context.run_config)
                })
                .await;
                *output = out;

                let compiled_pkgs = match result? {
                    forc_pkg::Built::Package(built_pkg) => vec![(name.clone(), *built_pkg)],
                    forc_pkg::Built::Workspace(built_workspace) => built_workspace
                        .iter()
                        .map(|(n, b)| (n.clone(), b.clone()))
                        .collect(),
                };

                check_file_checker(checker, &name, output)?;

                if validate_abi {
                    for (name, built_pkg) in &compiled_pkgs {
                        let (result, out) = run_and_capture_output(|| async {
                            harness::test_json_abi(name, built_pkg)
                        })
                        .await;
                        result?;
                        output.push_str(&out);
                    }
                }

                if validate_storage_slots {
                    for (name, built_pkg) in &compiled_pkgs {
                        let (result, out) = run_and_capture_output(|| async {
                            harness::test_json_storage_slots(name, built_pkg)
                        })
                        .await;
                        result?;
                        output.push_str(&out);
                    }
                }
                Ok(())
            }

            TestCategory::FailsToCompile => {
                let (result, out) = run_and_capture_output(|| {
                    harness::compile_to_bytes(&name, &context.run_config)
                })
                .await;
                *output = out;

                if result.is_ok() {
                    Err(anyhow::Error::msg("Test compiles but is expected to fail"))
                } else {
                    check_file_checker(checker, &name, output)?;
                    Ok(())
                }
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
                        One or more contract paths are required for 'run_on_node' tests."
                    );
                }

                let mut contract_ids = Vec::new();
                for contract_path in contract_paths.clone() {
                    let (result, out) = run_and_capture_output(|| async {
                        context.deploy_contract(contract_path).await
                    })
                    .await;
                    output.push_str(&out);
                    contract_ids.push(result);
                }
                let contract_ids: Result<Vec<ContractId>, anyhow::Error> =
                    contract_ids.into_iter().collect();
                let (result, out) =
                    harness::runs_on_node(&name, &context.run_config, &contract_ids?).await;
                output.push_str(&out);

                let receipt = result?;
                assert!(receipt.iter().all(|res| !matches!(
                    res,
                    fuel_tx::Receipt::Revert { .. } | fuel_tx::Receipt::Panic { .. }
                )));
                assert!(receipt.len() >= 2);
                assert_matches!(receipt[receipt.len() - 2], fuel_tx::Receipt::Return { .. });
                assert_eq!(receipt[receipt.len() - 2].val().unwrap(), val);

                Ok(())
            }

            TestCategory::UnitTestsPass => {
                let (result, out) =
                    harness::compile_and_run_unit_tests(&name, &context.run_config, true).await;
                *output = out;

                let tested_pkg = result.expect("failed to compile and run unit tests");
                let failed: Vec<String> = tested_pkg
                    .tests
                    .into_iter()
                    .enumerate()
                    .filter_map(|(ix, test)| match test.state {
                        ProgramState::Revert(code) => {
                            Some(format!("Test {ix} failed with revert {code}\n"))
                        }
                        _ => None,
                    })
                    .collect();

                if !failed.is_empty() {
                    panic!(
                        "For {name}\n{} tests failed:\n{}",
                        failed.len(),
                        failed.into_iter().collect::<String>()
                    );
                }
                Ok(())
            }

            category => Err(anyhow::Error::msg(format!(
                "Unexpected test category: {:?}",
                category,
            ))),
        }
    }
}

pub async fn run(filter_config: &FilterConfig, run_config: &RunConfig) -> Result<()> {
    // Discover tests
    let mut tests = discover_test_configs()?;
    let total_number_of_tests = tests.len();

    // Filter tests
    let skipped_tests = filter_config
        .skip_until
        .as_ref()
        .map(|skip_until| {
            let mut found = false;
            tests.retained(|t| {
                found
                    || if skip_until.is_match(&t.name) {
                        found = true;
                        true
                    } else {
                        false
                    }
            })
        })
        .unwrap_or_default();
    let disabled_tests = tests.retained(|t| t.category != TestCategory::Disabled);
    let included_tests = filter_config
        .include
        .as_ref()
        .map(|include| tests.retained(|t| include.is_match(&t.name)))
        .unwrap_or_default();
    let excluded_tests = filter_config
        .exclude
        .as_ref()
        .map(|exclude| tests.retained(|t| !exclude.is_match(&t.name)))
        .unwrap_or_default();
    if filter_config.abi_only {
        tests.retain(|t| t.validate_abi);
    }
    if filter_config.contract_only {
        tests.retain(|t| t.category == TestCategory::RunsWithContract);
    }
    if filter_config.first_only && !tests.is_empty() {
        tests = vec![tests.remove(0)];
    }

    // Run tests
    let context = TestContext {
        run_config: run_config.clone(),
        deployed_contracts: Default::default(),
    };
    let mut number_of_tests_executed = 0;
    let mut number_of_tests_failed = 0;

    for (i, test) in tests.into_iter().enumerate() {
        print!("Testing {} ...", test.name.bold());
        stdout().flush().unwrap();

        let mut output = String::new();
        let result = if !filter_config.first_only {
            context
                .run(test, &mut output)
                .instrument(tracing::trace_span!("E2E", i))
                .await
        } else {
            context.run(test, &mut output).await
        };

        if let Err(err) = result {
            println!(" {}", "failed".red().bold());
            println!("{}", textwrap::indent(err.to_string().as_str(), "     "));
            println!("{}", textwrap::indent(&output, "          "));
            number_of_tests_failed += 1;
        } else {
            println!(" {}", "ok".green().bold());

            // If verbosity is requested then print it out.
            if run_config.verbose {
                println!("{}", textwrap::indent(&output, "     "));
            }
        }

        number_of_tests_executed += 1;
    }

    if number_of_tests_executed == 0 {
        if let Some(skip_until) = &filter_config.skip_until {
            tracing::info!(
                "Filtered {} tests with `skip-until` regex: {}",
                skipped_tests.len(),
                skip_until.to_string()
            );
        }
        if let Some(include) = &filter_config.include {
            tracing::info!(
                "Filtered {} tests with `include` regex: {}",
                included_tests.len(),
                include.to_string()
            );
        }
        if let Some(exclude) = &filter_config.exclude {
            tracing::info!(
                "Filtered {} tests with `exclude` regex: {}",
                excluded_tests.len(),
                exclude.to_string()
            );
        }
        if !disabled_tests.is_empty() {
            tracing::info!("{} tests were disabled.", disabled_tests.len());
        }
        tracing::warn!(
            "No tests were run. Regex filters filtered out all {} tests.",
            total_number_of_tests
        );
    } else {
        tracing::info!("_________________________________");
        tracing::info!(
            "Sway tests result: {}. {} total, {} passed; {} failed; {} disabled",
            if number_of_tests_failed == 0 {
                "ok".green().bold()
            } else {
                "failed".red().bold()
            },
            total_number_of_tests,
            number_of_tests_executed - number_of_tests_failed,
            number_of_tests_failed,
            disabled_tests.len()
        );
    }
    Ok(())
}

fn discover_test_configs() -> Result<Vec<TestDescription>> {
    fn recursive_search(path: &Path, configs: &mut Vec<TestDescription>) -> Result<()> {
        let wrap_err = |e| {
            let relative_path = path
                .iter()
                .skip_while(|part| part.to_string_lossy() != "test_programs")
                .skip(1)
                .collect::<PathBuf>();
            anyhow!("{}: {}", relative_path.display(), e)
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

fn build_file_checker(content: &str) -> Result<filecheck::Checker> {
    let mut checker = filecheck::CheckerBuilder::new();

    // Parse the file and check for unknown FileCheck directives.
    let re = Regex::new(DIRECTIVE_RX).unwrap();
    for cap in re.captures_iter(content) {
        if let Ok(false) = checker.directive(&cap[0]) {
            bail!("Unknown FileCheck directive: {}", &cap[1]);
        }
    }

    Ok(checker.finish())
}

/// This functions gets passed the previously built FileCheck-based file checker,
/// along with the output of the compilation, and checks the output for the
/// FileCheck directives that were found in the test.toml file, panicking
/// if the checking fails.
fn check_file_checker(checker: filecheck::Checker, name: &String, output: &str) -> Result<()> {
    match checker.explain(output, filecheck::NO_VARIABLES) {
        Ok((success, report)) if !success => Err(anyhow::Error::msg(format!(
            "For {name}:\nFilecheck failed:\n{report}"
        ))),
        Err(e) => {
            panic!("For {name}:\nFilecheck directive error: {e}");
        }
        _ => Ok(()),
    }
}

fn parse_test_toml(path: &Path) -> Result<TestDescription> {
    let toml_content_str = std::fs::read_to_string(path)?;

    let checker = build_file_checker(&toml_content_str)?;

    let toml_content = toml_content_str.parse::<toml::Value>()?;

    if !toml_content.is_table() {
        bail!("Malformed test description.");
    }

    let category = toml_content
        .get("category")
        .ok_or_else(|| anyhow!("Missing mandatory 'category' entry."))
        .and_then(|category_val| match category_val.as_str() {
            Some("run") => Ok(TestCategory::Runs),
            Some("run_on_node") => Ok(TestCategory::RunsWithContract),
            Some("fail") => Ok(TestCategory::FailsToCompile),
            Some("compile") => Ok(TestCategory::Compiles),
            Some("disabled") => Ok(TestCategory::Disabled),
            Some("unit_tests_pass") => Ok(TestCategory::UnitTestsPass),
            None => Err(anyhow!(
                "Malformed category '{category_val}', should be a string."
            )),
            Some(other) => Err(anyhow!("Unknown category '{}'.", other)),
        })?;

    // Abort early if we find a FailsToCompile test without any Checker directives.
    if category == TestCategory::FailsToCompile && checker.is_empty() {
        bail!("'fail' tests must contain some FileCheck verification directives.");
    }

    let script_data = match &category {
        TestCategory::Runs | TestCategory::RunsWithContract => {
            match toml_content.get("script_data") {
                Some(toml::Value::String(v)) => {
                    let decoded = hex::decode(v)
                        .map_err(|e| anyhow!("Invalid hex value for 'script_data': {}", e))?;
                    Some(decoded)
                }
                Some(_) => {
                    bail!("Expected 'script_data' to be a hex string.");
                }
                _ => None,
            }
        }
        TestCategory::Compiles
        | TestCategory::FailsToCompile
        | TestCategory::UnitTestsPass
        | TestCategory::Disabled => None,
    };

    let expected_result = match &category {
        TestCategory::Runs | TestCategory::RunsWithContract => {
            Some(get_expected_result(&toml_content)?)
        }
        TestCategory::Compiles
        | TestCategory::FailsToCompile
        | TestCategory::UnitTestsPass
        | TestCategory::Disabled => None,
    };

    let contract_paths = match toml_content.get("contracts") {
        None => Vec::new(),
        Some(contracts) => contracts
            .as_array()
            .ok_or_else(|| anyhow!("Contracts must be an array of strings."))
            .and_then(|vals| {
                vals.iter()
                    .map(|val| {
                        val.as_str()
                            .ok_or_else(|| anyhow!("Contracts must be path strings."))
                            .map(|path_str| path_str.to_owned())
                    })
                    .collect::<Result<Vec<_>, _>>()
            })?,
    };

    let validate_abi = toml_content
        .get("validate_abi")
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
        script_data,
        expected_result,
        contract_paths,
        validate_abi,
        validate_storage_slots,
        checker,
    })
}

fn get_expected_result(toml_content: &toml::Value) -> Result<TestResult> {
    fn get_action_value(action: &toml::Value, expected_value: &toml::Value) -> Result<TestResult> {
        match (action.as_str(), expected_value) {
            // A simple integer value.
            (Some("return"), toml::Value::Integer(v)) => Ok(TestResult::Return(*v as u64)),

            // Also a simple integer value, but is a result from a contract call.
            (Some("result"), toml::Value::Integer(v)) => Ok(TestResult::Result(*v as Word)),

            // A bytes32 value.
            (Some("return_data"), toml::Value::String(v)) => hex::decode(v)
                .map(TestResult::ReturnData)
                .map_err(|e| anyhow!("Invalid hex value for 'return_data': {}", e)),

            // Revert with a specific code.
            (Some("revert"), toml::Value::Integer(v)) => Ok(TestResult::Revert(*v as u64)),

            _otherwise => Err(anyhow!("Malformed action value: {action} {expected_value}")),
        }
    }

    toml_content
        .get("expected_result")
        .ok_or_else(|| anyhow!( "Could not find mandatory 'expected_result' entry."))
        .and_then(|expected_result_table| {
            expected_result_table
                .get("action")
                .ok_or_else(|| {
                    anyhow!("Could not find mandatory 'action' field in 'expected_result' entry.")
                })
                .and_then(|action| {
                    expected_result_table
                        .get("value")
                        .ok_or_else(|| {
                            anyhow!("Could not find mandatory 'value' field in 'expected_result' entry.")
                        })
                        .and_then(|expected_value| get_action_value(action, expected_value))
                })
        })
}
