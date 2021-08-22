use forc;
use forc::cli::BuildCommand;
use fuel_tx::{ContractId, Input, Output, Transaction};
use fuel_vm::interpreter::Interpreter;
use fuel_vm::prelude::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Very basic check that code does indeed run in the VM.
/// `true` if it does, `false` if not.
pub(crate) fn runs_in_vm(file_name: &str) {
    let mut storage = MemoryStorage::default();
    let program = vec![Opcode::NOOP, Opcode::RET(16)];

    let program: Witness = program.into_iter().collect::<Vec<u8>>().into();

    let contract = Contract::from(program.as_ref());
    let rng = &mut StdRng::seed_from_u64(2322u64);

    let salt: Salt = rng.gen();

    let contract_root = contract.root();
    let contract_id = contract.id(&salt, &contract_root);

    let output = Output::contract_created(contract_id);

    let gas_price = 10;
    let gas_limit = 10000;
    let maturity = 0;
    let bytecode_witness = 0;
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

    // Deploy the contract
    Interpreter::transition(&mut storage, tx).expect("Failed to transact");
    let mut interpreter = Interpreter::with_storage(storage);

    // construct the test case transaction
    // this is the ID that the above transition deployed to
    // in hex, this is 0x781168189b0865cce557e5f53af9357238b55dbed0dccfc016a46505296a41a1
    let contract_id = ContractId::from([
        120, 17, 104, 24, 155, 8, 101, 204, 229, 87, 229, 245, 58, 249, 53, 114, 56, 181, 93, 190,
        208, 220, 207, 192, 22, 164, 101, 5, 41, 106, 65, 161,
    ]);
    // mock input
    let input_contract = Input::Contract {
        utxo_id: Default::default(),
        balance_root: Default::default(),
        state_root: Default::default(),
        contract_id: contract_id.clone(),
    };

    // mock output
    let output_contract = Output::Contract {
        input_index: 0,
        balance_root: Default::default(),
        state_root: Default::default(),
    };

    // grab the test case and compile it to bytecode
    let script = compile_to_bytes(file_name);
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

    // evaluate the test case
    interpreter.transact(tx_to_test).unwrap();
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
