use crate::e2e_vm_tests::harness_callback_handler::HarnessCallbackHandler;

use super::RunConfig;
use anyhow::{anyhow, bail, Result};
use colored::Colorize;
use forc_client::{
    cmd::{Deploy as DeployCommand, Run as RunCommand},
    op::{deploy, run, DeployedPackage},
    NodeTarget,
};
use forc_pkg::{BuildProfile, Built, BuiltPackage, PrintOpts};
use forc_test::ecal::EcalSyscallHandler;
use fuel_tx::TransactionBuilder;
use fuel_vm::checked_transaction::builder::TransactionBuilderExt;
use fuel_vm::fuel_tx::{self, consensus_parameters::ConsensusParametersV1};
use fuel_vm::interpreter::Interpreter;
use fuel_vm::prelude::*;
use futures::Future;
use normalize_path::NormalizePath as _;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use regex::{Captures, Regex};
use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
};
use sway_core::{asm_generation::ProgramABI, engine_threading::CallbackHandler, BuildTarget};

pub const NODE_URL: &str = "http://127.0.0.1:4000";
pub const SECRET_KEY: &str = "de97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c";

pub(crate) async fn run_and_capture_output<F, Fut, T>(func: F) -> (T, String)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    let mut output = String::new();

    // Capture both stdout and stderr to buffers, run the code and save to a string.
    let buf_stdout = gag::BufferRedirect::stdout();
    let buf_stderr = gag::BufferRedirect::stderr();
    let result = func().await;

    if let Ok(mut buf_stdout) = buf_stdout {
        buf_stdout.read_to_string(&mut output).unwrap();
        drop(buf_stdout);
    }

    if let Ok(mut buf_stderr) = buf_stderr {
        buf_stderr.read_to_string(&mut output).unwrap();
        drop(buf_stderr);
    }

    if cfg!(windows) {
        // In windows output error and warning path files start with \\?\
        // We replace \ by / so tests can check unix paths only
        let regex = Regex::new(r"\\\\?\\(.*)").unwrap();
        output = regex
            .replace_all(output.as_str(), |caps: &Captures| {
                caps[1].replace('\\', "/")
            })
            .to_string();
    }

    (result, output)
}

pub(crate) async fn deploy_contract(file_name: &str, run_config: &RunConfig) -> Result<ContractId> {
    // build the contract
    // deploy it
    println!(" Deploying {} ...", file_name.bold());
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let deployed_packages = deploy(DeployCommand {
        pkg: forc_client::cmd::deploy::Pkg {
            path: Some(format!(
                "{manifest_dir}/src/e2e_vm_tests/test_programs/{file_name}"
            )),
            terse: !run_config.verbose,
            locked: run_config.locked,
            ..Default::default()
        },
        signing_key: Some(SecretKey::from_str(SECRET_KEY).unwrap()),
        default_salt: true,
        build_profile: match run_config.release {
            true => BuildProfile::RELEASE.to_string(),
            false => BuildProfile::DEBUG.to_string(),
        },
        experimental: run_config.experimental.clone(),
        ..Default::default()
    })
    .await?;

    deployed_packages
        .into_iter()
        .map(|deployed_pkg| {
            if let DeployedPackage::Contract(deployed_contract) = deployed_pkg {
                Some(deployed_contract.id)
            } else {
                None
            }
        })
        .next()
        .flatten()
        .ok_or_else(|| anyhow!("expected to find at least one deployed contract."))
}

/// Run a given project against a node. Assumes the node is running at localhost:4000.
pub(crate) async fn runs_on_node(
    file_name: &str,
    run_config: &RunConfig,
    contract_ids: &[fuel_tx::ContractId],
) -> (Result<Vec<fuel_tx::Receipt>>, String) {
    run_and_capture_output(|| async {
        println!(" Running on node {} ...", file_name.bold());
        let manifest_dir = env!("CARGO_MANIFEST_DIR");

        let mut contracts = Vec::<String>::with_capacity(contract_ids.len());
        for contract_id in contract_ids {
            let contract = format!("0x{contract_id:x}");
            contracts.push(contract);
        }

        let command = RunCommand {
            pkg: forc_client::cmd::run::Pkg {
                path: Some(format!(
                    "{manifest_dir}/src/e2e_vm_tests/test_programs/{file_name}"
                )),
                locked: run_config.locked,
                terse: !run_config.verbose,
                ..Default::default()
            },
            node: NodeTarget {
                node_url: Some(NODE_URL.into()),
                ..Default::default()
            },
            contract: Some(contracts),
            signing_key: Some(SecretKey::from_str(SECRET_KEY).unwrap()),
            experimental: run_config.experimental.clone(),
            ..Default::default()
        };
        run(command).await.map(|ran_scripts| {
            ran_scripts
                .into_iter()
                .flat_map(|ran_script| ran_script.receipts)
                .collect::<Vec<_>>()
        })
    })
    .await
}

pub(crate) enum VMExecutionResult {
    Fuel(ProgramState, Vec<Receipt>, Box<EcalSyscallHandler>),
    Evm(revm::primitives::result::ExecutionResult),
}

/// Very basic check that code does indeed run in the VM.
pub(crate) fn runs_in_vm(
    script: BuiltPackage,
    script_data: Option<Vec<u8>>,
    witness_data: Option<Vec<Vec<u8>>>,
) -> Result<VMExecutionResult> {
    match script.descriptor.target {
        BuildTarget::Fuel => {
            let storage = MemoryStorage::default();

            let rng = &mut StdRng::seed_from_u64(2322u64);
            let maturity = 1.into();
            let script_data = script_data.unwrap_or_default();
            let block_height = (u32::MAX >> 1).into();
            // The default max length is 1MB which isn't enough for the bigger tests.
            let max_size = 64 * 1024 * 1024;
            let script_params = ScriptParameters::DEFAULT
                .with_max_script_length(max_size)
                .with_max_script_data_length(max_size);
            let tx_params = TxParameters::DEFAULT.with_max_size(max_size);
            let params = ConsensusParameters::V1(ConsensusParametersV1 {
                script_params,
                tx_params,
                ..Default::default()
            });
            let mut tb = TransactionBuilder::script(script.bytecode.bytes, script_data);

            tb.with_params(params)
                .add_unsigned_coin_input(
                    SecretKey::random(rng),
                    rng.r#gen(),
                    1,
                    Default::default(),
                    rng.r#gen(),
                )
                .maturity(maturity);

            if let Some(witnesses) = witness_data {
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
            let max_gas =
                tmp_tx.max_gas(consensus_params.gas_costs(), consensus_params.fee_params()) + 1;
            // Increase `script_gas_limit` to the maximum allowed value.
            tb.script_gas_limit(consensus_params.tx_params().max_gas_per_tx() - max_gas);

            let tx = tb
                .finalize_checked(block_height)
                .into_ready(gas_price, params.gas_costs(), params.fee_params(), None)
                .map_err(|e| anyhow::anyhow!("{e:?}"))?;

            let mem_instance = MemoryInstance::new();
            let mut i: Interpreter<_, _, _, EcalSyscallHandler> =
                Interpreter::with_storage(mem_instance, storage, Default::default());
            let transition = i.transact(tx).map_err(anyhow::Error::msg)?;

            Ok(VMExecutionResult::Fuel(
                *transition.state(),
                transition.receipts().to_vec(),
                Box::new(i.ecal_state().clone()),
            ))
        }
        BuildTarget::EVM => {
            let mut evm = revm::EvmBuilder::default()
                .with_db(revm::InMemoryDB::default())
                .with_clear_env()
                .build();

            // Transaction to create the smart contract
            let result = evm
                .transact_commit()
                .map_err(|e| anyhow::anyhow!("Could not create smart contract on EVM: {e:?}"))?;

            match result {
                revm::primitives::ExecutionResult::Revert { .. }
                | revm::primitives::ExecutionResult::Halt { .. } => todo!(),
                revm::primitives::ExecutionResult::Success { ref output, .. } => match output {
                    revm::primitives::result::Output::Call(_) => todo!(),
                    revm::primitives::result::Output::Create(_bytes, address_opt) => {
                        match address_opt {
                            None => todo!(),
                            Some(address) => {
                                evm.tx_mut().data = script.bytecode.bytes.into();
                                evm.tx_mut().transact_to =
                                    revm::interpreter::primitives::TransactTo::Call(*address);

                                let result = evm
                                    .transact_commit()
                                    .map_err(|e| anyhow::anyhow!("Failed call on EVM: {e:?}"))?;

                                Ok(VMExecutionResult::Evm(result))
                            }
                        }
                    }
                },
            }
        }
    }
}

/// Compiles the code and optionally captures the output of forc and the compilation.
/// Returns a tuple with the result of the compilation, as well as the output.
pub(crate) async fn compile_to_bytes(
    file_name: &str,
    run_config: &RunConfig,
    logs: &Option<String>,
) -> Result<Built> {
    println!("Compiling {} ...", file_name.bold());

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let root = format!("{manifest_dir}/src/e2e_vm_tests/test_programs/{file_name}");
    let build_opts = forc_pkg::BuildOpts {
        build_target: run_config.build_target,
        build_profile: BuildProfile::DEBUG.into(),
        release: run_config.release,
        print: PrintOpts {
            ast: false,
            dca_graph: None,
            dca_graph_url_format: None,
            asm: run_config.print_asm,
            bytecode: run_config.print_bytecode,
            bytecode_spans: run_config.print_bytecode,
            ir: run_config.print_ir.clone(),
            reverse_order: false,
        },
        pkg: forc_pkg::PkgOpts {
            path: Some(root.clone()),
            locked: run_config.locked,
            terse: false,
            ..Default::default()
        },
        experimental: run_config.experimental.experimental.clone(),
        no_experimental: run_config.experimental.no_experimental.clone(),
        ..Default::default()
    };

    match std::panic::catch_unwind(|| {
        if let Some(script) = logs {
            let handler = Arc::new(HarnessCallbackHandler::new(&script));
            let r = forc_pkg::build_with_options(&build_opts, Some(handler.clone()));
            handler.generate_snapshot(&root);
            r
        } else {
            forc_pkg::build_with_options(&build_opts, None)
        }
    }) {
        Ok(result) => {
            // Print the result of the compilation (i.e., any errors Forc produces).
            if let Err(ref e) = result {
                println!("\n{e}");
            }
            result
        }
        Err(_) => Err(anyhow!("Compiler panic")),
    }
}

/// Compiles the project's unit tests, then runs all unit tests.
/// Returns the tested package result.
pub(crate) async fn compile_and_run_unit_tests(
    file_name: &str,
    run_config: &RunConfig,
    capture_output: bool,
) -> (Result<Vec<forc_test::TestedPackage>>, String) {
    run_and_capture_output(|| async {
        tracing::info!("Compiling {} ...", file_name.bold());
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let path: PathBuf = [
            manifest_dir,
            "src",
            "e2e_vm_tests",
            "test_programs",
            file_name,
        ]
        .iter()
        .collect();

        match std::panic::catch_unwind(|| {
            forc_test::build(forc_test::TestOpts {
                pkg: forc_pkg::PkgOpts {
                    path: Some(path.to_string_lossy().into_owned()),
                    locked: run_config.locked,
                    terse: !(capture_output || run_config.verbose),
                    ..Default::default()
                },
                experimental: run_config.experimental.experimental.clone(),
                no_experimental: run_config.experimental.no_experimental.clone(),
                release: run_config.release,
                print: PrintOpts {
                    asm: run_config.print_asm,
                    bytecode: run_config.print_bytecode,
                    ir: run_config.print_ir.clone(),
                    ..Default::default()
                },
                build_target: run_config.build_target,
                ..Default::default()
            })
        }) {
            Ok(Ok(built_tests)) => {
                let test_filter = None;
                let tested = built_tests.run(forc_test::TestRunnerCount::Auto, test_filter)?;
                match tested {
                    forc_test::Tested::Package(tested_pkg) => Ok(vec![*tested_pkg]),
                    forc_test::Tested::Workspace(tested_pkgs) => Ok(tested_pkgs),
                }
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(anyhow!("Compiler panic")),
        }
    })
    .await
}

pub(crate) fn test_json_abi(
    file_name: &str,
    built_package: &BuiltPackage,
    experimental_new_encoding: bool,
    update_output_files: bool,
    suffix: &Option<String>,
    has_experimental_field: bool,
) -> Result<()> {
    emit_json_abi(file_name, built_package)?;
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let oracle_path = match (has_experimental_field, experimental_new_encoding) {
        (true, _) => {
            format!(
                "{}/src/e2e_vm_tests/test_programs/{}/json_abi_oracle.{}json",
                manifest_dir,
                file_name,
                suffix
                    .as_ref()
                    .unwrap()
                    .strip_prefix("test")
                    .unwrap()
                    .strip_suffix("toml")
                    .unwrap()
                    .trim_start_matches('.')
            )
        }
        (false, true) => {
            format!(
                "{}/src/e2e_vm_tests/test_programs/{}/{}",
                manifest_dir, file_name, "json_abi_oracle_new_encoding.json"
            )
        }
        (false, false) => {
            format!(
                "{}/src/e2e_vm_tests/test_programs/{}/{}",
                manifest_dir, file_name, "json_abi_oracle.json"
            )
        }
    };

    let output_path = format!(
        "{}/src/e2e_vm_tests/test_programs/{}/{}",
        manifest_dir, file_name, "json_abi_output.json"
    );

    // Update the oracle failing silently
    if update_output_files {
        let _ = std::fs::copy(&output_path, &oracle_path);
    }

    if fs::metadata(oracle_path.clone()).is_err() {
        bail!(
            "JSON ABI oracle file does not exist for this test\nExpected oracle path: {}",
            &oracle_path
        );
    }
    if fs::metadata(output_path.clone()).is_err() {
        bail!(
            "JSON ABI output file does not exist for this test\nExpected output path: {}",
            &output_path
        );
    }
    let oracle_contents = fs::read_to_string(&oracle_path)
        .expect("Something went wrong reading the JSON ABI oracle file.");
    let output_contents = fs::read_to_string(&output_path)
        .expect("Something went wrong reading the JSON ABI output file.");
    if oracle_contents != output_contents {
        bail!(
            "Mismatched ABI JSON output.\nOracle path: {}\nOutput path: {}\n{}",
            oracle_path,
            output_path,
            prettydiff::diff_lines(&oracle_contents, &output_contents)
        );
    }
    Ok(())
}

fn emit_json_abi(file_name: &str, built_package: &BuiltPackage) -> Result<()> {
    tracing::info!("ABI gen {} ...", file_name.bold());
    let json_abi = match &built_package.program_abi {
        ProgramABI::Fuel(abi) => serde_json::json!(abi),
        ProgramABI::Evm(abi) => serde_json::json!(abi),
        ProgramABI::MidenVM(_) => todo!(),
    };
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let file = std::fs::File::create(format!(
        "{}/src/e2e_vm_tests/test_programs/{}/{}",
        manifest_dir, file_name, "json_abi_output.json"
    ))?;
    let res = serde_json::to_writer_pretty(&file, &json_abi);
    res?;
    Ok(())
}

pub(crate) fn test_json_storage_slots(
    file_name: &str,
    built_package: &BuiltPackage,
    suffix: &Option<String>,
) -> Result<()> {
    emit_json_storage_slots(file_name, built_package)?;
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let oracle_path = format!(
        "{}/src/e2e_vm_tests/test_programs/{}/json_storage_slots_oracle.{}json",
        manifest_dir,
        file_name,
        suffix
            .as_ref()
            .unwrap()
            .strip_prefix("test")
            .unwrap()
            .strip_suffix("toml")
            .unwrap()
            .trim_start_matches('.')
    );
    let output_path = format!(
        "{}/src/e2e_vm_tests/test_programs/{}/{}",
        manifest_dir, file_name, "json_storage_slots_output.json"
    );
    if fs::metadata(oracle_path.clone()).is_err() {
        bail!("JSON storage slots oracle file does not exist for this test.\nExpected oracle path: {}", &oracle_path);
    }
    if fs::metadata(output_path.clone()).is_err() {
        bail!("JSON storage slots output file does not exist for this test.\nExpected output path: {}", &output_path);
    }
    let oracle_contents = fs::read_to_string(oracle_path.clone())
        .expect("Something went wrong reading the JSON storage slots oracle file.");
    let output_contents = fs::read_to_string(output_path.clone())
        .expect("Something went wrong reading the JSON storage slots output file.");
    if oracle_contents != output_contents {
        bail!(
            "Mismatched storage slots JSON output.\nOracle path: {}\nOutput path: {}\n{}",
            oracle_path,
            output_path,
            prettydiff::diff_lines(&oracle_contents, &output_contents)
        );
    }
    Ok(())
}

fn emit_json_storage_slots(file_name: &str, built_package: &BuiltPackage) -> Result<()> {
    tracing::info!("Storage slots JSON gen {} ...", file_name.bold());
    let json_storage_slots = serde_json::json!(built_package.storage_slots);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let file = std::fs::File::create(format!(
        "{}/src/e2e_vm_tests/test_programs/{}/{}",
        manifest_dir, file_name, "json_storage_slots_output.json"
    ))?;
    let res = serde_json::to_writer_pretty(&file, &json_storage_slots);
    res?;
    Ok(())
}
