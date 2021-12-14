use forc::test::{forc_build, forc_deploy, forc_run, BuildCommand, DeployCommand, RunCommand};
use fuel_tx::Transaction;
use fuel_vm::interpreter::Interpreter;
use fuel_vm::prelude::*;

pub(crate) fn deploy_contract(file_name: &str) {
    // build the contract
    // deploy it
    println!(" Deploying {}", file_name);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(forc_deploy::deploy(DeployCommand {
            path: Some(format!(
                "{}/src/e2e_vm_tests/test_programs/{}",
                manifest_dir, file_name
            )),
            print_finalized_asm: false,
            print_intermediate_asm: false,
            binary_outfile: None,
            offline_mode: false,
            silent_mode: true,
        }))
        .unwrap()
}

/// Run a given project against a node. Assumes the node is running at localhost:4000.
pub(crate) fn runs_on_node(file_name: &str) {
    println!("Running on node: {}", file_name);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let command = RunCommand {
        data: None,
        path: Some(format!(
            "{}/src/e2e_vm_tests/test_programs/{}",
            manifest_dir, file_name
        )),
        dry_run: false,
        node_url: "127.0.0.1:4000".into(),
        kill_node: false,
        binary_outfile: None,
        print_finalized_asm: false,
        print_intermediate_asm: false,
        silent_mode: true,
        pretty_print: false,
    };
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(forc_run::run(command))
        .unwrap()
}

/// Very basic check that code does indeed run in the VM.
/// `true` if it does, `false` if not.
pub(crate) fn runs_in_vm(file_name: &str) -> ProgramState {
    let mut storage = MemoryStorage::default();

    let script = compile_to_bytes(file_name).unwrap();
    let gas_price = 10;
    let gas_limit = 100000;
    let maturity = 0;
    let script_data = vec![];
    let inputs = vec![];
    let outputs = vec![];
    let witness = vec![];
    let tx_to_test = Transaction::script(
        gas_price,
        gas_limit,
        maturity,
        script,
        script_data,
        inputs,
        outputs,
        witness,
    );
    let block_height = (u32::MAX >> 1) as u64;
    tx_to_test.validate(block_height).unwrap();
    *Interpreter::transition(&mut storage, tx_to_test)
        .unwrap()
        .state()
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
pub(crate) fn compile_to_bytes(file_name: &str) -> Result<Vec<u8>, String> {
    println!(" Compiling {}", file_name);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    forc_build::build(BuildCommand {
        path: Some(format!(
            "{}/src/e2e_vm_tests/test_programs/{}",
            manifest_dir, file_name
        )),
        print_finalized_asm: false,
        print_intermediate_asm: false,
        binary_outfile: None,
        offline_mode: false,
        silent_mode: true,
    })
}
