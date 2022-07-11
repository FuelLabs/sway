use anyhow::{bail, Result};
use forc::test::{forc_build, forc_deploy, forc_run, BuildCommand, DeployCommand, RunCommand};
use forc_pkg::Compiled;
use fuel_tx::Transaction;
use fuel_vm::interpreter::Interpreter;
use fuel_vm::prelude::*;
use std::{fmt::Write, fs};

pub(crate) fn deploy_contract(file_name: &str, locked: bool) -> ContractId {
    // build the contract
    // deploy it
    tracing::info!(" Deploying {}", file_name);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let verbose = get_test_config_from_env();

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(forc_deploy::deploy(DeployCommand {
            path: Some(format!(
                "{}/src/e2e_vm_tests/test_programs/{}",
                manifest_dir, file_name
            )),
            silent_mode: !verbose,
            locked,
            ..Default::default()
        }))
        .unwrap()
}

/// Run a given project against a node. Assumes the node is running at localhost:4000.
pub(crate) fn runs_on_node(
    file_name: &str,
    locked: bool,
    contract_ids: &[fuel_tx::ContractId],
) -> Vec<fuel_tx::Receipt> {
    tracing::info!("Running on node: {}", file_name);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let mut contracts = Vec::<String>::with_capacity(contract_ids.len());
    for contract_id in contract_ids {
        let contract = format!("0x{:x}", contract_id);
        contracts.push(contract);
    }

    let verbose = get_test_config_from_env();

    let command = RunCommand {
        path: Some(format!(
            "{}/src/e2e_vm_tests/test_programs/{}",
            manifest_dir, file_name
        )),
        node_url: Some("http://127.0.0.1:4000".into()),
        silent_mode: !verbose,
        contract: Some(contracts),
        locked,
        ..Default::default()
    };
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(forc_run::run(command))
        .unwrap()
}

/// Very basic check that code does indeed run in the VM.
/// `true` if it does, `false` if not.
pub(crate) fn runs_in_vm(file_name: &str, locked: bool) -> (ProgramState, Compiled) {
    let storage = MemoryStorage::default();

    let script = compile_to_bytes(file_name, locked).unwrap();
    let gas_price = 10;
    let gas_limit = fuel_tx::ConsensusParameters::DEFAULT.max_gas_per_tx;
    let byte_price = 0;
    let maturity = 0;
    let script_data = vec![];
    let inputs = vec![];
    let outputs = vec![];
    let witness = vec![];
    let tx_to_test = Transaction::script(
        gas_price,
        gas_limit,
        byte_price,
        maturity,
        script.bytecode.clone(),
        script_data,
        inputs,
        outputs,
        witness,
    );
    let block_height = (u32::MAX >> 1) as u64;
    tx_to_test
        .validate(block_height, &Default::default())
        .unwrap();
    let mut i = Interpreter::with_storage(storage, Default::default());
    (*i.transact(tx_to_test).unwrap().state(), script)
}

/// Returns Err(()) if code _does_ compile, used for test cases where the source
/// code should have been rejected by the compiler.  When it fails to compile the
/// captured stdout is returned.
pub(crate) fn does_not_compile(file_name: &str, locked: bool) -> Result<String, ()> {
    use std::io::Read;

    tracing::info!(" Compiling {}", file_name);

    // Capture stdout to a buffer, compile the test and save stdout to a string.
    let mut buf = gag::BufferRedirect::stdout().unwrap();
    let result = compile_to_bytes_verbose(file_name, locked, true);
    let mut output = String::new();
    buf.read_to_string(&mut output).unwrap();
    drop(buf);

    // If verbosity is requested then print it out.
    if get_test_config_from_env() {
        tracing::info!("{output}");
    }

    // Invert the result; if it succeeds then return an Err.
    match result {
        Ok(_) => Err(()),
        Err(e) => {
            // Capture the result of the compilation (i.e., any errors Forc produces) and append to
            // the stdout from the compiler.
            write!(output, "\n{}", e).map_err(|_| ())?;
            Ok(output)
        }
    }
}

/// Returns `true` if a file compiled without any errors or warnings,
/// and `false` if it did not.
pub(crate) fn compile_to_bytes(file_name: &str, locked: bool) -> Result<Compiled> {
    compile_to_bytes_verbose(file_name, locked, get_test_config_from_env())
}

pub(crate) fn compile_to_bytes_verbose(
    file_name: &str,
    locked: bool,
    verbose: bool,
) -> Result<Compiled> {
    tracing::info!(" Compiling {}", file_name);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    forc_build::build(BuildCommand {
        path: Some(format!(
            "{}/src/e2e_vm_tests/test_programs/{}",
            manifest_dir, file_name
        )),
        locked,
        silent_mode: !verbose,
        ..Default::default()
    })
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
    let json_abi = serde_json::json!(compiled.json_abi);
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

fn get_test_config_from_env() -> bool {
    let var_exists = |key| std::env::var(key).map(|_| true).unwrap_or(false);
    var_exists("SWAY_TEST_VERBOSE")
}
