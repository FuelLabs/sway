// Please take a look in test_programs/README.md for details on how these tests work.

mod harness;
mod harness_callback_handler;
mod util;

use crate::e2e_vm_tests::harness::run_and_capture_output;
use crate::{FilterConfig, RunConfig};

use anyhow::{anyhow, bail, Result};
use chrono::Local;
use colored::*;
use core::fmt;
use forc_pkg::manifest::{GenericManifestFile, ManifestFile};
use forc_pkg::BuildProfile;
use forc_test::ecal::Syscall;
use forc_util::tx_utils::decode_log_data;
use fuel_vm::fuel_tx;
use fuel_vm::prelude::*;
use git2::Repository;
use rand::{Rng, SeedableRng};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashSet};
use std::fs::File;
use std::io::stdout;
use std::io::Write;
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::time::{Duration, Instant};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use sway_core::source_map::SourceMap;
use sway_core::BuildTarget;
use sway_features::{CliFields, ExperimentalFeatures};
use tokio::sync::Mutex;
use tracing::Instrument;

use self::util::VecExt;

use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug)]
enum TestCategory {
    Compiles,
    FailsToCompile,
    Runs,
    RunsWithContract,
    IrRuns,
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
    test_toml_path: String,
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
    /// Signing key to be used if the test is of [TestCategory::RunsWithContract].
    /// `None` if the test has any other [TestCategory].
    signing_key: Option<SecretKey>,
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

impl TestDescription {
    pub fn display_name(&self) -> Cow<str> {
        if let Some(suffix) = self.suffix.as_ref() {
            format!("{} ({})", self.name, suffix).into()
        } else {
            self.name.as_str().into()
        }
    }

    pub fn expect_signing_key(&self) -> &SecretKey {
        self.signing_key
            .as_ref()
            .expect("`RunsWithContract` test must have a signing key defined")
    }
}

#[derive(PartialEq, Eq, Hash)]
struct DeployedContractKey {
    pub contract_path: String,
    pub new_encoding: bool,
}

#[derive(Serialize, Deserialize)]
struct GasUsage {
    /// The name of the unit test, or `None` if it is the gas usage of a script run.
    pub unit_test_name: Option<String>,
    pub gas_used: usize,
}

impl GasUsage {
    pub fn new(gas_used: usize) -> Self {
        Self {
            unit_test_name: None,
            gas_used,
        }
    }

    pub fn with_unit_test_name(unit_test_name: String, gas_used: usize) -> Self {
        Self {
            unit_test_name: Some(unit_test_name),
            gas_used,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct BytecodeSize {
    /// The name of the compiled package if more than one package is compiled
    /// within a test, or `None` if it is a single package whose name is the
    /// same as the test name.
    pub package_name: Option<String>,
    pub bytecode_size: usize,
}

impl BytecodeSize {
    pub fn new(bytecode_size: usize) -> Self {
        Self {
            package_name: None,
            bytecode_size,
        }
    }

    pub fn with_package_name(package_name: String, bytecode_size: usize) -> Self {
        Self {
            package_name: Some(package_name),
            bytecode_size,
        }
    }
}

/// Performance data, bytecode sizes and gas usages,
/// collected during the run of a single test.
///
/// Performance data can be collected for tests of
/// these categories: "compile", "run", "unit_tests_pass".
///
/// A single test can have several bytecode sizes, if a
/// workspace is "compiled", and several gas usages, if
/// "unit_tests_pass" is run.
#[derive(Serialize, Deserialize)]
struct TestPerfData {
    pub test_display_name: String,
    pub bytecode_sizes: Vec<BytecodeSize>,
    pub gas_usages: Vec<GasUsage>,
}

impl TestPerfData {
    fn new(test_display_name: String) -> Self {
        Self {
            test_display_name,
            bytecode_sizes: vec![],
            gas_usages: vec![],
        }
    }
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
                            let data = data.as_ref().unwrap();
                            let payload = data
                                .as_ref()
                                .get(8..)
                                .expect("log data shorter than 8 byte header");
                            let s = std::str::from_utf8(payload).unwrap();

                            text_log.push_str(s);
                        }
                        1 => {
                            let data = data.as_ref().unwrap();
                            let s = u64::from_be_bytes(data.as_ref().try_into().unwrap());

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

impl TestContext {
    async fn deploy_contract(
        &self,
        run_config: &RunConfig,
        contract_path: String,
        signing_key: &SecretKey,
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
            let contract_id =
                harness::deploy_contract(contract_path.as_str(), run_config, signing_key).await?;
            deployed_contracts.insert(key, contract_id);
            contract_id
        })
    }

    async fn run(
        &self,
        test: &TestDescription,
        output: &mut String,
        verbose: bool,
    ) -> Result<TestPerfData> {
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

        let mut perf_data = TestPerfData::new(test.display_name().into());

        match category {
            TestCategory::Runs => {
                let expected_result = expected_result
                    .as_ref()
                    .expect("No expected result found. This is likely because the `test.toml` is missing either an \"expected_result_new_encoding\" or \"expected_result\" entry.");

                let (result, out) =
                    run_and_capture_output(|| harness::compile_to_bytes(name, run_config, logs))
                        .await;
                *output = out;

                if let Ok(result) = result.as_ref() {
                    let packages = match result {
                        forc_pkg::Built::Package(p) => [p.clone()].to_vec(),
                        forc_pkg::Built::Workspace(p) => p.clone(),
                    };

                    for p in packages {
                        let bytecode_len = p.bytecode.bytes.len();

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

                check_file_checker(checker, name, output)?;

                let compiled = result?;

                let compiled = match compiled {
                    forc_pkg::Built::Package(built_pkg) => built_pkg.as_ref().clone(),
                    forc_pkg::Built::Workspace(_) => {
                        panic!("workspaces are not supported in the test suite yet")
                    }
                };

                perf_data
                    .bytecode_sizes
                    .push(BytecodeSize::new(compiled.bytecode.bytes.len()));

                if compiled.warnings.len() > *expected_warnings as usize {
                    return Err(anyhow::Error::msg(format!(
                        "Expected warnings: {expected_warnings}\nActual number of warnings: {}",
                        compiled.warnings.len()
                    )));
                }

                let result = harness::runs_in_vm(
                    compiled.clone(),
                    script_data.clone(),
                    witness_data.clone(),
                )?;

                let actual_result = match result {
                    harness::VMExecutionResult::Fuel(state, receipts, ecal) => {
                        print_receipts(output, &receipts);

                        let gas_used = receipts.iter().find_map(|r| {
                            if let Receipt::ScriptResult { gas_used, .. } = r {
                                Some(*gas_used)
                            } else {
                                None
                            }
                        });

                        if let Some(gas_used) = gas_used {
                            perf_data.gas_usages.push(GasUsage::new(gas_used as usize));
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

                if &actual_result != expected_result {
                    return Err(anyhow::Error::msg(format!(
                        "expected: {expected_result:?}\nactual: {actual_result:?}"
                    )));
                } else if *validate_abi {
                    let (result, out) = run_and_capture_output(|| async {
                        harness::test_json_abi(
                            name,
                            &compiled,
                            experimental.new_encoding,
                            run_config.update_output_files,
                            suffix,
                            *has_experimental_field,
                            run_config.release,
                        )
                    })
                    .await;

                    output.push_str(&out);
                    result?;
                }
            }
            TestCategory::IrRuns => {
                let expected_result = expected_result
                    .as_ref()
                    .expect("No expected result found. This is likely because the `test.toml` is missing either an \"expected_result_new_encoding\" or \"expected_result\" entry.");

                let engines = sway_core::Engines::default();

                let manifest_dir = env!("CARGO_MANIFEST_DIR");
                let main_file =
                    format!("{manifest_dir}/src/e2e_vm_tests/test_programs/{name}/src/main.ir");

                let compiled = forc_pkg::compile_ir(
                    Path::new(&main_file),
                    &engines,
                    *experimental,
                    &mut SourceMap::new(),
                )?;

                // Code taken from harness::runs_in_vm
                let result = || -> Result<harness::VMExecutionResult> {
                    let storage = MemoryStorage::default();

                    let rng = &mut rand::rngs::StdRng::seed_from_u64(2322u64);
                    let maturity = 1.into();
                    let script_data = script_data.clone().unwrap_or_default();
                    let block_height = (u32::MAX >> 1).into();
                    // The default max length is 1MB which isn't enough for the bigger tests.
                    let max_size = 64 * 1024 * 1024;
                    let script_params = ScriptParameters::DEFAULT
                        .with_max_script_length(max_size)
                        .with_max_script_data_length(max_size);
                    let tx_params = TxParameters::DEFAULT.with_max_size(max_size);
                    let params =
                        ConsensusParameters::V1(consensus_parameters::ConsensusParametersV1 {
                            script_params,
                            tx_params,
                            ..Default::default()
                        });
                    let mut tb = TransactionBuilder::script(compiled.bytes, script_data);

                    tb.with_params(params)
                        .add_unsigned_coin_input(
                            SecretKey::random(rng),
                            rng.r#gen(),
                            1,
                            Default::default(),
                            rng.r#gen(),
                        )
                        .maturity(maturity);

                    if let Some(witnesses) = witness_data.clone() {
                        for witness in witnesses {
                            tb.add_witness(witness.into());
                        }
                    }
                    let gas_price = 0;
                    let consensus_params = tb.get_params().clone();

                    let params = ConsensusParameters::default();
                    // Temporarily finalize to calculate `script_gas_limit`
                    let tmp_tx = tb.clone().finalize();
                    // Get `max_gas` used by everything except the script execution. Add `1` because of rounding.
                    let max_gas = tmp_tx
                        .max_gas(consensus_params.gas_costs(), consensus_params.fee_params())
                        + 1;
                    // Increase `script_gas_limit` to the maximum allowed value.
                    tb.script_gas_limit(consensus_params.tx_params().max_gas_per_tx() - max_gas);

                    let tx = tb
                        .finalize_checked(block_height)
                        .into_ready(gas_price, params.gas_costs(), params.fee_params(), None)
                        .map_err(|e| anyhow::anyhow!("{e:?}"))?;

                    let mem_instance = MemoryInstance::new();
                    let mut i: Interpreter<_, _, _, forc_test::ecal::EcalSyscallHandler> =
                        Interpreter::with_storage(mem_instance, storage, Default::default());
                    let transition = i.transact(tx).map_err(anyhow::Error::msg)?;

                    Ok(harness::VMExecutionResult::Fuel(
                        *transition.state(),
                        transition.receipts().to_vec(),
                        Box::new(i.ecal_state().clone()),
                    ))
                };
                let result = result()?;
                let actual_result = match result {
                    harness::VMExecutionResult::Fuel(state, receipts, ecal) => {
                        print_receipts(output, &receipts);

                        let gas_used = receipts.iter().find_map(|r| {
                            if let Receipt::ScriptResult { gas_used, .. } = r {
                                Some(*gas_used)
                            } else {
                                None
                            }
                        });

                        if let Some(gas_used) = gas_used {
                            perf_data.gas_usages.push(GasUsage::new(gas_used as usize));
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

                if &actual_result != expected_result {
                    return Err(anyhow::Error::msg(format!(
                        "expected: {expected_result:?}\nactual: {actual_result:?}"
                    )));
                }
            }

            TestCategory::Compiles => {
                let (result, out) =
                    run_and_capture_output(|| harness::compile_to_bytes(name, run_config, logs))
                        .await;
                *output = out;

                let (is_single_package, compiled_pkgs) = match result? {
                    forc_pkg::Built::Package(built_pkg) => {
                        if built_pkg.warnings.len() > *expected_warnings as usize {
                            return Err(anyhow::Error::msg(format!(
                                "Expected warnings: {expected_warnings}\nActual number of warnings: {}",
                                built_pkg.warnings.len()
                            )));
                        }
                        (true, vec![(name.clone(), built_pkg.as_ref().clone())])
                    }
                    forc_pkg::Built::Workspace(built_workspace) => (
                        false,
                        built_workspace
                            .iter()
                            .map(|built_pkg| {
                                (
                                    built_pkg.descriptor.pinned.name.clone(),
                                    built_pkg.as_ref().clone(),
                                )
                            })
                            .collect(),
                    ),
                };

                for (name, built_pkg) in &compiled_pkgs {
                    if is_single_package {
                        perf_data
                            .bytecode_sizes
                            .push(BytecodeSize::new(built_pkg.bytecode.bytes.len()));
                    } else {
                        perf_data
                            .bytecode_sizes
                            .push(BytecodeSize::with_package_name(
                                name.clone(),
                                built_pkg.bytecode.bytes.len(),
                            ));
                    }
                }

                check_file_checker(checker, name, output)?;

                if *validate_abi {
                    for (name, built_pkg) in &compiled_pkgs {
                        let (result, out) = run_and_capture_output(|| async {
                            harness::test_json_abi(
                                name,
                                built_pkg,
                                experimental.new_encoding,
                                run_config.update_output_files,
                                suffix,
                                *has_experimental_field,
                                run_config.release,
                            )
                        })
                        .await;
                        output.push_str(&out);
                        result?;
                    }
                }

                if *validate_storage_slots {
                    for (name, built_pkg) in &compiled_pkgs {
                        let (result, out) = run_and_capture_output(|| async {
                            harness::test_json_storage_slots(name, built_pkg, suffix)
                        })
                        .await;
                        result?;
                        output.push_str(&out);
                    }
                }
            }

            TestCategory::FailsToCompile => {
                let (result, out) =
                    run_and_capture_output(|| harness::compile_to_bytes(name, run_config, logs))
                        .await;

                *output = out;

                if result.is_ok() {
                    if verbose {
                        eprintln!("[{output}]");
                    }

                    return Err(anyhow::Error::msg("Test compiles but is expected to fail"));
                } else {
                    check_file_checker(checker, name, output)?;
                }
            }

            TestCategory::RunsWithContract => {
                if contract_paths.is_empty() {
                    panic!(
                        "For {name}\n\
                        One or more contract paths are required for 'run_on_node' tests."
                    );
                }

                let signing_key = test.expect_signing_key();
                let mut contract_ids = Vec::new();
                for contract_path in contract_paths.clone() {
                    let (result, out) = run_and_capture_output(|| async {
                        self.deploy_contract(run_config, contract_path, signing_key)
                            .await
                    })
                    .await;
                    output.push_str(&out);
                    contract_ids.push(result);
                }
                let contract_ids = contract_ids.into_iter().collect::<Result<Vec<_>, _>>()?;

                let (result, out) =
                    harness::runs_on_node(name, run_config, &contract_ids, signing_key).await;

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
                        println!("Deployed contract: {}", format!("{:#x}", cid).bold(),);
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
                    Receipt::Return { val, .. } => match expected_result.as_ref().unwrap() {
                        TestResult::Result(v) => {
                            if *v != *val {
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
                    Receipt::ReturnData { data, .. } => match expected_result.as_ref().unwrap() {
                        TestResult::ReturnData(v) => {
                            let actual = data.as_ref().map(|bytes| bytes.as_ref()).unwrap();
                            if v.as_slice() != actual {
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
            }

            TestCategory::UnitTestsPass => {
                let (result, out) =
                    harness::compile_and_run_unit_tests(name, run_config, true).await;
                *output = out;

                let mut decoded_logs = vec![];

                result.map(|tested_pkgs| {
                    let mut failed = vec![];
                    for pkg in tested_pkgs {
                        if !pkg.tests.is_empty() {
                            println!();
                        }
                        if let Some(bytecode_size_without_tests) = pkg.built.bytecode_without_tests.as_ref().map(|bc| bc.bytes.len()) {
                            perf_data.bytecode_sizes.push(BytecodeSize::new(bytecode_size_without_tests));
                        }
                        for test in pkg.tests.into_iter() {
                            perf_data.gas_usages.push(GasUsage::with_unit_test_name(
                                test.name.clone(),
                                test.gas_used as usize,
                            ));
                            if verbose {
                                // "test incorrect_def_modeling ... ok (17.673µs, 59 gas)"
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

                    let expected_decoded_test_logs = if let Some(expected_decoded_test_logs) = expected_decoded_test_logs.as_ref() {
                        expected_decoded_test_logs
                    } else {
                        &vec![]
                    };

                    if !failed.is_empty() {
                        println!("FAILED!! output:\n{output}");
                        panic!(
                            "For {name}\n{} tests failed:\n{}",
                            failed.len(),
                            failed.into_iter().collect::<String>()
                        );
                    } else if expected_decoded_test_logs != &decoded_logs {
                        println!("FAILED!! output:\n{output}");
                        panic!(
                            "For {name}\ncollected decoded logs: {decoded_logs:?}\nexpected decoded logs: {expected_decoded_test_logs:?}"
                        );
                    }
                })?;
            }

            category => {
                return Err(anyhow::Error::msg(format!(
                    "Unexpected test category: {category:?}",
                )))
            }
        }

        Ok(perf_data)
    }
}

struct TestsInRun {
    total_number_of_tests: usize,
    skipped_tests: Vec<TestDescription>,
    disabled_tests: Vec<TestDescription>,
    included_tests: Vec<TestDescription>,
    excluded_tests: Vec<TestDescription>,
    tests_to_run: Vec<TestDescription>,
}

/// Performance data collected during the entire test run,
/// for all the tests that were executed.
struct RunPerfData {
    collect_perf_data: bool,
    build_profile_name: String,
    perf_data: Vec<TestPerfData>,
}

impl RunPerfData {
    fn new(run_config: &RunConfig) -> Self {
        Self {
            collect_perf_data: run_config.perf,
            build_profile_name: if run_config.release {
                "release"
            } else {
                "debug"
            }
            .to_string(),
            perf_data: vec![],
        }
    }

    fn add_perf_data(&mut self, perf_data: TestPerfData) {
        if self.collect_perf_data {
            self.perf_data.push(perf_data);
        }
    }

    fn is_empty(&self) -> bool {
        self.perf_data.is_empty()
    }

    /// Consumes the collected performance data and returns:
    /// - build profile name,
    /// - vector containing full E2E test names and their bytecode sizes,
    /// - vector containing full unit test names and their gas usages.
    ///
    /// Both vectors are ordered by their test names.
    #[allow(clippy::type_complexity)]
    fn consume(self) -> (String, Vec<(String, usize)>, Vec<(String, usize)>) {
        let mut bytecode_sizes = vec![];
        let mut gas_usages = vec![];

        for perf_data in self.perf_data {
            for bytecode_size in perf_data.bytecode_sizes {
                let full_test_name = match bytecode_size.package_name {
                    Some(package_name) => {
                        format!("{}::{package_name}", perf_data.test_display_name)
                    }
                    None => perf_data.test_display_name.clone(),
                };
                bytecode_sizes.push((full_test_name, bytecode_size.bytecode_size));
            }
            for gas_usage in perf_data.gas_usages {
                let full_unit_test_name = match gas_usage.unit_test_name {
                    Some(unit_test_name) => {
                        format!("{}::{unit_test_name}", perf_data.test_display_name)
                    }
                    None => perf_data.test_display_name.clone(),
                };
                gas_usages.push((full_unit_test_name, gas_usage.gas_used));
            }
        }

        gas_usages.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));
        bytecode_sizes.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

        (self.build_profile_name, bytecode_sizes, gas_usages)
    }
}

impl TestsInRun {
    fn new(filter_config: &FilterConfig, run_config: &RunConfig) -> Result<Self> {
        let all_tests = discover_test_tomls(run_config)?;
        let total_number_of_tests = all_tests.len();

        let mut tests_to_run = all_tests;
        let skipped_tests = filter_config
            .skip_until
            .as_ref()
            .map(|skip_until| {
                let mut found = false;
                tests_to_run.retain_and_get_removed(|t| {
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
        let disabled_tests =
            tests_to_run.retain_and_get_removed(|t| t.category != TestCategory::Disabled);
        let included_tests = filter_config
            .include
            .as_ref()
            .map(|include| tests_to_run.retain_and_get_removed(|t| include.is_match(&t.name)))
            .unwrap_or_default();
        let excluded_tests = filter_config
            .exclude
            .as_ref()
            .map(|exclude| tests_to_run.retain_and_get_removed(|t| !exclude.is_match(&t.name)))
            .unwrap_or_default();

        if filter_config.no_std_only {
            tests_to_run.retain(|t| exclude_tests_dependency(t, "std"));
        }
        if filter_config.abi_only {
            tests_to_run.retain(|t| t.validate_abi);
        }
        if filter_config.contract_only {
            tests_to_run.retain(|t| t.category == TestCategory::RunsWithContract);
        }
        if filter_config.forc_test_only {
            tests_to_run.retain(|t| t.category == TestCategory::UnitTestsPass);
        }
        if filter_config.perf_only {
            tests_to_run.retain(|t| {
                matches!(
                    t.category,
                    TestCategory::Runs | TestCategory::Compiles | TestCategory::UnitTestsPass
                )
            });
        }
        let cur_profile = if run_config.release {
            BuildProfile::RELEASE
        } else {
            BuildProfile::DEBUG
        };
        tests_to_run.retain(|test| !test.unsupported_profiles.contains(&cur_profile));
        tests_to_run.retain(|test| test.supported_targets.contains(&run_config.build_target));

        if filter_config.first_only {
            tests_to_run.truncate(1);
        }

        // Assign signing keys to all "run_on_node" tests.
        // Below keys are taken from the `fuel-core` repo: fuel-core/crates/chain-config/src/config/state.rs
        const TESTNET_WALLET_SECRETS: [&str; 5] = [
            "0xde97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c",
            "0x37fa81c84ccd547c30c176b118d5cb892bdb113e8e80141f266519422ef9eefd",
            "0x862512a2363db2b3a375c0d4bbbd27172180d89f23f2e259bac850ab02619301",
            "0x976e5c3fa620092c718d852ca703b6da9e3075b9f2ecb8ed42d9f746bf26aafb",
            "0x7f8a325504e7315eda997db7861c9447f5c3eff26333b20180475d94443a10c6",
        ];
        for (signing_key_index, test) in tests_to_run
            .iter_mut()
            .filter(|test| test.category == TestCategory::RunsWithContract)
            .enumerate()
        {
            let secret = TESTNET_WALLET_SECRETS[signing_key_index % TESTNET_WALLET_SECRETS.len()];
            test.signing_key = Some(SecretKey::from_str(secret).unwrap());
        }

        Ok(Self {
            total_number_of_tests,
            skipped_tests,
            disabled_tests,
            included_tests,
            excluded_tests,
            tests_to_run,
        })
    }
}

pub async fn run_exact(exact: &str, run_config: &RunConfig) -> Result<()> {
    let mut test = parse_test_toml(Path::new(exact), run_config)?;
    if test.category == TestCategory::RunsWithContract {
        let Some(signing_key) = std::env::var("RUNS_ON_NODE_TEST_SIGNING_KEY")
            .ok()
            .and_then(|key_str| SecretKey::from_str(&key_str).ok())
        else {
            return Err(anyhow::Error::msg(
                "Environment variable `RUNS_ON_NODE_TEST_SIGNING_KEY` must be set to run 'run_on_node' tests using `--exact` flag.",
            ));
        };
        test.signing_key = Some(signing_key);
    }

    let context = TestContext {
        deployed_contracts: Default::default(),
    };

    let mut output = String::new();
    let perf_data = context.run(&test, &mut output, run_config.verbose).await?;

    if run_config.perf {
        // Print the serialized performance data to piped stdout, followed by a newline.
        let mut std_out = std::io::stdout();
        serde_json::to_writer(&mut std_out, &perf_data)?;
        writeln!(&mut std_out)?;
    }

    Ok(())
}

pub async fn run_in_parallel(filter_config: &FilterConfig, run_config: &RunConfig) -> Result<()> {
    let mut tests_in_run = TestsInRun::new(filter_config, run_config)?;

    // Build common CLI args to pass to each subprocess.
    let mut common_args = vec![];
    common_args.push("--build-target".to_string());
    common_args.push(run_config.build_target.to_string());
    if run_config.locked {
        common_args.push("--locked".to_string());
    }
    if run_config.release {
        common_args.push("--release".to_string());
    }
    if run_config.update_output_files {
        common_args.push("--update-output-files".to_string());
    }
    if run_config.perf {
        common_args.push("--perf".to_string());
    }
    if let Some(experimental) = run_config.experimental.experimental_as_cli_string() {
        common_args.push("--experimental".to_string());
        common_args.push(experimental);
    }
    if let Some(no_experimental) = run_config.experimental.no_experimental_as_cli_string() {
        common_args.push("--no-experimental".to_string());
        common_args.push(no_experimental);
    }

    // Running tests of all test categories in parallel is safe by default, except for "run_on_node" tests.
    //
    // All other test categories ("compiles", "fails_to_compile", "runs", "forc_test") do not produce
    // compiler output files that could interfere with each other when run in parallel (e.g., JSON ABI files,
    // storage slots files, etc) for different `test.<feature>.toml`s in the same test. Also, the
    // storage slots JSON and ABI JSON files created to compare with the corresponding oracles,
    // are given unique names based on the test name and the profile (debug/release).
    //
    // However, "run_on_node" tests deploy contracts to a local node, and if multiple such tests
    // are run in parallel, they will try to deploy a same contract multiple times. For sequential
    // execution, the `TestContext` caches deployed contracts by their path, so that each contract
    // is deployed only once per test run. But for parallel execution, this cache cannot be shared
    // across multiple processes. Therefore, to avoid multiple deployments of the same contract
    // from different processes, we run "run_on_node" tests in parallel only if they don't share
    // contracts. Those that share contracts are run sequentially.
    //
    // Moreover, even for the "run_on_node" tests that don't share contracts, we still need to ensure that
    // each test uses a different wallet/signing key, so that transactions do not collide with each other,
    // if using the same wallet/signing key in parallel.

    // Remove "run_on_node" tests from the main list of tests to run in parallel.
    let mut run_on_node_tests = tests_in_run
        .tests_to_run
        .retain_and_get_removed(|test| test.category != TestCategory::RunsWithContract);

    // Determine which "run_on_node" tests share contracts with each other.
    // Maps contract path to the list of tests that deploy it.
    let mut contracts_deployed_in_tests = HashMap::<String, Vec<_>>::new();
    run_on_node_tests.iter().for_each(|test| {
        test.contract_paths.iter().for_each(|contract_path| {
            contracts_deployed_in_tests
                .entry(contract_path.clone())
                .or_default()
                .push(test);
        });
    });
    let tests_sharing_contracts: HashSet<_> = contracts_deployed_in_tests
        .values()
        .filter(|tests| tests.len() > 1)
        .flat_map(|tests| tests.iter())
        .map(|t| t.test_toml_path.clone())
        .collect();

    // Splitting the tests into three groups:
    //  1. `tests_in_run.tests_to_run` - non-"run_on_node" tests, which can be run in parallel
    //  2. `run_on_node_tests`         - "run_on_node" tests that don't share contracts, which can be run in parallel over different wallets
    //  3. `tests_to_run_sequentially` - "run_on_node" tests that share contracts, which must be run sequentially
    let tests_to_run_sequentially = run_on_node_tests
        .retain_and_get_removed(|test| !tests_sharing_contracts.contains(&test.test_toml_path));

    let run_perf_data = std::sync::Mutex::new(RunPerfData::new(run_config));
    let failed_tests = std::sync::Mutex::new(Vec::<String>::new());
    let start_time = Instant::now();

    // 1. Run non-"run_on_node" tests that can be safely run in parallel.
    run_in_parallel_impl(
        &tests_in_run.tests_to_run.iter().collect::<Vec<_>>(),
        &common_args,
        run_config.perf.then_some(&run_perf_data),
        &failed_tests,
    );

    // 2. Run "run_on_node" tests that don't share contracts in parallel over different wallets.
    let mut run_on_node_tests_per_wallet = HashMap::new();
    for test in run_on_node_tests.iter() {
        let signing_key = *test.expect_signing_key();
        run_on_node_tests_per_wallet
            .entry(signing_key)
            .or_insert_with(Vec::new)
            .push(test);
    }
    let longest_wallet_test_chain_size = run_on_node_tests_per_wallet
        .values()
        .max_by_key(|tests| tests.len())
        .map(|tests| tests.len())
        .unwrap_or(0);
    let parallel_test_groups = (0..longest_wallet_test_chain_size)
        .map(|i| {
            run_on_node_tests_per_wallet
                .values()
                .filter_map(|tests| tests.get(i))
                .cloned()
                .collect::<Vec<_>>()
        })
        .filter(|tests| !tests.is_empty())
        .collect::<Vec<_>>();

    parallel_test_groups.iter().for_each(|tests_in_run| {
        run_in_parallel_impl(
            tests_in_run,
            &common_args,
            run_config.perf.then_some(&run_perf_data),
            &failed_tests,
        );
    });

    // 3. Run sequentially "run_on_node" tests that share contracts.
    let context = TestContext {
        deployed_contracts: Default::default(),
    };

    for test in tests_to_run_sequentially.iter() {
        let name = test.display_name();

        let mut output = String::new();
        let result = context.run(test, &mut output, run_config.verbose).await;

        if let Ok(test_perf_data) = result {
            println!("  ✅ Passed: {name}");
            run_perf_data.lock().unwrap().add_perf_data(test_perf_data);
        } else {
            println!("  ❌ Failed: {name}");
            failed_tests.lock().unwrap().push(name.into());
        }
    }

    let duration = Instant::now().duration_since(start_time);

    // To ensure proper statistics printed in the results, get the list of tests that
    // were run separately and add them back to the list of tests to run.
    tests_in_run.tests_to_run.extend(run_on_node_tests);
    tests_in_run.tests_to_run.extend(tests_to_run_sequentially);

    output_run_perf_data(run_perf_data.into_inner().unwrap())?;

    print_run_results(
        &tests_in_run,
        filter_config,
        &failed_tests.into_inner().unwrap(),
        &duration,
    )
}

fn run_in_parallel_impl(
    tests_to_run: &[&TestDescription],
    common_args: &[String],
    run_perf_data: Option<&std::sync::Mutex<RunPerfData>>,
    failed_tests: &std::sync::Mutex<Vec<String>>,
) {
    tests_to_run.par_iter().for_each(|test| {
        fn get_test_perf_data_from_stdout(output: &[u8]) -> Option<TestPerfData> {
            // Just in case we have some extra output, we try to find
            // the first '{"test_display_name":' and parse JSON from there
            // until the first '\n' after that.
            const JSON_START: &[u8] = b"{\"test_display_name\":";
            let start_index = output
                .windows(JSON_START.len())
                .position(|window| window == JSON_START)?;
            let end_index = output[start_index..].iter().position(|&b| b == b'\n')?;
            let output = &output[start_index..end_index + start_index];
            serde_json::from_slice(output).ok()
        }

        let name = test.display_name();

        // Stdout we either ignore in parallel runs, or use for piping/reporting performance data.
        let std_out = if run_perf_data.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        };

        let output = Command::new(std::env::current_exe().unwrap())
            .args(common_args)
            .args(vec!["--exact", &test.test_toml_path])
            .stdout(std_out)
            .stdin(Stdio::null())
            .stderr(Stdio::inherit())
            .env(
                "RUNS_ON_NODE_TEST_SIGNING_KEY",
                test.signing_key
                    .as_ref()
                    .map(|k| k.to_string())
                    .unwrap_or_default(),
            )
            .spawn()
            .unwrap()
            .wait_with_output()
            .unwrap();

        if output.status.success() {
            println!("  ✅ Passed: {name}");
            if let Some(run_perf_data) = run_perf_data {
                if let Some(test_perf_data) = get_test_perf_data_from_stdout(&output.stdout) {
                    run_perf_data.lock().unwrap().add_perf_data(test_perf_data);
                }
            }
        } else {
            println!("  ❌ Failed: {name}");
            failed_tests.lock().unwrap().push(name.into());
        }
    });
}

pub async fn run_sequentially(filter_config: &FilterConfig, run_config: &RunConfig) -> Result<()> {
    let tests_in_run = TestsInRun::new(filter_config, run_config)?;

    let context = TestContext {
        deployed_contracts: Default::default(),
    };

    let mut run_perf_data = RunPerfData::new(run_config);
    let mut failed_tests = Vec::<String>::new();
    let start_time = Instant::now();
    for (i, test) in tests_in_run.tests_to_run.iter().enumerate() {
        let name = test.display_name();

        print!("Testing {} ...", name.clone().bold());
        stdout().flush().unwrap();

        let mut output = String::new();

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
                failed_tests.push(name.into());
            }
            Ok(test_perf_data) => {
                println!(" {}", "ok".green().bold());
                if run_config.verbose && !output.is_empty() {
                    println!("{}", textwrap::indent(&output, "     "));
                }
                if run_config.perf {
                    run_perf_data.add_perf_data(test_perf_data);
                }
            }
        }
    }
    let duration = Instant::now().duration_since(start_time);

    output_run_perf_data(run_perf_data)?;

    print_run_results(&tests_in_run, filter_config, &failed_tests, &duration)
}

fn output_run_perf_data(run_stats: RunPerfData) -> Result<()> {
    if run_stats.is_empty() {
        return Ok(());
    }

    let timestamp = Local::now().format("%m%d%H%M%S").to_string();
    let perf_out_dir = format!("{}/perf_out", env!("CARGO_MANIFEST_DIR"));
    let branch_name =
        current_branch_display_name(&perf_out_dir).unwrap_or_else(|| "unknown-branch".to_string());

    let (build_profile_name, bytecode_sizes, gas_usages) = run_stats.consume();

    let write_stats_to_file = |stats: &[(String, usize)], stats_type: &str| -> Result<()> {
        if !stats.is_empty() {
            let file_name = format!(
                "{perf_out_dir}/{timestamp}-e2e-{}-{build_profile_name}-{branch_name}.csv",
                stats_type.to_lowercase().replace(' ', "-")
            );
            let mut file = File::create(&file_name)?;
            for (test_name, stat_value) in stats {
                writeln!(file, "{test_name},{stat_value}")?;
            }
            println!(
                "{:27} .{}",
                format!("{stats_type} written to: "),
                &file_name.as_str()[file_name.find("/test/perf_out").unwrap()..]
            );
        }

        Ok(())
    };

    println!("_________________________________\n");

    write_stats_to_file(&bytecode_sizes, "Bytecode sizes")?;
    write_stats_to_file(&gas_usages, "Gas usages")?;

    Ok(())
}

fn current_branch_display_name<P: AsRef<Path>>(path: P) -> Option<String> {
    Repository::discover(path)
        .ok()
        .and_then(|repo| {
            repo.head()
                .ok()
                .and_then(|head| head.shorthand().map(|s| s.to_string()))
        })
        .map(|branch_name| branch_name.replace("/", "-"))
}

fn print_run_results(
    tests_in_run: &TestsInRun,
    filter_config: &FilterConfig,
    failed_tests: &[String],
    duration: &Duration,
) -> Result<()> {
    let number_of_tests_executed = tests_in_run.tests_to_run.len();
    let number_of_tests_failed = failed_tests.len();

    if number_of_tests_executed == 0 {
        if let Some(skip_until) = &filter_config.skip_until {
            tracing::info!(
                "Filtered {} test{} with `skip-until` regex: {:?}",
                tests_in_run.skipped_tests.len(),
                if tests_in_run.skipped_tests.len() == 1 {
                    ""
                } else {
                    "s"
                },
                skip_until.to_string(),
            );
        }
        if let Some(include) = &filter_config.include {
            tracing::info!(
                "Filtered {} test{} with `include` regex: {:?}",
                tests_in_run.included_tests.len(),
                if tests_in_run.included_tests.len() == 1 {
                    ""
                } else {
                    "s"
                },
                include.to_string(),
            );
        }
        if let Some(exclude) = &filter_config.exclude {
            tracing::info!(
                "Filtered {} test{} with `exclude` regex: {:?}",
                tests_in_run.excluded_tests.len(),
                if tests_in_run.excluded_tests.len() == 1 {
                    ""
                } else {
                    "s"
                },
                exclude.to_string(),
            );
        }
        if !tests_in_run.disabled_tests.is_empty() {
            tracing::info!(
                "{} test{} disabled.",
                tests_in_run.disabled_tests.len(),
                if tests_in_run.disabled_tests.len() == 1 {
                    " was"
                } else {
                    "s were"
                },
            );
        }
        tracing::warn!(
            "No tests were run. Provided test filters filtered out all {} tests.",
            tests_in_run.total_number_of_tests
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
            tests_in_run.total_number_of_tests,
            number_of_tests_executed - number_of_tests_failed,
            number_of_tests_failed,
            tests_in_run.disabled_tests.len(),
            util::duration_to_str(duration),
        );
        if number_of_tests_failed > 0 {
            tracing::info!("{}", "Failing tests:".red().bold());
            tracing::info!(
                "    {}",
                failed_tests
                    .iter()
                    .map(|failed_test| format!(
                        "{} ... {}",
                        failed_test.bold(),
                        "failed".red().bold()
                    ))
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
            Some("ir_run") => Ok(TestCategory::IrRuns),
            Some("fail") => Ok(TestCategory::FailsToCompile),
            Some("compile") => Ok(TestCategory::Compiles),
            Some("disabled") => Ok(TestCategory::Disabled),
            Some("unit_tests_pass") => Ok(TestCategory::UnitTestsPass),
            None => Err(anyhow!(
                "Malformed category '{category_val}', should be a string."
            )),
            Some(other) => Err(anyhow!("Unknown test category '{other}'. Valid categories are: run, run_on_node, ir_run, fail, compile, disabled, and unit_tests_pass.")),
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
        TestCategory::Runs | TestCategory::RunsWithContract | TestCategory::IrRuns => {
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
        TestCategory::Runs | TestCategory::RunsWithContract | TestCategory::IrRuns => {
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
        TestCategory::Runs | TestCategory::RunsWithContract | TestCategory::IrRuns => {
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
        (
            TestCategory::Runs | TestCategory::RunsWithContract | TestCategory::IrRuns,
            Some(value),
        ) if !has_experimental_field => Some(value),
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
        test_toml_path: path.to_str().unwrap().into(),
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
        signing_key: None, // Not a part of `test.toml` and assigned later.
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
