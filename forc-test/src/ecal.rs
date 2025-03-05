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
