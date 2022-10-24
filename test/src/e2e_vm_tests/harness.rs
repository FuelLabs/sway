use anyhow::{bail, Result};
use forc::test::{forc_build, BuildCommand};
use forc_client::ops::{
    deploy::{cmd::DeployCommand, op::deploy},
    run::{cmd::RunCommand, op::run},
};
use forc_pkg::Compiled;
use fuel_tx::TransactionBuilder;
use fuel_vm::interpreter::Interpreter;
use fuel_vm::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::{fmt::Write, fs, io::Read, str::FromStr};

use super::RunConfig;

pub const NODE_URL: &str = "http://127.0.0.1:4000";
pub const SECRET_KEY: &str = "de97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c";

pub(crate) async fn deploy_contract(file_name: &str, run_config: &RunConfig) -> Result<ContractId> {
    // build the contract
    // deploy it
    tracing::info!(" Deploying {}", file_name);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    deploy(DeployCommand {
        path: Some(format!(
            "{}/src/e2e_vm_tests/test_programs/{}",
            manifest_dir, file_name
        )),
        terse_mode: !run_config.verbose,
        locked: run_config.locked,
        signing_key: Some(SecretKey::from_str(SECRET_KEY).unwrap()),
        ..Default::default()
    })
    .await
}

/// Run a given project against a node. Assumes the node is running at localhost:4000.
pub(crate) async fn runs_on_node(
    file_name: &str,
    run_config: &RunConfig,
    contract_ids: &[fuel_tx::ContractId],
) -> Result<Vec<fuel_tx::Receipt>> {
    tracing::info!("Running on node: {}", file_name);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let mut contracts = Vec::<String>::with_capacity(contract_ids.len());
    for contract_id in contract_ids {
        let contract = format!("0x{:x}", contract_id);
        contracts.push(contract);
    }

    let command = RunCommand {
        path: Some(format!(
            "{}/src/e2e_vm_tests/test_programs/{}",
            manifest_dir, file_name
        )),
        node_url: Some(NODE_URL.into()),
        terse_mode: !run_config.verbose,
        contract: Some(contracts),
        locked: run_config.locked,
        signing_key: Some(SecretKey::from_str(SECRET_KEY).unwrap()),
        ..Default::default()
    };
    run(command).await
}

/// Very basic check that code does indeed run in the VM.
/// `true` if it does, `false` if not.
pub(crate) fn runs_in_vm(
    script: Compiled,
    script_data: Option<Vec<u8>>,
) -> (ProgramState, Compiled) {
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

    let tx = TransactionBuilder::script(script.bytecode.clone(), script_data)
        .add_unsigned_coin_input(rng.gen(), rng.gen(), 1, Default::default(), rng.gen(), 0)
        .gas_limit(fuel_tx::ConsensusParameters::DEFAULT.max_gas_per_tx)
        .maturity(maturity)
        .finalize_checked(block_height as Word, params);

    let mut i = Interpreter::with_storage(storage, Default::default());
    let transition = i.transact(tx).unwrap();
    (*transition.state(), script)
}

/// Compiles the code and optionally captures the output of forc and the compilation.
/// Returns a tuple with the result of the compilation, as well as the output.
pub(crate) fn compile_to_bytes(
    file_name: &str,
    run_config: &RunConfig,
    capture_output: bool,
) -> (Result<Compiled>, String) {
    tracing::info!(" Compiling {}", file_name);

    let mut buf_stdout: Option<gag::BufferRedirect> = None;
    let mut buf_stderr: Option<gag::BufferRedirect> = None;
    if capture_output {
        // Capture both stdout and stderr to buffers, compile the test and save to a string.
        buf_stdout = Some(gag::BufferRedirect::stdout().unwrap());
        buf_stderr = Some(gag::BufferRedirect::stderr().unwrap());
    }

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let result = forc_build::build(BuildCommand {
        path: Some(format!(
            "{}/src/e2e_vm_tests/test_programs/{}",
            manifest_dir, file_name
        )),
        locked: run_config.locked,
        terse_mode: !(capture_output || run_config.verbose),
        ..Default::default()
    });

    let mut output = String::new();
    if capture_output {
        let mut buf_stdout = buf_stdout.unwrap();
        let mut buf_stderr = buf_stderr.unwrap();
        buf_stdout.read_to_string(&mut output).unwrap();
        buf_stderr.read_to_string(&mut output).unwrap();
        drop(buf_stdout);
        drop(buf_stderr);

        // If verbosity is requested then print it out.
        if run_config.verbose {
            tracing::info!("{output}");
        }

        // Capture the result of the compilation (i.e., any errors Forc produces) and append to
        // the stdout from the compiler.
        if let Err(ref e) = result {
            write!(output, "\n{}", e).expect("error writing output");
        }
    }

    (result, output)
}

pub(crate) fn test_json_abi(file_name: &str, compiled: &Compiled) -> Result<()> {
    emit_json_abi(file_name, compiled)?;
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

fn emit_json_abi(file_name: &str, compiled: &Compiled) -> Result<()> {
    tracing::info!("   ABI gen {}", file_name);
    let json_abi = serde_json::json!(compiled.json_abi_program);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let file = std::fs::File::create(format!(
        "{}/src/e2e_vm_tests/test_programs/{}/{}",
        manifest_dir, file_name, "json_abi_output.json"
    ))?;
    let res = serde_json::to_writer_pretty(&file, &json_abi);
    res?;
    Ok(())
}

pub(crate) fn test_json_storage_slots(file_name: &str, compiled: &Compiled) -> Result<()> {
    emit_json_storage_slots(file_name, compiled)?;
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

fn emit_json_storage_slots(file_name: &str, compiled: &Compiled) -> Result<()> {
    tracing::info!("   storage slots JSON gen {}", file_name);
    let json_storage_slots = serde_json::json!(compiled.storage_slots);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let file = std::fs::File::create(format!(
        "{}/src/e2e_vm_tests/test_programs/{}/{}",
        manifest_dir, file_name, "json_storage_slots_output.json"
    ))?;
    let res = serde_json::to_writer_pretty(&file, &json_storage_slots);
    res?;
    Ok(())
}
