use forc;

use fuel_tx::Transaction;
use fuel_vm_rust::interpreter::Interpreter;

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
    let block_height = (u32::MAX >> 1) as u64;
    tx.validate(block_height).unwrap();
    Interpreter::execute_tx(tx).unwrap();
}

/// Returns `true` if a file compiled without any errors or warnings,
/// and `false` if it did not.
pub(crate) fn compile_to_bytes(file_name: &str) -> Vec<u8> {
    println!("Compiling {}", file_name);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    forc::ops::forc_build::build(Some(format!(
        "{}/src/e2e_vm_tests/test_programs/{}",
        manifest_dir, file_name
    )))
    .unwrap()
}
