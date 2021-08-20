use forc;
use forc::cli::BuildCommand;

use fuel_tx::{ContractId, Input, Output, Transaction};
use fuel_vm::interpreter::Interpreter;
use fuel_vm::prelude::{Contract, MemoryStorage, Storage};

/// Very basic check that code does indeed run in the VM.
/// `true` if it does, `false` if not.
pub(crate) fn runs_in_vm(file_name: &str) {
    let contract_id = ContractId::from([
        17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17,
        17, 17, 17, 17, 17, 17, 17, 17, 17,
    ]);
    let input_contract = Input::Contract {
        utxo_id: Default::default(),
        balance_root: Default::default(),
        state_root: Default::default(),
        contract_id: contract_id.clone(),
    };
    let output_contract = Output::Contract {
        input_index: 0,
        balance_root: Default::default(),
        state_root: Default::default(),
    };

    let script = compile_to_bytes(file_name);
    let gas_price = 10;
    let gas_limit = 10000;
    let maturity = 0;
    let script_data = vec![];
    let inputs = vec![input_contract];
    let outputs = vec![output_contract];
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
    let mut storage = MemoryStorage::default();
    Storage::<_, Contract>::insert(&mut storage, &contract_id, &Default::default());
    //    storage.insert::<_, Contract>(&contract_id, Default::default());
    let mut interpreter = Interpreter::with_storage(storage);
    interpreter.transact(tx).unwrap();
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
