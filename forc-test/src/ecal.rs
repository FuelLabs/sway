use std::sync::Arc;

use fuel_vm::{
    error::SimpleResult,
    interpreter::{EcalHandler, Interpreter},
    prelude::{Memory, RegId},
};
use sway_core::asm_generation::ProgramABI;

/// A handler for processing log data during test execution in the Fuel VM.
///
/// This handler decodes and displays log data emitted by scripts and predicates during test
/// execution, using the program's ABI to interpret the data in a human-readable format.
///
/// # Memory Layout
/// The handler expects the following register layout:
/// - Register a: *Ignored*
/// - Register b: The log ID that identifies the type of log data
/// - Register c: Pointer to the log data in memory
/// - Register d: Length of the log data
///
/// # Operation
/// When called, the handler:
/// 1. Reads the log ID and data from VM memory
/// 2. Attempts to decode the log data using the program's ABI
/// 3. Prints the decoded data (if successfully decoded) or the raw log data (if decoding fails)
///
/// # Output Format
/// - Successfully decoded data is printed to console
/// - Decoding failures are printed in red with additional debug information
///
/// # Dependencies
/// The struct requires a program ABI to properly decode log data according to the
/// types defined in the Sway program.
#[derive(Debug, Clone)]
pub struct PredicateLoggingEcal {
    pub program_abi: Arc<ProgramABI>,
}

impl EcalHandler for PredicateLoggingEcal {
    fn ecal<M, S, Tx>(
        vm: &mut Interpreter<M, S, Tx, Self>,
        a: RegId,
        b: RegId,
        c: RegId,
        d: RegId,
    ) -> SimpleResult<()>
    where
        M: Memory,
    {
        let a = vm.registers()[a];
        let log_id = vm.registers()[b];
        let ptr = vm.registers()[c];
        let len: u64 = vm.registers()[d];

        // Read bytes from VM memory at the given pointer and length
        let bytes = vm.memory().read(ptr, len)?.to_vec();

        let receipt = fuel_tx::Receipt::log_data(
            Default::default(),
            a,
            log_id,
            ptr,
            vm.registers()[RegId::PC],
            vm.registers()[RegId::IS],
            bytes,
        );
        vm.receipts_mut().push(receipt)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fuel_asm::op;
    use fuel_tx::{ConsensusParameters, Finalizable, Receipt, TransactionBuilder};
    use fuel_vm::prelude::{IntoChecked, MemoryClient};
    use sway_core::asm_generation::ProgramABI;

    #[test]
    fn test_predicate_logging_ecal() {
        let program_abi = Arc::new(ProgramABI::Fuel(Default::default()));
        let vm = Interpreter::with_memory_storage_and_ecal(PredicateLoggingEcal { program_abi });

        // Create test data and script
        let test_input = "Hello, PredicateLoggingEcal!";
        let script_data: Vec<u8> = test_input.bytes().collect();
        let script = vec![
            // set log id
            op::movi(0x20, 0x123),
            // ptr to script data
            op::gtf_args(0x10, 0x00, fuel_asm::GTFArgs::ScriptData),
            // length of script data
            op::movi(0x21, script_data.len().try_into().unwrap()),
            // call ECAL; add log data to receipts
            op::ecal(RegId::ZERO, 0x20, 0x10, 0x21),
            // return
            op::ret(RegId::ONE),
        ]
        .into_iter()
        .collect();

        // Execute transaction
        let mut client = MemoryClient::from_txtor(vm.into());
        let tx = TransactionBuilder::script(script, script_data)
            .script_gas_limit(1_000_000)
            .add_fee_input()
            .finalize()
            .into_checked(Default::default(), &ConsensusParameters::standard())
            .expect("failed to generate a checked tx");
        client.transact(tx);

        // Verify ECAL pushes log data to receipts
        let receipt = client
            .receipts()
            .expect("Expected receipts")
            .first()
            .unwrap();
        let bytes = match receipt {
            Receipt::LogData { data, .. } => data.as_ref().unwrap().clone(),
            _ => panic!("Expected LogData receipt"),
        };
        let output = String::from_utf8(bytes).unwrap();

        assert_eq!(output, test_input);
    }
}
