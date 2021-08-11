use forc;
use forc::cli::BuildCommand;

use fuel_tx::Transaction;
use fuel_vm::interpreter::Interpreter;
use fuel_vm::prelude::MemoryStorage;
use tx_client::client::TxClient;

/// Very basic check that code does indeed run in the VM.
/// `true` if it does, `false` if not.
pub(crate) fn runs_in_vm(file_name: &str) {
    let script = compile_to_bytes(file_name);
    let gas_price = 10;
    let gas_limit = 10000;
    let maturity = 100;
    let script_data = vec![];
    let inputs = vec![];
    let outputs = vec![];
    let witness = vec![];
    let tx = Transaction::script(
        gas_price,
        gas_limit,
        maturity,
        script,
        script_data,
        inputs,
        outputs,
        witness,
    );
    let node_url = "127.0.0.1:4000";

    let client = TxClient::new(node_url).unwrap();
    let res = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async { client.transact(&tx).await });

    res.unwrap()
}

/// Returns `true` if a file compiled without any errors or warnings,
/// and `false` if it did not.
pub(crate) fn compile_to_bytes(file_name: &str) -> Vec<u8> {
    println!("Compiling {}", file_name);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    forc::ops::forc_build::build(BuildCommand {
        path: Some(format!(
            "{}/src/e2e_vm_tests/test_programs/{}",
            manifest_dir, file_name
        )),
        print_asm: false,
        binary_outfile: None,
        offline_mode: false,
    })
    .unwrap()
}
