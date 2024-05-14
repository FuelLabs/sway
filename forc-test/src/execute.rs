use crate::maxed_consensus_params;
use crate::setup::TestSetup;
use crate::TestResult;
use crate::TEST_METADATA_SEED;
use forc_pkg::PkgTestEntry;
use fuel_tx::{self as tx, output::contract::Contract, Chargeable, Finalizable};
use fuel_vm::error::InterpreterError;
use fuel_vm::{
    self as vm,
    checked_transaction::builder::TransactionBuilderExt,
    interpreter::{Interpreter, NotSupportedEcal},
    prelude::{Instruction, SecretKey},
    storage::MemoryStorage,
};
use rand::{Rng, SeedableRng};

use tx::Receipt;

use vm::interpreter::InterpreterParams;
use vm::state::DebugEval;
use vm::state::ProgramState;

/// An interface for executing a test within a VM [Interpreter] instance.
#[derive(Debug, Clone)]
pub struct TestExecutor {
    pub interpreter: Interpreter<MemoryStorage, tx::Script, NotSupportedEcal>,
    pub tx: vm::checked_transaction::Ready<tx::Script>,
    pub test_entry: PkgTestEntry,
    pub name: String,
}

/// The result of executing a test with breakpoints enabled.
#[derive(Debug)]
pub enum DebugResult {
    // Holds the test result.
    TestComplete(TestResult),
    // Holds the program counter of where the program stopped due to a breakpoint.
    Breakpoint(u64),
}

impl TestExecutor {
    pub fn build(
        bytecode: &[u8],
        test_offset: u32,
        test_setup: TestSetup,
        test_entry: &PkgTestEntry,
        name: String,
    ) -> anyhow::Result<Self> {
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
        // NOTE: fuel-core is using dynamic asset id and interacting with the fuel-core, using static
        // asset id is not correct. But since forc-test maintains its own interpreter instance, correct
        // base asset id is indeed the static `tx::AssetId::BASE`.
        let asset_id = tx::AssetId::BASE;
        let tx_pointer = rng.gen();
        let block_height = (u32::MAX >> 1).into();
        let gas_price = 0;

        let mut tx_builder = tx::TransactionBuilder::script(bytecode, script_input_data);

        let params = maxed_consensus_params();

        tx_builder
            .with_params(params)
            .add_unsigned_coin_input(secret_key, utxo_id, amount, asset_id, tx_pointer)
            .maturity(maturity);

        let mut output_index = 1;
        // Insert contract ids into tx input
        for contract_id in test_setup.contract_ids() {
            tx_builder
                .add_input(tx::Input::contract(
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
        // Temporarily finalize to calculate `script_gas_limit`
        let tmp_tx = tx_builder.clone().finalize();
        // Get `max_gas` used by everything except the script execution. Add `1` because of rounding.
        let max_gas =
            tmp_tx.max_gas(consensus_params.gas_costs(), consensus_params.fee_params()) + 1;
        // Increase `script_gas_limit` to the maximum allowed value.
        tx_builder.script_gas_limit(consensus_params.tx_params().max_gas_per_tx() - max_gas);

        // We need to increase the tx size limit as the default is 110 * 1024 and for big tests
        // such as std and core this is not enough.

        let tx = tx_builder
            .finalize_checked(block_height)
            .into_ready(
                gas_price,
                consensus_params.gas_costs(),
                consensus_params.fee_params(),
            )
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;

        let interpreter_params = InterpreterParams::new(gas_price, &consensus_params);

        Ok(TestExecutor {
            interpreter: Interpreter::with_storage(storage, interpreter_params),
            tx,
            test_entry: test_entry.clone(),
            name,
        })
    }

    /// Execute the test with breakpoints enabled.
    pub fn start_debugging(&mut self) -> anyhow::Result<DebugResult> {
        let start = std::time::Instant::now();
        let transition = self
            .interpreter
            .transact(self.tx.clone())
            .map_err(|err: InterpreterError<_>| anyhow::anyhow!(err))?;
        let state = *transition.state();
        if let ProgramState::RunProgram(DebugEval::Breakpoint(breakpoint)) = state {
            // A breakpoint was hit, so we tell the client to stop.
            return Ok(DebugResult::Breakpoint(breakpoint.pc()));
        }
        let duration = start.elapsed();
        let (gas_used, logs) = Self::get_gas_and_receipts(transition.receipts().to_vec())?;
        let span = self.test_entry.span.clone();
        let file_path = self.test_entry.file_path.clone();
        let condition = self.test_entry.pass_condition.clone();
        let name = self.name.clone();
        Ok(DebugResult::TestComplete(TestResult {
            name,
            file_path,
            duration,
            span,
            state,
            condition,
            logs,
            gas_used,
        }))
    }

    /// Continue executing the test with breakpoints enabled.
    pub fn continue_debugging(&mut self) -> anyhow::Result<DebugResult> {
        let start = std::time::Instant::now();
        let state = self
            .interpreter
            .resume()
            .map_err(|err: InterpreterError<_>| {
                anyhow::anyhow!("VM failed to resume. {:?}", err)
            })?;
        if let ProgramState::RunProgram(DebugEval::Breakpoint(breakpoint)) = state {
            // A breakpoint was hit, so we tell the client to stop.
            return Ok(DebugResult::Breakpoint(breakpoint.pc()));
        }
        let duration = start.elapsed();
        let (gas_used, logs) = Self::get_gas_and_receipts(self.interpreter.receipts().to_vec())?; // TODO: calculate culumlative
        let span = self.test_entry.span.clone();
        let file_path = self.test_entry.file_path.clone();
        let condition = self.test_entry.pass_condition.clone();
        let name = self.name.clone();
        Ok(DebugResult::TestComplete(TestResult {
            name,
            file_path,
            duration,
            span,
            state,
            condition,
            logs,
            gas_used,
        }))
    }

    pub fn execute(&mut self) -> anyhow::Result<TestResult> {
        let start = std::time::Instant::now();
        let transition = self
            .interpreter
            .transact(self.tx.clone())
            .map_err(|err: InterpreterError<_>| anyhow::anyhow!(err))?;
        let state = *transition.state();

        let duration = start.elapsed();
        let (gas_used, logs) = Self::get_gas_and_receipts(transition.receipts().to_vec())?;
        let span = self.test_entry.span.clone();
        let file_path = self.test_entry.file_path.clone();
        let condition = self.test_entry.pass_condition.clone();
        let name = self.name.clone();
        Ok(TestResult {
            name,
            file_path,
            duration,
            span,
            state,
            condition,
            logs,
            gas_used,
        })
    }

    fn get_gas_and_receipts(receipts: Vec<Receipt>) -> anyhow::Result<(u64, Vec<Receipt>)> {
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
                matches!(receipt, tx::Receipt::Log { .. })
                    || matches!(receipt, tx::Receipt::LogData { .. })
            })
            .collect();
        Ok((gas_used, logs))
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
    let ji = vm::fuel_asm::op::ji(test_offset);
    let ji_bytes = ji.to_bytes();
    let start = PROGRAM_START_BYTE_OFFSET;
    let end = start + ji_bytes.len();
    let mut patched = bytecode.to_vec();
    patched.splice(start..end, ji_bytes);
    std::borrow::Cow::Owned(patched)
}
