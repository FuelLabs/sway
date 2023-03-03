use anyhow::{anyhow, bail, Result};
use colored::Colorize;
use forc_client::{
    cmd::{Deploy as DeployCommand, Run as RunCommand},
    op::{deploy, run},
};
use forc_pkg::{Built, BuiltPackage};
use fuel_tx::TransactionBuilder;
use fuel_vm::checked_transaction::builder::TransactionBuilderExt;
use fuel_vm::fuel_tx;
use fuel_vm::interpreter::Interpreter;
use fuel_vm::prelude::*;
use futures::Future;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use regex::{Captures, Regex};
use std::{fs, io::Read, path::PathBuf, str::FromStr};
use sway_core::{asm_generation::ProgramABI, BuildTarget};

use super::RunConfig;

pub const NODE_URL: &str = "http://127.0.0.1:4000";
pub const SECRET_KEY: &str = "de97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c";

pub(crate) async fn run_and_capture_output<F, Fut, T>(func: F) -> (T, String)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    let mut output = String::new();

    // Capture both stdout and stderr to buffers, run the code and save to a string.
    let buf_stdout = Some(gag::BufferRedirect::stdout().unwrap());
    let buf_stderr = Some(gag::BufferRedirect::stderr().unwrap());

    let result = func().await;

    let mut buf_stdout = buf_stdout.unwrap();
    let mut buf_stderr = buf_stderr.unwrap();
    buf_stdout.read_to_string(&mut output).unwrap();
    buf_stderr.read_to_string(&mut output).unwrap();
    drop(buf_stdout);
    drop(buf_stderr);

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

    deploy(DeployCommand {
        pkg: forc_client::cmd::deploy::Pkg {
            path: Some(format!(
                "{manifest_dir}/src/e2e_vm_tests/test_programs/{file_name}"
            )),
            terse: !run_config.verbose,
            locked: run_config.locked,
            ..Default::default()
        },
        signing_key: Some(SecretKey::from_str(SECRET_KEY).unwrap()),
        ..Default::default()
    })
    .await
    .map(|contract_ids| {
        contract_ids
            .first()
            .map(|contract_id| contract_id.id)
            .unwrap()
    })
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
            node_url: Some(NODE_URL.into()),
            contract: Some(contracts),
            signing_key: Some(SecretKey::from_str(SECRET_KEY).unwrap()),
            ..Default::default()
        };
        run(command).await.map(|ran_scripts| {
            ran_scripts
                .into_iter()
                .next()
                .map(|ran_script| ran_script.receipts)
                .unwrap()
        })
    })
    .await
}

pub(crate) enum VMExecutionResult {
    Fuel(ProgramState, Vec<Receipt>),
    Evm(revm::ExecutionResult),
    MidenVM(miden::ExecutionTrace),
}

/// Very basic check that code does indeed run in the VM.
pub(crate) fn runs_in_vm(
    script: BuiltPackage,
    script_data: Option<Vec<u8>>,
) -> Result<VMExecutionResult> {
    match script.build_target {
        BuildTarget::Fuel => {
            let storage = MemoryStorage::default();

            let rng = &mut StdRng::seed_from_u64(2322u64);
            let maturity = 1;
            let script_data = script_data.unwrap_or_default();
            let block_height = (u32::MAX >> 1) as u64;
            let params = &ConsensusParameters {
                // The default max length is 1MB which isn't enough for the bigger tests.
                max_script_length: 64 * 1024 * 1024,
                ..ConsensusParameters::DEFAULT
            };

            let tx = TransactionBuilder::script(script.bytecode.bytes, script_data)
                .add_unsigned_coin_input(rng.gen(), rng.gen(), 1, Default::default(), rng.gen(), 0)
                .gas_limit(fuel_tx::ConsensusParameters::DEFAULT.max_gas_per_tx)
                .maturity(maturity)
                .finalize_checked(block_height as Word, params, &GasCosts::default());

            let mut i = Interpreter::with_storage(storage, Default::default(), GasCosts::default());
            let transition = i.transact(tx)?;
            Ok(VMExecutionResult::Fuel(
                *transition.state(),
                transition.receipts().to_vec(),
            ))
        }
        BuildTarget::EVM => {
            let mut evm = revm::new();
            evm.database(revm::InMemoryDB::default());
            evm.env = revm::Env::default();

            // Transaction to create the smart contract
            evm.env.tx.transact_to = revm::TransactTo::create();
            evm.env.tx.data = bytes::Bytes::from(script.bytecode.bytes.into_boxed_slice());
            let result = evm.transact_commit();

            match result.out {
                revm::TransactOut::None => Err(anyhow!("Could not create smart contract")),
                revm::TransactOut::Call(_) => todo!(),
                revm::TransactOut::Create(ref _bytes, account_opt) => {
                    match account_opt {
                        Some(account) => {
                            evm.env.tx.transact_to = revm::TransactTo::Call(account);

                            // Now issue a call.
                            //evm.env.tx. = bytes::Bytes::from(script.bytecode.into_boxed_slice());
                            let result = evm.transact_commit();
                            Ok(VMExecutionResult::Evm(result))
                        }
                        None => todo!(),
                    }
                }
            }
        }
        BuildTarget::MidenVM => {
            use miden::{Assembler, ProgramInputs};

            // instantiate the assembler
            let assembler = Assembler::default();

            let bytecode_str = std::str::from_utf8(&script.bytecode.bytes)?;

            // compile Miden assembly source code into a program
            let program = assembler.compile(bytecode_str).unwrap();

            // execute the program with no inputs
            let _trace = miden::execute(&program, &ProgramInputs::none()).unwrap();

            let execution_trace = miden::execute(&program, &ProgramInputs::none())
                .map_err(|e| anyhow::anyhow!("Failed to execute on MidenVM: {e:?}"))?;

            Ok(VMExecutionResult::MidenVM(execution_trace))
        }
    }
}

/// Compiles the code and optionally captures the output of forc and the compilation.
/// Returns a tuple with the result of the compilation, as well as the output.
pub(crate) async fn compile_to_bytes(file_name: &str, run_config: &RunConfig) -> Result<Built> {
    println!("Compiling {} ...", file_name.bold());
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let build_opts = forc_pkg::BuildOpts {
        build_target: run_config.build_target,
        pkg: forc_pkg::PkgOpts {
            path: Some(format!(
                "{manifest_dir}/src/e2e_vm_tests/test_programs/{file_name}",
            )),
            locked: run_config.locked,
            terse: false,
            json_abi_with_callpaths: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let result = forc_pkg::build_with_options(build_opts);

    // Print the result of the compilation (i.e., any errors Forc produces).
    if let Err(ref e) = result {
        println!("\n{e}");
    }

    result
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
        let built_tests = forc_test::build(forc_test::Opts {
            pkg: forc_pkg::PkgOpts {
                path: Some(path.to_string_lossy().into_owned()),
                locked: run_config.locked,
                terse: !(capture_output || run_config.verbose),
                ..Default::default()
            },
            ..Default::default()
        })?;
        let tested = built_tests.run()?;

        match tested {
            forc_test::Tested::Package(tested_pkg) => Ok(vec![*tested_pkg]),
            forc_test::Tested::Workspace(tested_pkgs) => Ok(tested_pkgs),
        }
    })
    .await
}

pub(crate) fn test_json_abi(file_name: &str, built_package: &BuiltPackage) -> Result<()> {
    emit_json_abi(file_name, built_package)?;
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let oracle_path = format!(
        "{}/src/e2e_vm_tests/test_programs/{}/{}",
        manifest_dir, file_name, "json_abi_oracle.json"
    );
    let output_path = format!(
        "{}/src/e2e_vm_tests/test_programs/{}/{}",
        manifest_dir, file_name, "json_abi_output.json"
    );
    if fs::metadata(oracle_path.clone()).is_err() {
        bail!("JSON ABI oracle file does not exist for this test.");
    }
    if fs::metadata(output_path.clone()).is_err() {
        bail!("JSON ABI output file does not exist for this test.");
    }
    let oracle_contents =
        fs::read_to_string(oracle_path).expect("Something went wrong reading the file.");
    let output_contents =
        fs::read_to_string(output_path).expect("Something went wrong reading the file.");
    if oracle_contents != output_contents {
        bail!("Mismatched ABI JSON output.");
    }
    Ok(())
}

fn emit_json_abi(file_name: &str, built_package: &BuiltPackage) -> Result<()> {
    tracing::info!("ABI gen {} ...", file_name.bold());
    let json_abi = match &built_package.json_abi_program {
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

pub(crate) fn test_json_storage_slots(file_name: &str, built_package: &BuiltPackage) -> Result<()> {
    emit_json_storage_slots(file_name, built_package)?;
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let oracle_path = format!(
        "{}/src/e2e_vm_tests/test_programs/{}/{}",
        manifest_dir, file_name, "json_storage_slots_oracle.json"
    );
    let output_path = format!(
        "{}/src/e2e_vm_tests/test_programs/{}/{}",
        manifest_dir, file_name, "json_storage_slots_output.json"
    );
    if fs::metadata(oracle_path.clone()).is_err() {
        bail!("JSON storage slots oracle file does not exist for this test.");
    }
    if fs::metadata(output_path.clone()).is_err() {
        bail!("JSON storage slots output file does not exist for this test.");
    }
    let oracle_contents =
        fs::read_to_string(oracle_path).expect("Something went wrong reading the file.");
    let output_contents =
        fs::read_to_string(output_path).expect("Something went wrong reading the file.");
    if oracle_contents != output_contents {
        bail!("Mismatched storage slots JSON output.");
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
