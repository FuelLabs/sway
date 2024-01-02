// Please take a look in test_programs/README.md for details on how these tests work.

mod harness;
mod util;

use crate::e2e_vm_tests::harness::run_and_capture_output;
use crate::{FilterConfig, RunConfig};

use anyhow::{anyhow, bail, Result};
use assert_matches::assert_matches;
use colored::*;
use core::fmt;
use forc_pkg::BuildProfile;
use fuel_vm::fuel_tx;
use fuel_vm::fuel_types::canonical::Serialize;
use fuel_vm::prelude::*;
use regex::Regex;
use std::collections::HashSet;
use std::io::stdout;
use std::io::Write;
use std::str::FromStr;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use sway_core::BuildTarget;
use tokio::sync::Mutex;
use tracing::Instrument;

use self::util::VecExt;

#[derive(Clone, PartialEq, Debug)]
enum TestCategory {
    Compiles,
    FailsToCompile,
    Runs,
    RunsWithContract,
    UnitTestsPass,
    Disabled,
}

#[derive(Clone, PartialEq)]
enum TestResult {
    Result(Word),
    Return(u64),
    ReturnData(Vec<u8>),
    Revert(u64),
}

impl fmt::Debug for TestResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestResult::Result(result) => write!(f, "Result({result})"),
            TestResult::Return(code) => write!(f, "Return({code})"),
            TestResult::ReturnData(data) => write!(f, "ReturnData(0x{})", hex::encode(data)),
            TestResult::Revert(code) => write!(f, "Revert({code})"),
        }
    }
}

#[derive(Clone)]
pub struct FileCheck(String);

impl FileCheck {
    pub fn build(&self) -> Result<filecheck::Checker, anyhow::Error> {
        const DIRECTIVE_RX: &str = r"(?m)^\s*#\s*(\w+):\s+(.*)$";

        let mut checker = filecheck::CheckerBuilder::new();

        // Parse the file and check for unknown FileCheck directives.
        let re = Regex::new(DIRECTIVE_RX).unwrap();
        for cap in re.captures_iter(&self.0) {
            if let Ok(false) = checker.directive(&cap[0]) {
                bail!("Unknown FileCheck directive: {}", &cap[1]);
            }
        }

        Ok(checker.finish())
    }
}

#[derive(Clone)]
struct TestDescription {
    name: String,
    suffix: Option<String>,
    category: TestCategory,
    script_data: Option<Vec<u8>>,
    script_data_new_encoding: Option<Vec<u8>>,
    witness_data: Option<Vec<Vec<u8>>>,
    expected_result: Option<TestResult>,
    expected_warnings: u32,
    contract_paths: Vec<String>,
    validate_abi: bool,
    validate_storage_slots: bool,
    supported_targets: HashSet<BuildTarget>,
    unsupported_profiles: Vec<&'static str>,
    checker: FileCheck,
    run_config: Option<RunConfig>,
}

#[derive(Clone)]
struct TestContext {
    run_config: RunConfig,
    deployed_contracts: Arc<Mutex<HashMap<String, ContractId>>>,
}

fn print_receipts(output: &mut String, receipts: &[Receipt]) {
    use std::fmt::Write;
    let _ = writeln!(output, "  {}", "Receipts".green().bold());
    for (i, receipt) in receipts.iter().enumerate() {
        let _ = write!(output, "    {}", format!("#{i}").bold());
        match receipt {
            Receipt::LogData {
                id,
                ra,
                rb,
                ptr,
                len,
                digest,
                pc,
                is,
                data,
            } => {
                let _ = write!(output, " LogData\n      ID: {id:?}\n      RA: {ra:?}\n      RB: {rb:?}\n      Ptr: {ptr:?}\n      Len: {len:?}\n      Digest: {digest:?}\n      PC: {pc:?}\n      IS: {is:?}\n      Data: {data:?}\n");
            }
            Receipt::ReturnData {
                id,
                ptr,
                len,
                digest,
                pc,
                is,
                data,
            } => {
                let _ = write!(output, " ReturnData\n      ID: {id:?}\n      Ptr: {ptr:?}\n      Len: {len:?}\n      Digest: {digest:?}\n      PC: {pc:?}\n      IS: {is:?}\n      Data: {data:?}\n");
            }
            Receipt::Call {
                id,
                to,
                amount,
                asset_id,
                gas,
                param1,
                param2,
                pc,
                is,
            } => {
                let _ = write!(output, " Call\n      ID: {id:?}\n      To: {to:?}\n      Amount: {amount:?}\n      Asset ID: {asset_id:?}\n      Gas: {gas:?}\n      Param #1: {param1:?}\n      Param #2: {param2:?}\n      PC: {pc:?}\n      IS: {is:?}\n");
            }
            Receipt::Return { id, val, pc, is } => {
                let _ = write!(output, " Return\n      ID: {id:?}\n      Value: {val:?}\n      PC: {pc:?}\n      IS: {is:?}\n");
            }
            Receipt::Panic {
                id,
                reason,
                pc,
                is,
                contract_id,
            } => {
                let _ = write!(output, " Panic\n      ID: {id:?}\n      Reason: {reason:?}\n      PC: {pc:?}\n      IS: {is:?}\n      Contract ID: {contract_id:?}\n");
            }
            Receipt::Revert { id, ra, pc, is } => {
                let _ = write!(output, " Revert\n      ID: {id:?}\n      RA: {ra:?}\n      PC: {pc:?}\n      IS: {is:?}\n");
            }
            Receipt::Log {
                id,
                ra,
                rb,
                rc,
                rd,
                pc,
                is,
            } => {
                let _ = write!(output, " Log\n      ID: {id:?}\n      RA: {ra:?}\n      RB: {rb:?}\n      RC: {rc:?}\n      RD: {rd:?}\n      PC: {pc:?}\n      IS: {is:?}\n");
            }
            Receipt::Transfer {
                id,
                to,
                amount,
                asset_id,
                pc,
                is,
            } => {
                let _ = write!(output, " Transfer\n      ID: {id:?}\n      To: {to:?}\n      Amount: {amount:?}\n      Asset ID: {asset_id:?}\n      PC: {pc:?}\n      IS: {is:?}\n");
            }
            Receipt::TransferOut {
                id,
                to,
                amount,
                asset_id,
                pc,
                is,
            } => {
                let _ = write!(output, " TransferOut\n      ID: {id:?}\n      To: {to:?}\n      Amount: {amount:?}\n      Asset ID: {asset_id:?}\n      PC: {pc:?}\n      IS: {is:?}\n");
            }
            Receipt::ScriptResult { result, gas_used } => {
                let _ = write!(
                    output,
                    " ScriptResult\n      Result: {result:?}\n      Gas Used: {gas_used:?}\n"
                );
            }
            Receipt::MessageOut {
                sender,
                recipient,
                amount,
                nonce,
                len,
                digest,
                data,
            } => {
                let _ = write!(output, " MessageOut\n      Sender: {sender:?}\n      Recipient: {recipient:?}\n      Amount: {amount:?}\n      Nonce: {nonce:?}\n      Len: {len:?}\n      Digest: {digest:?}\n      Data: {data:?}\n");
            }
            Receipt::Mint {
                sub_id,
                contract_id,
                val,
                pc,
                is,
            } => {
                let _ = write!(output, " Mint\n      Sub ID: {sub_id:?}\n      Contract ID: {contract_id:?}\n      Val: {val:?}\n      PC: {pc:?}\n      IS: {is:?}\n");
            }
            Receipt::Burn {
                sub_id,
                contract_id,
                val,
                pc,
                is,
            } => {
                let _ = write!(output, " Burn\n      Sub ID: {sub_id:?}\n      Contract ID: {contract_id:?}\n      Val: {val:?}\n      PC: {pc:?}\n      IS: {is:?}\n");
            }
        }
    }
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

    async fn run(&self, test: TestDescription, output: &mut String, verbose: bool) -> Result<()> {
        let context = self;
        let TestDescription {
            name,
            category,
            script_data,
            script_data_new_encoding,
            witness_data,
            expected_result,
            expected_warnings,
            contract_paths,
            validate_abi,
            validate_storage_slots,
            checker,
            ..
        } = test;

        let checker = checker.build().unwrap();

        let script_data = if self.run_config.experimental.new_encoding {
            script_data_new_encoding
        } else {
            script_data
        };

        match category {
            TestCategory::Runs => {
                let expected_result = match expected_result {
                    Some(TestResult::Return(v)) => {
                        // With the new encoding, `Return` is actually `ReturnData`
                        if context.run_config.experimental.new_encoding {
                            TestResult::ReturnData(v.to_bytes())
                        } else {
                            expected_result.unwrap()
                        }
                    }
                    Some(TestResult::ReturnData(_)) | Some(TestResult::Revert(_)) => {
                        expected_result.unwrap()
                    }

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
                    forc_pkg::Built::Package(built_pkg) => built_pkg.as_ref().clone(),
                    forc_pkg::Built::Workspace(_) => {
                        panic!("workspaces are not supported in the test suite yet")
                    }
                };

                if compiled.warnings.len() > expected_warnings as usize {
                    return Err(anyhow::Error::msg(format!(
                        "Expected warnings: {expected_warnings}\nActual number of warnings: {}",
                        compiled.warnings.len()
                    )));
                }

                let result = harness::runs_in_vm(compiled.clone(), script_data, witness_data)?;
                let actual_result = match result {
                    harness::VMExecutionResult::Fuel(state, receipts) => {
                        print_receipts(output, &receipts);
                        match state {
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
                            ProgramState::RunProgram(_) => {
                                panic!("Execution is in a suspended state: RunProgram");
                            }
                            ProgramState::VerifyPredicate(_) => {
                                panic!("Execution is in a suspended state: VerifyPredicate");
                            }
                        }
                    }
                    harness::VMExecutionResult::Evm(state) => match state.exit_reason {
                        revm::Return::Continue => todo!(),
                        revm::Return::Stop => TestResult::Result(0),
                        revm::Return::Return => todo!(),
                        revm::Return::SelfDestruct => todo!(),
                        revm::Return::Revert => TestResult::Revert(0),
                        _ => {
                            panic!("EVM exited with unhandled reason: {:?}", state.exit_reason);
                        }
                    },
                    harness::VMExecutionResult::MidenVM(trace) => {
                        let outputs = trace.program_outputs();
                        let stack = outputs.stack();
                        // for now, just test primitive u64s.
                        // Later on, we can test stacks that have more elements in them.
                        TestResult::Return(stack[0])
                    }
                };

                if actual_result != expected_result {
                    Err(anyhow::Error::msg(format!(
                        "expected: {expected_result:?}\nactual: {actual_result:?}"
                    )))
                } else {
                    if validate_abi {
                        let (result, out) = run_and_capture_output(|| async {
                            harness::test_json_abi(
                                &name,
                                &compiled,
                                self.run_config.experimental.new_encoding,
                                self.run_config.update_output_files,
                            )
                        })
                        .await;
                        output.push_str(&out);
                        result?;
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
                    forc_pkg::Built::Package(built_pkg) => {
                        if built_pkg.warnings.len() > expected_warnings as usize {
                            return Err(anyhow::Error::msg(format!(
                                "Expected warnings: {expected_warnings}\nActual number of warnings: {}",
                                built_pkg.warnings.len()
                            )));
                        }
                        vec![(name.clone(), built_pkg.as_ref().clone())]
                    }
                    forc_pkg::Built::Workspace(built_workspace) => built_workspace
                        .iter()
                        .map(|built_pkg| {
                            (
                                built_pkg.descriptor.pinned.name.clone(),
                                built_pkg.as_ref().clone(),
                            )
                        })
                        .collect(),
                };

                check_file_checker(checker, &name, output)?;

                if validate_abi {
                    for (name, built_pkg) in &compiled_pkgs {
                        let (result, out) = run_and_capture_output(|| async {
                            harness::test_json_abi(
                                name,
                                built_pkg,
                                self.run_config.experimental.new_encoding,
                                self.run_config.update_output_files,
                            )
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
                let contract_ids = contract_ids.into_iter().collect::<Result<Vec<_>, _>>()?;
                let (result, out) =
                    harness::runs_on_node(&name, &context.run_config, &contract_ids).await;
                output.push_str(&out);

                let receipt = result?;
                if !receipt.iter().all(|res| {
                    !matches!(
                        res,
                        fuel_tx::Receipt::Revert { .. } | fuel_tx::Receipt::Panic { .. }
                    )
                }) {
                    println!();
                    for cid in contract_ids {
                        println!("Deployed contract: 0x{cid}");
                    }
                    panic!("Receipts contain reverts or panics: {receipt:?}");
                }
                assert!(receipt.len() >= 2);
                assert_matches!(receipt[receipt.len() - 2], fuel_tx::Receipt::Return { .. });
                assert_eq!(receipt[receipt.len() - 2].val().unwrap(), val);

                Ok(())
            }

            TestCategory::UnitTestsPass => {
                let (result, out) =
                    harness::compile_and_run_unit_tests(&name, &context.run_config, true).await;
                *output = out;

                result.map(|tested_pkgs| {
                    let mut failed = vec![];
                    for pkg in tested_pkgs {
                        for test in pkg.tests.into_iter() {
                            if verbose {
                                println!("Test: {} {}", test.name, test.passed());
                                for log in test.logs.iter() {
                                    println!("{:?}", log);
                                }
                            }

                            if !test.passed() {
                                failed.push(format!(
                                    "{}: Test '{}' failed with state {:?}, expected: {:?}",
                                    pkg.built.descriptor.name,
                                    test.name,
                                    test.state,
                                    test.condition,
                                ));
                            }
                        }
                    }

                    if !failed.is_empty() {
                        println!("FAILED!! output:\n{}", output);
                        panic!(
                            "For {name}\n{} tests failed:\n{}",
                            failed.len(),
                            failed.into_iter().collect::<String>()
                        );
                    }
                })
            }

            category => Err(anyhow::Error::msg(format!(
                "Unexpected test category: {category:?}",
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

    // Expand tests that need to run with multiple configurations.
    // Be mindful that this can explode exponentially the number of tests
    // that run because one expansion expands on top of another
    let mut tests = tests;
    let expansions = ["new_encoding"];
    for expansion in expansions {
        tests = tests
            .into_iter()
            .flat_map(|t| {
                if expansion == "new_encoding" && t.script_data_new_encoding.is_some() {
                    let mut with_new_encoding = t.clone();
                    with_new_encoding.suffix = Some("New Encoding".into());

                    let mut run_config_with_new_encoding = run_config.clone();
                    run_config_with_new_encoding.experimental.new_encoding = true;
                    with_new_encoding.run_config = Some(run_config_with_new_encoding);

                    vec![t, with_new_encoding]
                } else {
                    vec![t]
                }
            })
            .collect();
    }

    let cur_profile = if run_config.release {
        BuildProfile::RELEASE
    } else {
        BuildProfile::DEBUG
    };
    tests.retain(|t| !t.unsupported_profiles.contains(&cur_profile));

    // Run tests
    let context = TestContext {
        run_config: run_config.clone(),
        deployed_contracts: Default::default(),
    };
    let mut number_of_tests_executed = 0;
    let mut number_of_tests_failed = 0;
    let mut failed_tests = vec![];

    for (i, test) in tests.into_iter().enumerate() {
        let name = if let Some(suffix) = test.suffix.as_ref() {
            format!("{} ({})", test.name, suffix)
        } else {
            test.name.clone()
        };

        let run_config = test
            .run_config
            .clone()
            .unwrap_or_else(|| run_config.clone());

        print!("Testing {} ...", name.clone().bold());
        stdout().flush().unwrap();

        let mut output = String::new();

        // Skip the test if its not compatible with the current build target.
        if !test.supported_targets.contains(&run_config.build_target) {
            continue;
        }

        use std::fmt::Write;
        let _ = writeln!(output, " {}", "Verbose Output".green().bold());
        let result = if !filter_config.first_only {
            context
                .run(test, &mut output, run_config.verbose)
                .instrument(tracing::trace_span!("E2E", i))
                .await
        } else {
            context.run(test, &mut output, run_config.verbose).await
        };

        if let Err(err) = result {
            println!(" {}", "failed".red().bold());
            println!("{}", textwrap::indent(err.to_string().as_str(), "     "));
            println!("{}", textwrap::indent(&output, "          "));
            number_of_tests_failed += 1;
            failed_tests.push(name);
        } else {
            println!(" {}", "ok".green().bold());

            // If verbosity is requested then print it out.
            if run_config.verbose && !output.is_empty() {
                println!("{}", textwrap::indent(&output, "     "));
            }
        }

        number_of_tests_executed += 1;
    }

    if number_of_tests_executed == 0 {
        if let Some(skip_until) = &filter_config.skip_until {
            tracing::info!(
                "Filtered {} tests with `skip-until` regex: {:?}",
                skipped_tests.len(),
                skip_until.to_string()
            );
        }
        if let Some(include) = &filter_config.include {
            tracing::info!(
                "Filtered {} tests with `include` regex: {:?}",
                included_tests.len(),
                include.to_string()
            );
        }
        if let Some(exclude) = &filter_config.exclude {
            tracing::info!(
                "Filtered {} tests with `exclude` regex: {:?}",
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
        if number_of_tests_failed > 0 {
            tracing::info!("{}", "Failing tests:".red().bold());
            tracing::info!(
                "    {}",
                failed_tests
                    .into_iter()
                    .map(|test_name| format!("{} ... {}", test_name.bold(), "failed".red().bold()))
                    .collect::<Vec<_>>()
                    .join("\n    ")
            );
        }
    }
    if number_of_tests_failed != 0 {
        Err(anyhow::Error::msg("Failed tests"))
    } else {
        Ok(())
    }
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

    let file_check = FileCheck(toml_content_str.clone());
    let checker = file_check.build()?;

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

    let (script_data, script_data_new_encoding) = match &category {
        TestCategory::Runs | TestCategory::RunsWithContract => {
            let script_data = match toml_content.get("script_data") {
                Some(toml::Value::String(v)) => {
                    let decoded = hex::decode(v.replace(' ', ""))
                        .map_err(|e| anyhow!("Invalid hex value for 'script_data': {}", e))?;
                    Some(decoded)
                }
                Some(_) => {
                    bail!("Expected 'script_data' to be a hex string.");
                }
                _ => None,
            };

            let script_data_new_encoding = match toml_content.get("script_data_new_encoding") {
                Some(toml::Value::String(v)) => {
                    let decoded = hex::decode(v.replace(' ', ""))
                        .map_err(|e| anyhow!("Invalid hex value for 'script_data': {}", e))?;
                    Some(decoded)
                }
                Some(_) => {
                    bail!("Expected 'script_data' to be a hex string.");
                }
                _ => None,
            };

            (script_data, script_data_new_encoding)
        }
        TestCategory::Compiles
        | TestCategory::FailsToCompile
        | TestCategory::UnitTestsPass
        | TestCategory::Disabled => (None, None),
    };

    let witness_data = match &category {
        TestCategory::Runs | TestCategory::RunsWithContract => {
            match toml_content.get("witness_data") {
                Some(toml::Value::Array(items)) => {
                    let mut data = vec![];

                    for item in items {
                        let decoded = item
                            .as_str()
                            .ok_or_else(|| anyhow!("witness data should be a hex string"))
                            .and_then(|x| {
                                hex::decode(x).map_err(|e| {
                                    anyhow!("Invalid hex value for 'script_data': {}", e)
                                })
                            })?;
                        data.push(decoded);
                    }

                    Some(data)
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

    let expected_warnings = u32::try_from(
        toml_content
            .get("expected_warnings")
            .map(|v| v.as_integer().unwrap_or(0))
            .unwrap_or(0),
    )
    .unwrap_or(0u32);

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

    // Check for supported build target for each test. For now we assume that the
    // the default is that only Fuel VM target is supported. Once the other targets
    // get to a fully usable state, we should update this.
    let supported_targets = toml_content
        .get("supported_targets")
        .map(|v| v.as_array().cloned().unwrap_or_default())
        .unwrap_or_default()
        .iter()
        .map(get_test_abi_from_value)
        .collect::<Result<Vec<BuildTarget>>>()?;

    // Check for not supported build profiles. Default is empty.
    let unsupported_profiles = toml_content
        .get("unsupported_profiles")
        .map(|v| v.as_array().cloned().unwrap_or_default())
        .unwrap_or_default()
        .iter()
        .map(get_build_profile_from_value)
        .collect::<Result<Vec<&'static str>>>()?;

    let supported_targets = HashSet::from_iter(if supported_targets.is_empty() {
        vec![BuildTarget::Fuel]
    } else {
        supported_targets
    });

    Ok(TestDescription {
        name,
        suffix: None,
        category,
        script_data,
        script_data_new_encoding,
        witness_data,
        expected_result,
        expected_warnings,
        contract_paths,
        validate_abi,
        validate_storage_slots,
        supported_targets,
        unsupported_profiles,
        checker: file_check,
        run_config: None,
    })
}

fn get_test_abi_from_value(value: &toml::Value) -> Result<BuildTarget> {
    match value.as_str() {
        Some(target) => match BuildTarget::from_str(target) {
            Ok(target) => Ok(target),
            _ => Err(anyhow!(format!("Unknown build target: {target}"))),
        },
        None => Err(anyhow!("Invalid TOML value")),
    }
}

fn get_build_profile_from_value(value: &toml::Value) -> Result<&'static str> {
    match value.as_str() {
        Some(profile) => match profile {
            BuildProfile::DEBUG => Ok(BuildProfile::DEBUG),
            BuildProfile::RELEASE => Ok(BuildProfile::RELEASE),
            _ => Err(anyhow!(format!("Unknown build profile"))),
        },
        None => Err(anyhow!("Invalid TOML value")),
    }
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
