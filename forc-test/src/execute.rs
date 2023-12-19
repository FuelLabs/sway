use forc_pkg as pkg;
use fuel_abi_types::error_codes::ErrorSignal;
use fuel_tx as tx;
use fuel_vm::checked_transaction::builder::TransactionBuilderExt;
use fuel_vm::{self as vm, fuel_asm, prelude::Instruction};
use pkg::{Built, BuiltPackage};
use pkg::{PkgTestEntry, TestPassCondition};
use rand::{Rng, SeedableRng};
use rayon::prelude::*;
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};
use sway_core::BuildTarget;
use sway_types::Span;
use tx::output::contract::Contract;
use tx::{Chargeable, Finalizable, TransactionBuilder};
use vm::interpreter::ExecutableTransaction;
use vm::prelude::SecretKey;
use vm::storage::MemoryStorage;
use crate::setup::TestSetup;
use crate::TestResult;
use crate::TEST_METADATA_SEED;

#[derive(Debug)]
pub struct TestExecutor {
    pub interpreter:
        vm::prelude::Interpreter<MemoryStorage, fuel_tx::Script, vm::interpreter::NotSupportedEcal>,
    tx_builder: TransactionBuilder<fuel_tx::Script>,
    test_entry: PkgTestEntry,
    name: String,
}

impl TestExecutor {
    pub fn new(
        bytecode: &[u8],
        test_offset: u32,
        test_setup: TestSetup,
        test_entry: &PkgTestEntry,
        name: String,
    ) -> Self {
        let storage = test_setup.storage().clone();

        // Patch the bytecode to jump to the relevant test.
        let bytecode = patch_test_bytecode(bytecode, test_offset).into_owned();

        // Create a transaction to execute the test function.
        let script_input_data = vec![];
        let rng = &mut rand::rngs::StdRng::seed_from_u64(TEST_METADATA_SEED);

        // Prepare the transaction metadata.
        let secret_key = SecretKey::random(rng);
        let utxo_id = rng.gen();
        let amount = 1;
        let maturity = 1.into();
        let asset_id = rng.gen();
        let tx_pointer = rng.gen();

        let mut tx_builder = tx::TransactionBuilder::script(bytecode, script_input_data)
            .add_unsigned_coin_input(
                secret_key,
                utxo_id,
                amount,
                asset_id,
                tx_pointer,
                0u32.into(),
            )
            .maturity(maturity)
            .clone();
        let mut output_index = 1;
        // Insert contract ids into tx input
        for contract_id in test_setup.contract_ids() {
            tx_builder.add_input(tx::Input::contract(
                tx::UtxoId::new(tx::Bytes32::zeroed(), 0),
                tx::Bytes32::zeroed(),
                tx::Bytes32::zeroed(),
                tx::TxPointer::new(0u32.into(), 0),
                contract_id,
            ))
            .add_output(tx::Output::Contract(Contract {
                input_index: output_index,
                balance_root: fuel_tx::Bytes32::zeroed(),
                state_root: tx::Bytes32::zeroed(),
            }));
            output_index += 1;
        }
        let consensus_params = tx_builder.get_params().clone();

        TestExecutor {
            interpreter: vm::interpreter::Interpreter::with_storage(
                storage,
                consensus_params.into(),
            ),
            tx_builder,
            test_entry: test_entry.clone(),
            name,
        }
    }

    pub fn execute(&mut self) -> anyhow::Result<TestResult> {
        // let offset = u32::try_from(entry.finalized.imm)
        // .expect("test instruction offset out of range");
        // let name = entry.finalized.fn_name.clone();
        // let test_setup = self.setup()?;

        let block_height = (u32::MAX >> 1).into();
        let start = std::time::Instant::now();
        let transition = self
            .interpreter
            .transact(self.tx_builder.finalize_checked(block_height)).map_err(|err| anyhow::anyhow!(err))?;
        let duration = start.elapsed();
        let state = *transition.state();
        let receipts = transition.receipts().to_vec();

        let gas_used = *receipts
            .iter()
            .find_map(|receipt| match receipt {
                tx::Receipt::ScriptResult { gas_used, .. } => Some(gas_used),
                _ => None,
            })
            .ok_or_else(|| anyhow::anyhow!("missing used gas information from test execution"))?;

        // Only retain `Log` and `LogData` receipts.
        let logs = receipts
            .into_iter()
            .filter(|receipt| {
                matches!(receipt, fuel_tx::Receipt::Log { .. })
                    || matches!(receipt, fuel_tx::Receipt::LogData { .. })
            })
            .collect();

        let span = self.test_entry.span.clone();
        let file_path = self.test_entry.file_path.clone();
        let condition = self.test_entry.pass_condition.clone();
        Ok(TestResult {
            name: self.name.clone(),
            file_path,
            duration,
            span,
            state,
            condition,
            logs,
            gas_used,
        })
    }
}

/// Given some bytecode and an instruction offset for some test's desired entry point, patch the
/// bytecode with a `JI` (jump) instruction to jump to the desired test.
///
/// We want to splice in the `JI` only after the initial data section setup is complete, and only
/// if the entry point doesn't begin exactly after the data section setup.
///
/// The following is how the beginning of the bytecode is laid out:
///
/// ```ignore
/// [0] ji   i4                       ; Jumps to the data section setup.
/// [1] noop
/// [2] DATA_SECTION_OFFSET[0..32]
/// [3] DATA_SECTION_OFFSET[32..64]
/// [4] lw   $ds $is 1                ; The data section setup, i.e. where the first ji lands.
/// [5] add  $$ds $$ds $is
/// [6] <first-entry-point>           ; This is where we want to jump from to our test code!
/// ```
fn patch_test_bytecode(bytecode: &[u8], test_offset: u32) -> std::borrow::Cow<[u8]> {
    // TODO: Standardize this or add metadata to bytecode.
    const PROGRAM_START_INST_OFFSET: u32 = 6;
    const PROGRAM_START_BYTE_OFFSET: usize = PROGRAM_START_INST_OFFSET as usize * Instruction::SIZE;

    // If our desired entry point is the program start, no need to jump.
    if test_offset == PROGRAM_START_INST_OFFSET {
        return std::borrow::Cow::Borrowed(bytecode);
    }

    // Create the jump instruction and splice it into the bytecode.
    let ji = fuel_asm::op::ji(test_offset);
    let ji_bytes = ji.to_bytes();
    let start = PROGRAM_START_BYTE_OFFSET;
    let end = start + ji_bytes.len();
    let mut patched = bytecode.to_vec();
    patched.splice(start..end, ji_bytes);
    std::borrow::Cow::Owned(patched)
}
