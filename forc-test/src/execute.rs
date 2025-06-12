use crate::ecal::EcalSyscallHandler;
use crate::maxed_consensus_params;
use crate::setup::TestSetup;
use crate::TestResult;
use crate::TEST_METADATA_SEED;
use forc_pkg::PkgTestEntry;
use fuel_tx::{self as tx, output::contract::Contract, Chargeable, Finalizable};
use fuel_vm::error::InterpreterError;
use fuel_vm::fuel_asm;
use fuel_vm::prelude::Instruction;
use fuel_vm::prelude::RegId;
use fuel_vm::{
    self as vm, checked_transaction::builder::TransactionBuilderExt, interpreter::Interpreter,
    prelude::SecretKey, storage::MemoryStorage,
};
use rand::{Rng, SeedableRng};

use tx::Receipt;

use vm::interpreter::{InterpreterParams, MemoryInstance};
use vm::state::DebugEval;
use vm::state::ProgramState;

/// An interface for executing a test within a VM [Interpreter] instance.
#[derive(Debug, Clone)]
pub struct TestExecutor {
    pub interpreter: Interpreter<MemoryInstance, MemoryStorage, tx::Script, EcalSyscallHandler>,
    pub tx: vm::checked_transaction::Ready<tx::Script>,
    pub test_entry: PkgTestEntry,
    pub name: String,
    pub jump_instruction_index: usize,
    pub relative_jump_in_bytes: u32,
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
        test_instruction_index: u32,
        test_setup: TestSetup,
        test_entry: &PkgTestEntry,
        name: String,
    ) -> anyhow::Result<Self> {
        let storage = test_setup.storage().clone();

        // Find the instruction which we will jump into the
        // specified test
        let jump_instruction_index = find_jump_instruction_index(bytecode);

        // Create a transaction to execute the test function.
        let script_input_data = vec![];
        let rng = &mut rand::rngs::StdRng::seed_from_u64(TEST_METADATA_SEED);

        // Prepare the transaction metadata.
        let secret_key = SecretKey::random(rng);
        let utxo_id = rng.r#gen();
        let amount = 1;
        let maturity = 1.into();
        // NOTE: fuel-core is using dynamic asset id and interacting with the fuel-core, using static
        // asset id is not correct. But since forc-test maintains its own interpreter instance, correct
        // base asset id is indeed the static `tx::AssetId::BASE`.
        let asset_id = tx::AssetId::BASE;
        let tx_pointer = rng.r#gen();
        let block_height = (u32::MAX >> 1).into();
        let gas_price = 0;

        let mut tx_builder = tx::TransactionBuilder::script(bytecode.to_vec(), script_input_data);

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
        // such as std this is not enough.

        let tx = tx_builder
            .finalize_checked(block_height)
            .into_ready(
                gas_price,
                consensus_params.gas_costs(),
                consensus_params.fee_params(),
                None,
            )
            .map_err(|e| anyhow::anyhow!("{e:?}"))?;

        let interpreter_params = InterpreterParams::new(gas_price, &consensus_params);
        let memory_instance = MemoryInstance::new();
        let interpreter = Interpreter::with_storage(memory_instance, storage, interpreter_params);

        Ok(TestExecutor {
            interpreter,
            tx,
            test_entry: test_entry.clone(),
            name,
            jump_instruction_index,
            relative_jump_in_bytes: (test_instruction_index - jump_instruction_index as u32)
                * Instruction::SIZE as u32,
        })
    }

    // single-step until the jump-to-test instruction, then
    // jump into the first instruction of the test
    fn single_step_until_test(&mut self) -> ProgramState {
        let jump_pc = (self.jump_instruction_index * Instruction::SIZE) as u64;

        let old_single_stepping = self.interpreter.single_stepping();
        self.interpreter.set_single_stepping(true);
        let mut state = {
            let transition = self.interpreter.transact(self.tx.clone());
            Ok(*transition.unwrap().state())
        };

        loop {
            match state {
                // if the VM fails, we interpret as a revert
                Err(_) => {
                    break ProgramState::Revert(0);
                }
                Ok(
                    state @ ProgramState::Return(_)
                    | state @ ProgramState::ReturnData(_)
                    | state @ ProgramState::Revert(_),
                ) => break state,
                Ok(
                    s @ ProgramState::RunProgram(eval) | s @ ProgramState::VerifyPredicate(eval),
                ) => {
                    // time to jump into the specified test
                    if let Some(b) = eval.breakpoint() {
                        if b.pc() == jump_pc {
                            self.interpreter.registers_mut()[RegId::PC] +=
                                self.relative_jump_in_bytes as u64;
                            self.interpreter.set_single_stepping(old_single_stepping);
                            break s;
                        }
                    }

                    state = self.interpreter.resume();
                }
            }
        }
    }

    /// Execute the test with breakpoints enabled.
    pub fn start_debugging(&mut self) -> anyhow::Result<DebugResult> {
        let start = std::time::Instant::now();

        let _ = self.single_step_until_test();
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
        let (gas_used, logs) = Self::get_gas_and_receipts(self.interpreter.receipts().to_vec())?;
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
            ecal: Box::new(self.interpreter.ecal_state().clone()),
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
            ecal: Box::new(self.interpreter.ecal_state().clone()),
        }))
    }

    pub fn execute(&mut self) -> anyhow::Result<TestResult> {
        self.interpreter.ecal_state_mut().clear();

        let start = std::time::Instant::now();

        let mut state = Ok(self.single_step_until_test());

        // Run test until its end
        loop {
            match state {
                Err(_) => {
                    state = Ok(ProgramState::Revert(0));
                    break;
                }
                Ok(
                    ProgramState::Return(_) | ProgramState::ReturnData(_) | ProgramState::Revert(_),
                ) => break,
                Ok(ProgramState::RunProgram(_) | ProgramState::VerifyPredicate(_)) => {
                    state = self.interpreter.resume();
                }
            }
        }

        let duration = start.elapsed();
        let (gas_used, logs) = Self::get_gas_and_receipts(self.interpreter.receipts().to_vec())?;
        let span = self.test_entry.span.clone();
        let file_path = self.test_entry.file_path.clone();
        let condition = self.test_entry.pass_condition.clone();
        let name = self.name.clone();
        Ok(TestResult {
            name,
            file_path,
            duration,
            span,
            state: state.unwrap(),
            condition,
            logs,
            gas_used,
            ecal: Box::new(self.interpreter.ecal_state().clone()),
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

fn find_jump_instruction_index(bytecode: &[u8]) -> usize {
    // Search first `move $$locbase $sp`
    // This will be `__entry` for script/predicate/contract using encoding v1;
    // `main` for script/predicate using encoding v0;
    // or the first function for libraries
    // MOVE R59 $sp                                    ;; [26, 236, 80, 0]
    let a = vm::fuel_asm::op::move_(59, fuel_asm::RegId::SP).to_bytes();

    // for contracts using encoding v0
    // search the first `lw $r0 $fp i73`
    // which is the start of the fn selector
    // LW $writable $fp 0x49                           ;; [93, 64, 96, 73]
    let b = vm::fuel_asm::op::lw(fuel_asm::RegId::WRITABLE, fuel_asm::RegId::FP, 73).to_bytes();

    bytecode
        .chunks(Instruction::SIZE)
        .position(|instruction| {
            let instruction: [u8; 4] = instruction.try_into().unwrap();
            instruction == a || instruction == b
        })
        .unwrap()
}
