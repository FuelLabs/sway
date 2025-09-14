// Please take a look in test_programs/README.md for details on how these tests work.

mod harness;
mod harness_callback_handler;
mod util;

use crate::e2e_vm_tests::harness::run_and_capture_output;
use crate::{FilterConfig, RunConfig};

use anyhow::{anyhow, bail, Result};
use colored::*;
use core::fmt;
use forc_pkg::manifest::{GenericManifestFile, ManifestFile};
use forc_pkg::BuildProfile;
use forc_test::ecal::Syscall;
use forc_util::tx_utils::decode_log_data;
use fuel_vm::fuel_tx;
use fuel_vm::fuel_types::canonical::Input;
use fuel_vm::prelude::*;
use regex::Regex;
use std::collections::{BTreeMap, HashSet};
use std::io::stdout;
use std::io::Write;
use std::str::FromStr;
use std::time::Instant;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use sway_core::BuildTarget;
use sway_features::{CliFields, ExperimentalFeatures};
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
    expected_result_new_encoding: Option<TestResult>,
    expected_warnings: u32,
    contract_paths: Vec<String>,
    validate_abi: bool,
    validate_storage_slots: bool,
    supported_targets: HashSet<BuildTarget>,
    expected_decoded_test_logs: Option<Vec<String>>,
    unsupported_profiles: Vec<&'static str>,
    checker: FileCheck,
    run_config: RunConfig,
    experimental: ExperimentalFeatures,
    has_experimental_field: bool,
    logs: Option<String>,
}

#[derive(PartialEq, Eq, Hash)]
struct DeployedContractKey {
    pub contract_path: String,
    pub new_encoding: bool,
}

#[derive(Clone)]
struct TestContext {
    deployed_contracts: Arc<Mutex<HashMap<DeployedContractKey, ContractId>>>,
}

fn print_receipts(output: &mut String, receipts: &[Receipt]) {
    let mut text_log = String::new();

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
                // Small hack to allow log from tests.
                if *ra == u64::MAX {
                    match rb {
                        0 => {
                            let mut data = data.as_deref().unwrap();
                            data.skip(8).unwrap();
                            let s = std::str::from_utf8(data).unwrap();

                            text_log.push_str(s);
                        }
                        1 => {
                            let data = data.as_deref().unwrap();
                            let s = u64::from_be_bytes(data.try_into().unwrap());

                            text_log.push_str(&format!("{s}"));
                        }
                        2 => {
                            text_log.push('\n');
                        }
                        _ => {}
                    }
                }
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

    if !text_log.is_empty() {
        let _ = writeln!(output, "  {}", "Text Logs".green().bold());

        for l in text_log.lines() {
            let _ = writeln!(output, "{l}");
        }
    }
}

struct RunResult {
    size: Option<u64>,
    gas: Option<u64>,
}

impl TestContext {
    async fn deploy_contract(
        &self,
        run_config: &RunConfig,
        contract_path: String,
    ) -> Result<ContractId> {
        let experimental = ExperimentalFeatures::new(
            &HashMap::default(),
            &run_config.experimental.experimental,
            &run_config.experimental.no_experimental,
        )
        .unwrap();

        let key = DeployedContractKey {
            contract_path: contract_path.clone(),
            new_encoding: experimental.new_encoding,
        };

        let mut deployed_contracts = self.deployed_contracts.lock().await;
        Ok(if let Some(contract_id) = deployed_contracts.get(&key) {
            *contract_id
        } else {
            let contract_id = harness::deploy_contract(contract_path.as_str(), run_config).await?;
            deployed_contracts.insert(key, contract_id);
            contract_id
        })
    }

    async fn run(&self, test: TestDescription, output: &mut String, verbose: bool) -> Result<RunResult> {
        let TestDescription {
            name,
            suffix,
            category,
            script_data,
            script_data_new_encoding,
            witness_data,
            expected_result,
            expected_result_new_encoding,
            expected_warnings,
            contract_paths,
            validate_abi,
            validate_storage_slots,
            checker,
            run_config,
            expected_decoded_test_logs,
            experimental,
            has_experimental_field,
            logs,
            ..
        } = test;

        let checker = checker.build().unwrap();

        let script_data = if !has_experimental_field && experimental.new_encoding {
            script_data_new_encoding
        } else {
            script_data
        };

        let expected_result = if !has_experimental_field && experimental.new_encoding {
            expected_result_new_encoding
        } else {
            expected_result
        };

        let mut r = RunResult {
            size: None,
            gas: None,
        };

        match category {
            TestCategory::Runs => {
                let expected_result = expected_result.expect("No expected result found. This is likely because test.toml is missing either an \"expected_result_new_encoding\" or \"expected_result\" entry");

                let (result, out) =
                    run_and_capture_output(|| harness::compile_to_bytes(&name, &run_config, &logs))
                        .await;
                *output = out;

                if let Ok(result) = result.as_ref() {
                    let packages = match result {
                        forc_pkg::Built::Package(p) => [p.clone()].to_vec(),
                        forc_pkg::Built::Workspace(p) => p.clone(),
                    };

                    for p in packages {
                        let bytecode_len = p.bytecode.bytes.len();
                        r.size = Some(bytecode_len as u64);

                        let configurables = match &p.program_abi {
                            sway_core::asm_generation::ProgramABI::Fuel(abi) => {
                                abi.configurables.as_ref().cloned().unwrap_or_default()
                            }
                            sway_core::asm_generation::ProgramABI::Evm(_)
                            | sway_core::asm_generation::ProgramABI::MidenVM(_) => vec![],
                        }
                        .into_iter()
                        .map(|x| (x.offset, x.name))
                        .collect::<BTreeMap<u64, String>>();

                        let mut items = configurables.iter().peekable();
                        while let Some(current) = items.next() {
                            let next_offset = match items.peek() {
                                Some(next) => *next.0,
                                None => bytecode_len as u64,
                            };
                            let size = next_offset - current.0;
                            output.push_str(&format!(
                                "Configurable Encoded Bytes Buffer Size: {} {}\n",
                                current.1, size
                            ));
                        }
                    }
                }

                check_file_checker(checker, &name, output)?;

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
                    harness::VMExecutionResult::Fuel(state, receipts, ecal) => {
                        print_receipts(output, &receipts);

                        if let Some(gas_used) = receipts.iter().filter_map(|x| match x {
                            Receipt::ScriptResult { gas_used, .. } => Some(*gas_used),
                            _ => None
                        }).last() {
                            r.gas = Some(gas_used);
                        }

                        use std::fmt::Write;
                        let _ = writeln!(output, "  {}", "Captured Output".green().bold());
                        for captured in ecal.captured.iter() {
                            match captured {
                                Syscall::Write { bytes, .. } => {
                                    let s = std::str::from_utf8(bytes.as_slice()).unwrap();
                                    output.push_str(s);
                                }
                                Syscall::Fflush { .. } => {}
                                Syscall::Unknown { ra, rb, rc, rd } => {
                                    let _ = writeln!(output, "Unknown ecal: {ra} {rb} {rc} {rd}");
                                }
                            }
                        }

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
                    harness::VMExecutionResult::Evm(state) => match state {
                        revm::primitives::ExecutionResult::Success { reason, .. } => match reason {
                            revm::primitives::SuccessReason::Stop => TestResult::Result(0),
                            revm::primitives::SuccessReason::Return => todo!(),
                            revm::primitives::SuccessReason::SelfDestruct => todo!(),
                            revm::primitives::SuccessReason::EofReturnContract => todo!(),
                        },
                        revm::primitives::ExecutionResult::Revert { .. } => TestResult::Result(0),
                        revm::primitives::ExecutionResult::Halt { reason, .. } => {
                            panic!("EVM exited with unhandled reason: {reason:?}");
                        }
                    },
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
                                experimental.new_encoding,
                                run_config.update_output_files,
                                &suffix,
                                has_experimental_field,
                            )
                        })
                        .await;

                        output.push_str(&out);
                        result?;
                    }
                    Ok(r)
                }
            }

            TestCategory::Compiles => {
                let (result, out) =
                    run_and_capture_output(|| harness::compile_to_bytes(&name, &run_config, &logs))
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
                                experimental.new_encoding,
                                run_config.update_output_files,
                                &suffix,
                                has_experimental_field,
                            )
                        })
                        .await;
                        output.push_str(&out);
                        result?;
                    }
                }

                if validate_storage_slots {
                    for (name, built_pkg) in &compiled_pkgs {
                        let (result, out) = run_and_capture_output(|| async {
                            harness::test_json_storage_slots(name, built_pkg, &suffix)
                        })
                        .await;
                        result?;
                        output.push_str(&out);
                    }
                }
                Ok(r)
            }

            TestCategory::FailsToCompile => {
                let (result, out) =
                    run_and_capture_output(|| harness::compile_to_bytes(&name, &run_config, &logs))
                        .await;

                *output = out;

                if result.is_ok() {
                    if verbose {
                        eprintln!("[{output}]");
                    }

                    Err(anyhow::Error::msg("Test compiles but is expected to fail"))
                } else {
                    check_file_checker(checker, &name, output)?;
                    Ok(r)
                }
            }

            TestCategory::RunsWithContract => {
                if contract_paths.is_empty() {
                    panic!(
                        "For {name}\n\
                        One or more contract paths are required for 'run_on_node' tests."
                    );
                }

                let mut contract_ids = Vec::new();
                for contract_path in contract_paths.clone() {
                    let (result, out) = run_and_capture_output(|| async {
                        self.deploy_contract(&run_config, contract_path).await
                    })
                    .await;
                    output.push_str(&out);
                    contract_ids.push(result);
                }
                let contract_ids = contract_ids.into_iter().collect::<Result<Vec<_>, _>>()?;

                let (result, out) = harness::runs_on_node(&name, &run_config, &contract_ids).await;

                output.push_str(&out);

                let receipts = result?;

                if verbose {
                    print_receipts(output, &receipts);
                }

                if !receipts.iter().all(|res| {
                    !matches!(
                        res,
                        fuel_tx::Receipt::Revert { .. } | fuel_tx::Receipt::Panic { .. }
                    )
                }) {
                    println!();
                    for cid in contract_ids {
                        println!("Deployed contract: 0x{cid}");
                    }

                    return Err(anyhow::Error::msg("Receipts contain reverts or panics"));
                }

                if receipts.len() < 2 {
                    return Err(anyhow::Error::msg(format!(
                        "less than 2 receipts: {:?} receipts",
                        receipts.len()
                    )));
                }

                match &receipts[receipts.len() - 2] {
                    Receipt::Return { val, .. } => match expected_result.unwrap() {
                        TestResult::Result(v) => {
                            if v != *val {
                                return Err(anyhow::Error::msg(format!(
                                    "return value does not match expected: {v:?}, {val:?}"
                                )));
                            }
                        }
                        TestResult::ReturnData(_) => {
                            todo!("Test result `ReturnData` is currently not implemented.")
                        }
                        TestResult::Return(_) => {
                            todo!("Test result `Return` is currently not implemented.")
                        }
                        TestResult::Revert(_) => {
                            todo!("Test result `Revert` is currently not implemented.")
                        }
                    },
                    Receipt::ReturnData { data, .. } => match expected_result.unwrap() {
                        TestResult::ReturnData(v) => {
                            if v != *data.as_ref().unwrap() {
                                return Err(anyhow::Error::msg(format!(
                                    "return value does not match expected: {v:?}, {data:?}"
                                )));
                            }
                        }
                        TestResult::Result(_) => {
                            todo!("Test result `Result` is currently not implemented.")
                        }
                        TestResult::Return(_) => {
                            todo!("Test result `Return` is currently not implemented.")
                        }
                        TestResult::Revert(_) => {
                            todo!("Test result `Revert` is currently not implemented.")
                        }
                    },
                    _ => {}
                };

                Ok(r)
            }

            TestCategory::UnitTestsPass => {
                let (result, out) =
                    harness::compile_and_run_unit_tests(&name, &run_config, true).await;
                *output = out;

                let mut decoded_logs = vec![];

                result.map(|tested_pkgs| {
                    let mut failed = vec![];
                    for pkg in tested_pkgs {
                        if !pkg.tests.is_empty() {
                            println!();
                        }
                        for test in pkg.tests.into_iter() {
                            if verbose {
                                //"test incorrect_def_modeling ... ok (17.673µs, 59 gas)"
                                println!("    test {} ... {} ({:?}, {} gas)", 
                                    test.name,
                                    if test.passed() { "ok" } else { "nok" },
                                    test.duration,
                                    test.gas_used,
                                );
                                for log in test.logs.iter() {
                                    println!("{log:?}");
                                }
                            }

                            if expected_decoded_test_logs.is_some() {
                                for log in test.logs.iter() {
                                    if let Receipt::LogData {
                                        rb,
                                        data: Some(data),
                                        ..
                                    } = log
                                    {
                                        let decoded_log_data = decode_log_data(
                                            &rb.to_string(),
                                            data,
                                            &pkg.built.program_abi,
                                        )
                                        .unwrap();
                                        let var_value = decoded_log_data.value;
                                        if verbose {
                                            println!(
                                                "Decoded log value: {var_value}, log rb: {rb}"
                                            );
                                        }
                                        decoded_logs.push(var_value);
                                    }
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

                    let expected_decoded_test_logs = expected_decoded_test_logs.unwrap_or_default();

                    if !failed.is_empty() {
                        println!("FAILED!! output:\n{output}");
                        panic!(
                            "For {name}\n{} tests failed:\n{}",
                            failed.len(),
                            failed.into_iter().collect::<String>()
                        );
                    } else if expected_decoded_test_logs != decoded_logs {
                        println!("FAILED!! output:\n{output}");
                        panic!(
                            "For {name}\ncollected decoded logs: {decoded_logs:?}\nexpected decoded logs: {expected_decoded_test_logs:?}"
                        );
                    }

                    r
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
    let mut tests = discover_test_tomls(run_config)?;
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

    if filter_config.exclude_std {
        tests.retain(|t| exclude_tests_dependency(t, "std"));
    }
    if filter_config.abi_only {
        tests.retain(|t| t.validate_abi);
    }
    if filter_config.contract_only {
        tests.retain(|t| t.category == TestCategory::RunsWithContract);
    }
    if filter_config.forc_test_only {
        tests.retain(|t| t.category == TestCategory::UnitTestsPass);
    }
    if filter_config.first_only && !tests.is_empty() {
        tests = vec![tests.remove(0)];
    }

    // Run tests
    let context = TestContext {
        deployed_contracts: Default::default(),
    };
    let mut number_of_tests_executed = 0;
    let mut number_of_tests_failed = 0;
    let mut failed_tests = vec![];

    let start_time = Instant::now();
    for (i, test) in tests.into_iter().enumerate() {
        let cur_profile = if run_config.release {
            BuildProfile::RELEASE
        } else {
            BuildProfile::DEBUG
        };

        if test.unsupported_profiles.contains(&cur_profile) {
            continue;
        }

        let name = if let Some(suffix) = test.suffix.as_ref() {
            format!("{} ({})", test.name, suffix)
        } else {
            test.name.clone()
        };

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

        match result {
            Err(err) => {
                println!(" {}", "failed".red().bold());
                println!("{}", textwrap::indent(err.to_string().as_str(), "     "));
                println!("{}", textwrap::indent(&output, "          "));
                number_of_tests_failed += 1;
                failed_tests.push(name);
            }
            Ok(r) => {
                if let Some(size) = r.size {
                    print!(" {} bytes ", size);
                }

                 if let Some(gas) = r.gas {
                    print!(" {} gas used ", gas);
                }

                println!(" {}", "ok".green().bold());

                // If verbosity is requested then print it out.
                if run_config.verbose && !output.is_empty() {
                    println!("{}", textwrap::indent(&output, "     "));
                }
            }
        }

        number_of_tests_executed += 1;
    }
    let duration = Instant::now().duration_since(start_time);

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
            "Sway tests result: {}. {} total, {} passed; {} failed; {} disabled [test duration: {}]",
            if number_of_tests_failed == 0 {
                "ok".green().bold()
            } else {
                "failed".red().bold()
            },
            total_number_of_tests,
            number_of_tests_executed - number_of_tests_failed,
            number_of_tests_failed,
            disabled_tests.len(),
            util::duration_to_str(&duration)
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

fn exclude_tests_dependency(t: &TestDescription, dep: &str) -> bool {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let tests_root_dir = format!("{manifest_dir}/src/e2e_vm_tests/test_programs");
    let file_name = &t.name;
    let manifest_path = format!("{tests_root_dir}/{file_name}");
    match ManifestFile::from_dir(manifest_path) {
        Ok(manifest_file) => {
            let member_manifests = manifest_file.member_manifests().unwrap();
            !member_manifests.iter().any(|(_name, manifest)| {
                manifest
                    .dependencies
                    .as_ref()
                    .is_some_and(|map| map.contains_key(dep))
            })
        }
        Err(_) => true,
    }
}

fn discover_test_tomls(run_config: &RunConfig) -> Result<Vec<TestDescription>> {
    let mut descriptions = vec![];

    let pattern = format!(
        "{}/src/e2e_vm_tests/test_programs/**/test*.toml",
        env!("CARGO_MANIFEST_DIR")
    );

    for entry in glob::glob(&pattern)
        .expect("Failed to read glob pattern")
        .flatten()
    {
        let t = parse_test_toml(&entry, run_config)?;
        descriptions.push(t);
    }

    Ok(descriptions)
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

fn parse_test_toml(path: &Path, run_config: &RunConfig) -> Result<TestDescription> {
    let toml_content_str = std::fs::read_to_string(path)?;

    let mut file_check = FileCheck(toml_content_str.clone());
    let checker = file_check.build()?;

    let mut toml_content = toml_content_str.parse::<toml::Value>()?;

    if !toml_content.is_table() {
        bail!("Malformed test description.");
    }

    // use test.toml as base if this test has a suffix
    {
        let toml_content_map = toml_content.as_table_mut().unwrap();
        if path.file_name().and_then(|x| x.to_str()) != Some("test.toml") {
            let base_test_toml = path.parent().unwrap().join("test.toml");
            let base_test_toml = std::fs::read_to_string(base_test_toml)?;
            let base_toml_content = base_test_toml.parse::<toml::Value>()?;
            match base_toml_content {
                toml::Value::Table(map) => {
                    for (k, v) in map {
                        let _ = toml_content_map.entry(&k).or_insert(v);
                    }
                }
                _ => bail!("Malformed base test description (see test.toml)."),
            }
        };
    }

    let mut run_config = run_config.clone();

    // To keep the current test.toml compatible we check if a field named "experimental" exists
    // or not. If it does not, we keep the current behaviour.
    // If it does, we ignore the experimental flags from the CLI and use the one from the toml file.
    // TODO: this backwards compatibility can be removed after all tests migrate to new version
    let (has_experimental_field, experimental) =
        if let Some(toml_experimental) = toml_content.get("experimental") {
            run_config.experimental = CliFields::default();

            let mut experimental = ExperimentalFeatures::default();
            for (k, v) in toml_experimental.as_table().unwrap() {
                let v = v.as_bool().unwrap();
                experimental.set_enabled_by_name(k, v).unwrap();

                if v {
                    run_config
                        .experimental
                        .experimental
                        .push(k.parse().unwrap());
                } else {
                    run_config
                        .experimental
                        .no_experimental
                        .push(k.parse().unwrap());
                }
            }
            (true, experimental)
        } else {
            let mut experimental = ExperimentalFeatures::default();
            for f in &run_config.experimental.no_experimental {
                experimental.set_enabled(*f, false);
            }
            for f in &run_config.experimental.experimental {
                experimental.set_enabled(*f, true);
            }
            (false, experimental)
        };

    // if new encoding is on, allow a "category_new_encoding"
    // for tests that should have different categories
    let category = if !has_experimental_field && experimental.new_encoding {
        toml_content
            .get("category_new_encoding")
            .or_else(|| toml_content.get("category"))
    } else {
        toml_content.get("category")
    };
    let category = category
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

    let expected_decoded_test_logs = if let Some(toml::Value::Array(a)) =
        toml_content.get("expected_decoded_test_logs")
    {
        if category != TestCategory::UnitTestsPass {
            bail!("`expected_decoded_test_logs` is only valid for `unt_tests_pass` type of tests")
        }
        a.iter()
            .map(|elem| elem.as_str().map(|s| s.to_string()))
            .collect::<Option<Vec<_>>>()
    } else {
        None
    };

    // Abort early if we find a FailsToCompile test without any Checker directives.
    if category == TestCategory::FailsToCompile && checker.is_empty() {
        bail!("'fail' tests must contain some FileCheck verification directives.");
    }

    // We have some tests on old and new encoding that return different warnings.
    // There is no easy way to write `test.toml` to support both, and the effort
    // for such support is also questionable since we do not want to support
    // the old encoding anymore, and currently we do not have any other configurations.
    // Currently, we will simply ignore the `FileCheck` directives if the test category
    // is not "fails" and the "category_new_encoding" is explicitly specified.
    if !has_experimental_field
        && toml_content.get("category_new_encoding").is_some()
        && category != TestCategory::FailsToCompile
    {
        file_check = FileCheck("".into());
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
                    bail!("Expected 'script_data_new_encoding' to be a hex string.");
                }
                _ => None,
            };

            (
                script_data,
                if has_experimental_field {
                    None
                } else {
                    script_data_new_encoding
                },
            )
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
            get_expected_result("expected_result", &toml_content)
        }
        TestCategory::Compiles
        | TestCategory::FailsToCompile
        | TestCategory::UnitTestsPass
        | TestCategory::Disabled => None,
    };

    let expected_result_new_encoding = match (
        &category,
        get_expected_result("expected_result_new_encoding", &toml_content),
    ) {
        (TestCategory::Runs | TestCategory::RunsWithContract, Some(value))
            if !has_experimental_field =>
        {
            Some(value)
        }
        _ => None,
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

    let logs = toml_content
        .get("logs")
        .and_then(|x| x.as_str())
        .map(|x| x.to_string());

    Ok(TestDescription {
        name,
        suffix: path.file_name().unwrap().to_str().map(|x| x.to_string()),
        category,
        script_data,
        script_data_new_encoding,
        witness_data,
        expected_result,
        expected_result_new_encoding,
        expected_warnings,
        contract_paths,
        validate_abi,
        validate_storage_slots,
        supported_targets,
        unsupported_profiles,
        checker: file_check,
        run_config,
        expected_decoded_test_logs,
        experimental,
        has_experimental_field,
        logs,
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

fn get_expected_result(key: &str, toml_content: &toml::Value) -> Option<TestResult> {
    fn get_action_value(action: &toml::Value, expected_value: &toml::Value) -> Result<TestResult> {
        match (action.as_str(), expected_value) {
            // A simple integer value.
            (Some("return"), toml::Value::Integer(v)) => Ok(TestResult::Return(*v as u64)),

            // Also a simple integer value, but is a result from a contract call.
            (Some("result"), toml::Value::Integer(v)) => Ok(TestResult::Result(*v as Word)),

            // A bytes32 value.
            (Some("return_data"), toml::Value::String(v)) => hex::decode(v.replace(' ', ""))
                .map(TestResult::ReturnData)
                .map_err(|e| anyhow!("Invalid hex value for 'return_data': {}", e)),

            // Revert with a specific code.
            (Some("revert"), toml::Value::Integer(v)) => Ok(TestResult::Revert(*v as u64)),

            _otherwise => Err(anyhow!("Malformed action value: {action} {expected_value}")),
        }
    }

    let expected_result_table = toml_content.get(key)?;
    let action = expected_result_table.get("action")?;
    let value = expected_result_table.get("value")?;
    get_action_value(action, value).ok()
}
