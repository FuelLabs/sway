use anyhow::{bail, Result};
use forc::test::{
    forc_abi_json, forc_build, forc_deploy, forc_run, BuildCommand, DeployCommand, JsonAbiCommand,
    RunCommand,
};
use fuel_tx::Transaction;
use fuel_vm::interpreter::Interpreter;
use fuel_vm::prelude::*;
use serde_json::Value;
use std::fs;

pub(crate) fn deploy_contract(file_name: &str) -> ContractId {
    // build the contract
    // deploy it
    println!(" Deploying {}", file_name);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let (verbose, use_ir) = get_test_config_from_env();

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(forc_deploy::deploy(DeployCommand {
            path: Some(format!(
                "{}/src/e2e_vm_tests/test_programs/{}",
                manifest_dir, file_name
            )),
            use_ir,
            silent_mode: !verbose,
            ..Default::default()
        }))
        .unwrap()
}

/// Run a given project against a node. Assumes the node is running at localhost:4000.
pub(crate) fn runs_on_node(file_name: &str, contract_ids: &[fuel_tx::ContractId]) {
    println!("Running on node: {}", file_name);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let mut contracts = Vec::<String>::with_capacity(contract_ids.len());
    for contract_id in contract_ids {
        let contract = format!("0x{:x}", contract_id);
        contracts.push(contract);
    }

    let (verbose, use_ir) = get_test_config_from_env();

    let command = RunCommand {
        path: Some(format!(
            "{}/src/e2e_vm_tests/test_programs/{}",
            manifest_dir, file_name
        )),
        node_url: "127.0.0.1:4000".into(),
        use_ir,
        silent_mode: !verbose,
        contract: Some(contracts),
        ..Default::default()
    };
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(forc_run::run(command))
        .unwrap()
}

/// Very basic check that code does indeed run in the VM.
/// `true` if it does, `false` if not.
pub(crate) fn runs_in_vm(file_name: &str) -> ProgramState {
    let storage = MemoryStorage::default();

    let script = compile_to_bytes(file_name).unwrap();
    let gas_price = 10;
    let gas_limit = fuel_tx::consts::MAX_GAS_PER_TX;
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
        script,
        script_data,
        inputs,
        outputs,
        witness,
    );
    let block_height = (u32::MAX >> 1) as u64;
    tx_to_test.validate(block_height).unwrap();
    let mut i = Interpreter::with_storage(storage);
    *i.transact(tx_to_test).unwrap().state()
}

/// Panics if code _does_ compile, used for test cases where the source
/// code should have been rejected by the compiler.
pub(crate) fn does_not_compile(file_name: &str) {
    assert!(
        compile_to_bytes(file_name).is_err(),
        "{} should not have compiled.",
        file_name,
    )
}

/// Returns `true` if a file compiled without any errors or warnings,
/// and `false` if it did not.
pub(crate) fn compile_to_bytes(file_name: &str) -> Result<Vec<u8>> {
    println!(" Compiling {}", file_name);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let (verbose, use_ir) = get_test_config_from_env();
    forc_build::build(BuildCommand {
        path: Some(format!(
            "{}/src/e2e_vm_tests/test_programs/{}",
            manifest_dir, file_name
        )),
        use_ir,
        silent_mode: !verbose,
        ..Default::default()
    })
    .map(|compiled| compiled.bytecode)
}

pub(crate) fn test_json_abi(file_name: &str) -> Result<()> {
    let _script = compile_to_json_abi(file_name)?;
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

fn compile_to_json_abi(file_name: &str) -> Result<Value> {
    println!("   ABI gen {}", file_name);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    forc_abi_json::build(JsonAbiCommand {
        path: Some(format!(
            "{}/src/e2e_vm_tests/test_programs/{}",
            manifest_dir, file_name
        )),
        json_outfile: Some(format!(
            "{}/src/e2e_vm_tests/test_programs/{}/{}",
            manifest_dir, file_name, "json_abi_output.json"
        )),
        silent_mode: true,
        ..Default::default()
    })
}

fn get_test_config_from_env() -> (bool, bool) {
    let var_exists = |key| std::env::var(key).map(|_| true).unwrap_or(false);

    (
        var_exists("SWAY_TEST_VERBOSE"),
        var_exists("SWAY_TEST_USE_IR"),
    )
}
