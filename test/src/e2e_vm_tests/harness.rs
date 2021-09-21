use forc::test::{forc_build, BuildCommand};
use fuel_tx::{Input, Output, Transaction};
use fuel_vm::interpreter::Interpreter;
use fuel_vm::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Very basic check that code does indeed run in the VM.
/// `true` if it does, `false` if not.
pub(crate) fn runs_in_vm(file_name: &str) -> ProgramState {
    let mut storage = MemoryStorage::default();
    let program = vec![Opcode::NOOP, Opcode::RET(1)];

    let program: Witness = program.into_iter().collect::<Vec<u8>>().into();

    let contract = Contract::from(program.as_ref());
    let rng = &mut StdRng::seed_from_u64(2322u64);

    let salt: Salt = rng.gen();

    let contract_root = contract.root();
    let contract_id = contract.id(&salt, &contract_root);

    let output = Output::contract_created(contract_id.clone());

    let bytecode_witness = 0;
    let gas_price = 10;
    let gas_limit = 10000;
    let maturity = 0;
    let tx = Transaction::create(
        gas_price,
        gas_limit,
        maturity,
        bytecode_witness,
        salt,
        vec![],
        vec![],
        vec![output],
        vec![program],
    );

    // Deploy the contract into the blockchain
    Interpreter::transition(&mut storage, tx).expect("Failed to transact");
    // evaluate the test case
    let input_contract = Input::Contract {
        utxo_id: rng.gen(),
        balance_root: rng.gen(),
        state_root: rng.gen(),
        contract_id: contract_id.clone(),
    };
    let output_contract = Output::Contract {
        input_index: 0,
        balance_root: rng.gen(),
        state_root: rng.gen(),
    };

    let script = compile_to_bytes(file_name).unwrap();
    let gas_price = 10;
    let gas_limit = 10000;
    let maturity = 0;
    let script_data = vec![];
    let inputs = vec![input_contract];
    let outputs = vec![output_contract];
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
    Interpreter::transition(&mut storage, tx_to_test)
        .unwrap()
        .state()
        .clone()
}

/// Panics if code _does_ compile, used for test cases where the source
/// code should have been rejected by the compiler.
pub(crate) fn does_not_compile(file_name: &str) {
    if let Ok(_) = compile_to_bytes(file_name) {
        panic!("{} should not have compiled.", file_name);
    }
}

/// Returns `true` if a file compiled without any errors or warnings,
/// and `false` if it did not.
pub(crate) fn compile_to_bytes(file_name: &str) -> Result<Vec<u8>, String> {
    println!("Compiling {}", file_name);
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
    })
}
