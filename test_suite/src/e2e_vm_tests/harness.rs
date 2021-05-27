use forc;

use fuel_tx::Transaction;
use fuel_vm_rust::interpreter::Interpreter;

/// Very basic check that code does indeed run in the VM.
/// `true` if it does, `false` if not.
pub(crate) fn runs_in_vm(file_name: &str) -> bool {
    let bytes = compile_to_bytes(file_name);
    let transaction = Transaction::Script {
        gas_price: 0,
        gas_limit: u64::MAX,
        maturity: 0,
        script: bytes,
        script_data: vec![],
        inputs: vec![],
        outputs: vec![],
        witnesses: vec![],
    };

    Interpreter::execute_tx(transaction).is_ok()
}

/// Returns `true` if a file compiled without any errors or warnings,
/// and `false` if it did not.
pub(crate) fn compile_to_bytes(file_name: &str) -> Vec<u8> {
    println!("Compiling {}", file_name);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let res = forc::ops::forc_build::build(Some(format!(
        "{}/src/e2e_vm_tests/test_programs/{}",
        manifest_dir, file_name
    )));
    match res {
        Ok(bytes) => bytes,
        Err(_) => {
            panic!(
                "TEST FAILURE: Project \"{}\" failed to compile. ",
                file_name
            );
        }
    }
}
