pub mod storage;

use crate::{cmd, op::call::Abi};
use anyhow::{anyhow, Result};
use fuel_core_types::tai64::Tai64;
use fuel_tx::Receipt;
use fuel_vm::{
    fuel_asm::Word,
    fuel_types::BlockHeight,
    interpreter::{Interpreter, InterpreterParams, MemoryInstance},
    prelude::*,
    state::ProgramState,
};
use fuels::types::Token;
use fuels_core::{
    codec::{ABIDecoder, DecoderConfig},
    types::{param_types::ParamType, ContractId},
};
use std::{collections::HashMap, io::Read};
use storage::ShallowStorage;

/// A reader for VM memory that implements the necessary traits for ABI decoding
#[derive(Clone)]
pub struct MemoryReader<'a> {
    mem: &'a MemoryInstance,
    at: Word,
}

impl<'a> MemoryReader<'a> {
    pub fn new(mem: &'a MemoryInstance, at: Word) -> Self {
        Self { mem, at }
    }
}

impl Read for MemoryReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let at = self.at;
        self.at += buf.len() as Word;
        buf.copy_from_slice(self.mem.read(at, buf.len()).map_err(|_err| {
            std::io::Error::new(std::io::ErrorKind::Other, "Inaccessible memory")
        })?);
        Ok(buf.len())
    }
}

/// Interprets execution trace by stepping through VM execution until call receipts are encountered
pub async fn interpret_execution_trace(
    provider: &fuels::accounts::provider::Provider,
    mode: &cmd::call::ExecutionMode,
    consensus_params: &ConsensusParameters,
    script: &fuel_tx::Script,
    receipts: &[Receipt],
    storage_reads: Vec<fuel_core_types::services::executor::StorageReadReplayEvent>,
    abis: &HashMap<ContractId, Abi>,
) -> Result<Vec<TraceEvent>> {
    let mut tracer = CallRetTracer::new(abis);

    let block_height: BlockHeight = (provider.latest_block_height().await?).into();
    let gas_price = provider.latest_gas_price().await?;
    let block = provider
        .block_by_height(block_height)
        .await?
        .ok_or(anyhow!("Block not found"))?;

    // Create shallow storage with empty initial storage reads
    let storage = ShallowStorage {
        block_height,
        timestamp: Tai64::from_unix(
            block
                .header
                .time
                .ok_or(anyhow!("Block time not found"))?
                .timestamp(),
        ),
        consensus_parameters_version: block.header.consensus_parameters_version,
        state_transition_version: block.header.state_transition_bytecode_version,
        coinbase: Default::default(), // TODO: get from tx
        storage: std::cell::RefCell::new(ShallowStorage::initial_storage(storage_reads)),
    };

    let script_tx = script
        .clone()
        .into_checked_basic(block_height, consensus_params)
        .map_err(|err| anyhow!("Failed to check transaction: {err:?}"))?
        .into_ready(
            gas_price.gas_price,
            consensus_params.gas_costs(),
            consensus_params.fee_params(),
            None,
        )
        .map_err(|err| anyhow!("Failed to check transaction: {err:?}"))?;

    let mut vm = Interpreter::<_, _, Script>::with_storage(
        MemoryInstance::new(),
        storage.clone(),
        InterpreterParams::new(gas_price.gas_price, consensus_params),
    );
    vm.set_single_stepping(true);

    let mut t = *vm
        .transact(script_tx)
        .map_err(|e| anyhow!("Failed to transact in trace interpreter: {e:?}"))?
        .state();
    loop {
        tracer.process_vm_state(&vm)?;
        match t {
            ProgramState::Return(_) | ProgramState::ReturnData(_) | ProgramState::Revert(_) => {
                break
            }
            ProgramState::RunProgram(_) | ProgramState::VerifyPredicate(_) => {
                t = vm
                    .resume()
                    .map_err(|e| anyhow!("Failed to resume VM in trace interpreter: {e:?}"))?;
            }
        }
    }

    if vm.receipts() != receipts {
        match mode {
            cmd::call::ExecutionMode::Live => return Err(anyhow!("Receipts mismatch")),
            _ => forc_tracing::println_warning(
                "Receipts mismatch; this is expected for non-live mode",
            ),
        }
    }

    Ok(tracer.into_events())
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TraceEvent {
    Call {
        /// Which receipt this call corresponds to.
        index: usize,
        /// Method being called.
        method: String,
        /// Arguments being passed to the method.
        arguments: Option<Vec<String>>,
        /// Contract being called
        to: ContractId,
        /// Amount being transferred
        amount: u64,
        /// Gas for the call
        gas: u64,
    },
    Return {
        index: usize,
        /// Contract that returned
        id: ContractId,
        /// Return value (raw)
        val: u64,
    },
    ReturnData {
        index: usize,
        /// Contract that returned data
        id: ContractId,
        /// Return data; decoded if ABI provided, otherwise hex encoded
        data: String,
    },
    Panic {
        index: usize,
        /// Contract that panicked
        id: ContractId,
        /// Panic reason
        reason: String,
        /// Contract ID associated with the panic, if any
        contract_id: Option<ContractId>,
    },
    Revert {
        index: usize,
        /// Contract that reverted
        id: ContractId,
        /// Revert value
        ra: u64,
    },
    Log {
        index: usize,
        /// Contract that logged
        id: ContractId,
        /// Log values
        ra: u64,
        rb: u64,
        rc: u64,
        rd: u64,
    },
    LogData {
        index: usize,
        /// Contract that logged data
        id: ContractId,
        /// Decoded log data value
        value: Option<String>,
        /// Data length
        len: u64,
    },
    Transfer {
        index: usize,
        /// Source contract
        id: ContractId,
        /// Destination (either contract or address)
        to: String,
        /// Amount transferred
        amount: u64,
        /// Asset ID
        asset_id: String,
    },
    ScriptResult {
        index: usize,
        /// Script execution result
        result: ScriptExecutionResult,
        /// Gas used
        gas_used: u64,
    },
    MessageOut {
        index: usize,
        /// Sender address
        sender: String,
        /// Recipient address
        recipient: String,
        /// Nonce
        nonce: u64,
        /// Digest
        digest: String,
        /// Amount
        amount: u64,
        /// Message data (hex encoded)
        data: Option<String>,
    },
    Mint {
        index: usize,
        /// Contract that minted
        contract_id: ContractId,
        /// Sub asset ID
        asset_id: String,
        /// Amount minted
        val: u64,
    },
    Burn {
        index: usize,
        /// Contract that burned
        contract_id: ContractId,
        /// Sub asset ID
        asset_id: String,
        /// Amount burned
        val: u64,
    },
}

/// Format transaction trace events into a hierarchical trace visualization.
/// This function processes trace events sequentially and displays them with proper indentation
/// based on call depth, similar to the original format_transaction_trace function.
pub fn display_transaction_trace<W: std::io::Write>(
    total_gas: u64,
    trace_events: &[TraceEvent],
    labels: &HashMap<ContractId, String>,
    writer: &mut W,
) -> Result<()> {
    use ansiterm::Color;
    let format_contract_with_label =
        |contract_id: ContractId, labels: &HashMap<ContractId, String>| -> String {
            if let Some(label) = labels.get(&contract_id) {
                label.to_string()
            } else {
                format!("0x{}", contract_id)
            }
        };

    writeln!(writer, "Traces:")?;
    writeln!(writer, "  [Script]")?;

    let mut depth = 0;
    for event in trace_events {
        let indent = if depth > 0 {
            "    │".repeat(depth)
        } else {
            "".to_string()
        };
        match event {
            TraceEvent::Call {
                to,
                gas,
                method,
                arguments,
                ..
            } => {
                writeln!(
                    writer,
                    "{}    ├─ [{}] {}{}{}({})",
                    indent,
                    gas,
                    Color::Green.paint(format_contract_with_label(*to, labels)),
                    Color::DarkGray.paint("::"),
                    method,
                    Color::DarkGray.paint(arguments.as_ref().unwrap_or(&vec![]).join(", "))
                )?;
                depth += 1;
            }
            TraceEvent::ReturnData { data, .. } => {
                writeln!(
                    writer,
                    "{}    └─ ← {}",
                    indent,
                    Color::BrightCyan.paint(data),
                )?;
                depth = depth.saturating_sub(1);
            }
            TraceEvent::Return { val, .. } => {
                writeln!(writer, "{}    └─ ← [Return] val: {}", indent, val)?;
                depth = depth.saturating_sub(1);
            }
            TraceEvent::LogData { value, .. } => {
                if let Some(log_value) = value {
                    writeln!(
                        writer,
                        "{}    ├─ emit {}",
                        indent,
                        Color::BrightCyan.paint(log_value)
                    )?;
                } else {
                    writeln!(writer, "{}    ├─ emit ()", indent)?;
                }
            }
            TraceEvent::Revert { .. } => {
                writeln!(
                    writer,
                    "{}    └─ ← {}",
                    indent,
                    Color::Red.paint("[Revert]")
                )?;
                depth = depth.saturating_sub(1);
            }
            TraceEvent::Panic { reason, .. } => {
                writeln!(
                    writer,
                    "{}    └─ ← {} {}",
                    indent,
                    Color::Red.paint("[Panic]"),
                    Color::Red.paint(reason)
                )?;
                depth = depth.saturating_sub(1);
            }
            TraceEvent::Transfer {
                amount,
                asset_id,
                to,
                ..
            } => {
                writeln!(
                    writer,
                    "{}    ├─ [Transfer] to:{} asset_id:{} amount:{}",
                    indent, to, asset_id, amount
                )?;
            }
            TraceEvent::Mint { asset_id, val, .. } => {
                writeln!(
                    writer,
                    "{}    ├─ [Mint] asset_id:{} val:{}",
                    indent, asset_id, val
                )?;
            }
            TraceEvent::Burn { asset_id, val, .. } => {
                writeln!(
                    writer,
                    "{}    ├─ [Burn] asset_id:{} val:{}",
                    indent, asset_id, val
                )?;
            }

            TraceEvent::Log { rb, .. } => {
                writeln!(writer, "{}    ├─ [Log] rb: 0x{:x}", indent, rb)?;
            }
            TraceEvent::MessageOut {
                amount,
                recipient,
                nonce,
                digest,
                data,
                ..
            } => {
                writeln!(
                    writer,
                    "{}    ├─ [MessageOut] recipient:{} amount:{} nonce:{} digest:{} data:{}",
                    indent,
                    recipient,
                    amount,
                    nonce,
                    digest,
                    data.clone().unwrap_or("()".to_string())
                )?;
            }
            TraceEvent::ScriptResult {
                result, gas_used, ..
            } => {
                writeln!(
                    writer,
                    "  [ScriptResult] result: {:?}, gas_used: {}",
                    result, gas_used
                )?;
                writeln!(writer)?;

                match result {
                    ScriptExecutionResult::Success => writeln!(
                        writer,
                        "{}",
                        Color::Green.paint("Transaction successfully executed.")
                    )?,
                    _ => writeln!(writer, "{}", Color::Red.paint("Transaction failed."))?,
                }
            }
        }
    }
    writeln!(writer, "Gas used: {}", total_gas)?;
    Ok(())
}

pub type Vm =
    Interpreter<MemoryInstance, ShallowStorage, Script, fuel_vm::interpreter::NotSupportedEcal>;

pub struct CallRetTracer<'a> {
    abis: &'a HashMap<ContractId, Abi>,
    return_type_callstack: Vec<StackFrame>,
    events: Vec<TraceEvent>,
}

enum StackFrame {
    KnownAbi(ParamType),
    UnknownAbi,
}

impl<'a> CallRetTracer<'a> {
    pub fn new(abis: &'a HashMap<ContractId, Abi>) -> Self {
        Self {
            abis,
            return_type_callstack: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn process_vm_state(&mut self, vm: &Vm) -> Result<()> {
        let start_index = self.events.len();
        let decoder = ABIDecoder::new(DecoderConfig::default());

        for (i, receipt) in vm.receipts().iter().enumerate().skip(start_index) {
            let index = i + start_index;
            let event = match receipt {
                Receipt::Call {
                    to,
                    param1,
                    param2,
                    amount,
                    gas,
                    ..
                } => {
                    let method = match decoder
                        .decode(&ParamType::String, MemoryReader::new(vm.memory(), *param1))
                    {
                        Ok(Token::String(method)) => Some(method),
                        _ => None,
                    };

                    let arguments = if let Some((parameters, returns)) = method
                        .as_ref()
                        .and_then(|m| get_function_signature(self.abis.get(to)?, m.as_str()))
                    {
                        self.return_type_callstack
                            .push(StackFrame::KnownAbi(returns));
                        let args_reader = MemoryReader::new(vm.memory(), *param2);
                        decoder
                            .decode_multiple_as_debug_str(parameters.as_slice(), args_reader)
                            .ok()
                    } else {
                        self.return_type_callstack.push(StackFrame::UnknownAbi);
                        None
                    };

                    TraceEvent::Call {
                        index,
                        method: method.unwrap_or("unknown".to_string()),
                        arguments,
                        to: *to,
                        amount: *amount,
                        gas: *gas,
                    }
                }

                Receipt::Return { id, val, .. } => {
                    if !self.return_type_callstack.is_empty() {
                        let _ = self.return_type_callstack.pop().unwrap();
                    }
                    TraceEvent::Return {
                        index,
                        id: *id,
                        val: *val,
                    }
                }

                Receipt::ReturnData { id, ptr, data, .. } => {
                    let return_value = match self.return_type_callstack.pop() {
                        Some(StackFrame::KnownAbi(return_type)) => {
                            let reader = MemoryReader::new(vm.memory(), *ptr);
                            decoder
                                .decode_as_debug_str(&return_type, reader)
                                .unwrap_or_else(|_| match data {
                                    Some(data) if !data.is_empty() => {
                                        format!("0x{}", hex::encode(data))
                                    }
                                    _ => "()".to_string(),
                                })
                        }
                        Some(StackFrame::UnknownAbi) | None => match data {
                            // hex encode the data if available
                            Some(data) if !data.is_empty() => format!("0x{}", hex::encode(data)),
                            _ => "()".to_string(),
                        },
                    };

                    TraceEvent::ReturnData {
                        index,
                        data: return_value,
                        id: *id,
                    }
                }

                Receipt::Panic {
                    id,
                    reason,
                    contract_id,
                    ..
                } => TraceEvent::Panic {
                    index,
                    id: *id,
                    reason: format!("{:?}", reason.reason()),
                    contract_id: *contract_id,
                },

                Receipt::Revert { id, ra, .. } => TraceEvent::Revert {
                    index,
                    id: *id,
                    ra: *ra,
                },

                Receipt::Log {
                    id, ra, rb, rc, rd, ..
                } => TraceEvent::Log {
                    index,
                    id: *id,
                    ra: *ra,
                    rb: *rb,
                    rc: *rc,
                    rd: *rd,
                },

                Receipt::LogData {
                    id, rb, len, data, ..
                } => {
                    let data_str = match data {
                        Some(data) => {
                            let hex_str = format!("0x{}", hex::encode(data));
                            match self.abis.get(id) {
                                Some(abi) => {
                                    let program_abi = sway_core::asm_generation::ProgramABI::Fuel(
                                        abi.program.clone(),
                                    );
                                    forc_util::tx_utils::decode_log_data(
                                        &rb.to_string(),
                                        data,
                                        &program_abi,
                                    )
                                    .ok()
                                    .map(|decoded| decoded.value)
                                }
                                None => Some(hex_str),
                            }
                        }
                        None => None,
                    };
                    TraceEvent::LogData {
                        index,
                        value: data_str,
                        id: *id,
                        len: *len,
                    }
                }

                Receipt::Transfer {
                    id,
                    to,
                    amount,
                    asset_id,
                    ..
                } => TraceEvent::Transfer {
                    index,
                    id: *id,
                    to: format!("0x{}", to),
                    amount: *amount,
                    asset_id: format!("0x{}", asset_id),
                },

                Receipt::TransferOut {
                    id,
                    to,
                    amount,
                    asset_id,
                    ..
                } => TraceEvent::Transfer {
                    index,
                    id: *id,
                    to: format!("0x{}", to),
                    amount: *amount,
                    asset_id: format!("0x{}", asset_id),
                },

                Receipt::ScriptResult { result, gas_used } => TraceEvent::ScriptResult {
                    index,
                    result: *result,
                    gas_used: *gas_used,
                },

                Receipt::MessageOut {
                    sender,
                    recipient,
                    amount,
                    data,
                    ..
                } => {
                    let data_hex = data.as_ref().map(|d| format!("0x{}", hex::encode(d)));
                    TraceEvent::MessageOut {
                        index,
                        sender: format!("0x{}", sender),
                        recipient: format!("0x{}", recipient),
                        amount: *amount,
                        data: data_hex,
                        nonce: 0,
                        digest:
                            "0x0000000000000000000000000000000000000000000000000000000000000000"
                                .to_string(),
                    }
                }

                Receipt::Mint {
                    contract_id,
                    sub_id,
                    val,
                    ..
                } => TraceEvent::Mint {
                    index,
                    contract_id: *contract_id,
                    asset_id: format!("0x{}", sub_id),
                    val: *val,
                },

                Receipt::Burn {
                    contract_id,
                    sub_id,
                    val,
                    ..
                } => TraceEvent::Burn {
                    index,
                    contract_id: *contract_id,
                    asset_id: format!("0x{}", sub_id),
                    val: *val,
                },
            };
            self.events.push(event);
        }

        Ok(())
    }

    pub fn into_events(self) -> Vec<TraceEvent> {
        self.events
    }
}

/// Extract function signature (parameters and return type) from ABI
fn get_function_signature(abi: &Abi, method: &str) -> Option<(Vec<ParamType>, ParamType)> {
    let func = abi.unified.functions.iter().find(|f| f.name == *method)?;

    let mut parameters = Vec::new();
    for param in &func.inputs {
        parameters.push(ParamType::try_from_type_application(param, &abi.type_lookup).ok()?);
    }

    let returns = ParamType::try_from_type_application(&func.output, &abi.type_lookup).ok()?;
    Some((parameters, returns))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fuel_tx::ScriptExecutionResult;
    use fuels_core::types::ContractId;
    use std::str::FromStr;

    // Compare the results, ignoring whitespace differences and colors
    fn normalize(s: &str) -> String {
        // Remove ANSI color codes
        let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        let s = re.replace_all(s, "");
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn test_display_transaction_trace_revert() {
        let contract1_id = ContractId::from_str(
            "4211b7b7a0c3104e6b9450b7a9e1b7f61912c57c3b319a956d5d7f95b480eb8e",
        )
        .unwrap();
        let contract2_id = ContractId::from_str(
            "f6035b8ac5ad76c228784d03fbba08545820715e811f574ff77300eab5e1aee9",
        )
        .unwrap();

        let trace_events = vec![
            TraceEvent::Call {
                index: 0,
                method: "unknown".to_string(),
                arguments: None,
                to: contract1_id,
                amount: 0,
                gas: 46590,
            },
            TraceEvent::Call {
                index: 1,
                method: "unknown".to_string(),
                arguments: None,
                to: contract2_id,
                amount: 0,
                gas: 34124,
            },
            TraceEvent::LogData {
                index: 2,
                id: contract2_id,
                value: Some("0x0000000000000001".to_string()),
                len: 8,
            },
            TraceEvent::Revert {
                index: 3,
                id: contract2_id,
                ra: 0,
            },
            TraceEvent::ScriptResult {
                index: 4,
                result: ScriptExecutionResult::Revert,
                gas_used: 37531,
            },
        ];

        let mut output = Vec::new();
        display_transaction_trace(0, &trace_events, &HashMap::new(), &mut output).unwrap();
        let trace_output = String::from_utf8(output).unwrap();

        let expected_output = r#"
        Traces:
          [Script]
            ├─ [46590] 0x4211b7b7a0c3104e6b9450b7a9e1b7f61912c57c3b319a956d5d7f95b480eb8e::unknown()
            │    ├─ [34124] 0xf6035b8ac5ad76c228784d03fbba08545820715e811f574ff77300eab5e1aee9::unknown()
            │    │    ├─ emit 0x0000000000000001
            │    │    └─ ← [Revert]
          [ScriptResult] result: Revert, gas_used: 37531

        Transaction failed.
        Gas used: 0
        "#;

        assert_eq!(
            normalize(&trace_output),
            normalize(expected_output),
            "\nExpected:\n{}\n\nActual:\n{}\n",
            expected_output,
            trace_output
        );
    }

    #[test]
    fn test_display_transaction_trace_simple_call() {
        let contract_id = ContractId::from_str(
            "2af09151f8276611ba65f14650970657bc42c1503d6502ffbb4d085ec37065dd",
        )
        .unwrap();

        let trace_events = vec![
            TraceEvent::Call {
                index: 0,
                method: "unknown".to_string(),
                arguments: None,
                to: contract_id,
                amount: 0,
                gas: 8793,
            },
            TraceEvent::ReturnData {
                index: 1,
                id: contract_id,
                data: "0x00000000000000000000000000000001".to_string(),
            },
            TraceEvent::Return {
                index: 2,
                id: ContractId::zeroed(),
                val: 1,
            },
            TraceEvent::ScriptResult {
                index: 3,
                result: ScriptExecutionResult::Success,
                gas_used: 12400,
            },
        ];

        let mut output = Vec::new();
        display_transaction_trace(0, &trace_events, &HashMap::new(), &mut output).unwrap();
        let trace_output = String::from_utf8(output).unwrap();

        let expected_output = r#"
        Traces:
          [Script]
            ├─ [8793] 0x2af09151f8276611ba65f14650970657bc42c1503d6502ffbb4d085ec37065dd::unknown()
            │    └─ ← 0x00000000000000000000000000000001
            └─ ← [Return] val: 1
          [ScriptResult] result: Success, gas_used: 12400

        Transaction successfully executed.
        Gas used: 0
        "#;

        assert_eq!(
            normalize(&trace_output),
            normalize(expected_output),
            "\nExpected:\n{}\n\nActual:\n{}\n",
            expected_output,
            trace_output
        );
    }

    #[test]
    fn test_display_transaction_trace_simple_call_log() {
        let contract_id = ContractId::from_str(
            "4a89a8fb150bf814a6610e1172baef6c68e4e273fce379fa9b30c75f584a697e",
        )
        .unwrap();

        let trace_events = vec![
            TraceEvent::Call {
                index: 0,
                method: "unknown".to_string(),
                arguments: None,
                to: contract_id,
                amount: 0,
                gas: 28311,
            },
            TraceEvent::LogData {
                index: 1,
                id: contract_id,
                value: Some("0x00000000000000000000000000000001".to_string()),
                len: 8,
            },
            TraceEvent::ReturnData {
                index: 2,
                id: contract_id,
                data: "()".to_string(),
            },
            TraceEvent::Return {
                index: 3,
                id: ContractId::zeroed(),
                val: 1,
            },
            TraceEvent::ScriptResult {
                index: 4,
                result: ScriptExecutionResult::Success,
                gas_used: 25412,
            },
        ];

        let mut output = Vec::new();
        display_transaction_trace(0, &trace_events, &HashMap::new(), &mut output).unwrap();
        let trace_output = String::from_utf8(output).unwrap();

        let expected_output = r#"
        Traces:
          [Script]
            ├─ [28311] 0x4a89a8fb150bf814a6610e1172baef6c68e4e273fce379fa9b30c75f584a697e::unknown()
            │     ├─ emit 0x00000000000000000000000000000001
            │     └─ ← ()
            └─ ← [Return] val: 1
          [ScriptResult] result: Success, gas_used: 25412

        Transaction successfully executed.
        Gas used: 0
        "#;

        assert_eq!(
            normalize(&trace_output),
            normalize(expected_output),
            "\nExpected:\n{}\n\nActual:\n{}\n",
            expected_output,
            trace_output
        );
    }

    #[test]
    fn test_display_transaction_trace_call_mint_transfer_burn() {
        let contract_id = ContractId::from_str(
            "5598f77f568631ad7e37e1d88b248d0c5002705ae4582fd544c9a87662a6af03",
        )
        .unwrap();

        let trace_events = vec![
            TraceEvent::Call {
                index: 0,
                method: "unknown".to_string(),
                arguments: None,
                to: contract_id,
                amount: 100,
                gas: 46023,
            },
            TraceEvent::Mint {
                index: 1,
                contract_id,
                asset_id: "0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
                val: 100,
            },
            TraceEvent::Transfer {
                index: 2,
                id: contract_id,
                to: "de97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c".to_string(),
                amount: 100,
                asset_id: "f8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07"
                    .to_string(),
            },
            TraceEvent::Burn {
                index: 3,
                contract_id,
                asset_id: "0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
                val: 100,
            },
            TraceEvent::ReturnData {
                index: 4,
                id: contract_id,
                data: "()".to_string(),
            },
            TraceEvent::Return {
                index: 5,
                id: ContractId::zeroed(),
                val: 1,
            },
            TraceEvent::ScriptResult {
                index: 6,
                result: ScriptExecutionResult::Success,
                gas_used: 37228,
            },
        ];

        let mut output = Vec::new();
        display_transaction_trace(0, &trace_events, &HashMap::new(), &mut output).unwrap();
        let trace_output = String::from_utf8(output).unwrap();

        let expected_output = r#"
        Traces:
          [Script]
            ├─ [46023] 0x5598f77f568631ad7e37e1d88b248d0c5002705ae4582fd544c9a87662a6af03::unknown()
            │    ├─ [Mint] asset_id:0000000000000000000000000000000000000000000000000000000000000000 val:100
            │    ├─ [Transfer] to:de97d8624a438121b86a1956544bd72ed68cd69f2c99555b08b1e8c51ffd511c asset_id:f8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07 amount:100
            │    ├─ [Burn] asset_id:0000000000000000000000000000000000000000000000000000000000000000 val:100
            │    └─ ← ()
            └─ ← [Return] val: 1
          [ScriptResult] result: Success, gas_used: 37228

        Transaction successfully executed.
        Gas used: 0
        "#;

        assert_eq!(
            normalize(&trace_output),
            normalize(expected_output),
            "\nExpected:\n{}\n\nActual:\n{}\n",
            expected_output,
            trace_output
        );
    }

    #[test]
    fn test_display_transaction_trace_nested_call_log_success() {
        let contract1_id = ContractId::from_str(
            "7c05fa2efa56c4bba646af0c48db02cba34e54149785cc692ae7e297f031b12e",
        )
        .unwrap();
        let contract2_id = ContractId::from_str(
            "7ecdf7b507b33131cac7295af2156fd98cd299c3512ec8c2733f920d5f8e4506",
        )
        .unwrap();

        let trace_events = vec![
            TraceEvent::Call {
                index: 0,
                method: "unknown".to_string(),
                arguments: None,
                to: contract1_id,
                amount: 0,
                gas: 47382,
            },
            TraceEvent::Call {
                index: 1,
                method: "unknown".to_string(),
                arguments: None,
                to: contract2_id,
                amount: 0,
                gas: 34914,
            },
            TraceEvent::LogData {
                index: 2,
                id: contract2_id,
                value: Some("0x00000000000000000000000000000001".to_string()),
                len: 8,
            },
            TraceEvent::ReturnData {
                index: 3,
                id: contract2_id,
                data: "()".to_string(),
            },
            TraceEvent::ReturnData {
                index: 4,
                id: contract1_id,
                data: "()".to_string(),
            },
            TraceEvent::Return {
                index: 5,
                id: ContractId::zeroed(),
                val: 1,
            },
            TraceEvent::ScriptResult {
                index: 6,
                result: ScriptExecutionResult::Success,
                gas_used: 38059,
            },
        ];

        let mut output = Vec::new();
        display_transaction_trace(0, &trace_events, &HashMap::new(), &mut output).unwrap();
        let trace_output = String::from_utf8(output).unwrap();

        let expected_output = r#"
        Traces:
          [Script]
            ├─ [47382] 0x7c05fa2efa56c4bba646af0c48db02cba34e54149785cc692ae7e297f031b12e::unknown()
            │    ├─ [34914] 0x7ecdf7b507b33131cac7295af2156fd98cd299c3512ec8c2733f920d5f8e4506::unknown()
            │    │    ├─ emit 0x00000000000000000000000000000001
            │    │    └─ ← ()
            │    └─ ← ()
            └─ ← [Return] val: 1
          [ScriptResult] result: Success, gas_used: 38059

        Transaction successfully executed.
        Gas used: 0
        "#;

        assert_eq!(
            normalize(&trace_output),
            normalize(expected_output),
            "\nExpected:\n{}\n\nActual:\n{}\n",
            expected_output,
            trace_output
        );
    }

    #[test]
    fn test_display_transaction_trace_nested_call_log_success_with_multiple_calls() {
        let contract1_id = ContractId::from_str(
            "41a231bd983812dd51e5778751fc679461b8f580357515848b3ac9a297c6e8bc",
        )
        .unwrap();
        let contract2_id = ContractId::from_str(
            "38bf64bfa5ee78b652a36c70eb89fd97caff5ffb419d0abf1199247c168b730c",
        )
        .unwrap();

        let trace_events = vec![
            TraceEvent::Call {
                index: 0,
                method: "unknown".to_string(),
                arguments: None,
                to: contract1_id,
                amount: 0,
                gas: 105141,
            },
            TraceEvent::Call {
                index: 1,
                method: "unknown".to_string(),
                arguments: None,
                to: contract2_id,
                amount: 0,
                gas: 92530,
            },
            TraceEvent::ReturnData {
                index: 2,
                id: contract2_id,
                data: "()".to_string(),
            },
            TraceEvent::LogData {
                index: 3,
                id: contract1_id,
                value: Some("0x00000000000000000000000000000001".to_string()),
                len: 25,
            },
            TraceEvent::Call {
                index: 4,
                method: "unknown".to_string(),
                arguments: None,
                to: contract2_id,
                amount: 0,
                gas: 67314,
            },
            TraceEvent::ReturnData {
                index: 5,
                id: contract2_id,
                data: "0x00000000000000000000000000000002".to_string(),
            },
            TraceEvent::LogData {
                index: 6,
                id: contract1_id,
                value: Some("0x00000000000000000000000000000002".to_string()),
                len: 8,
            },
            TraceEvent::LogData {
                index: 7,
                id: contract1_id,
                value: Some("0x00000000000000000000000000000003".to_string()),
                len: 12,
            },
            TraceEvent::Call {
                index: 8,
                method: "unknown".to_string(),
                arguments: None,
                to: contract2_id,
                amount: 0,
                gas: 53729,
            },
            TraceEvent::ReturnData {
                index: 9,
                id: contract2_id,
                data: "()".to_string(),
            },
            TraceEvent::ReturnData {
                index: 10,
                id: contract1_id,
                data: "()".to_string(),
            },
            TraceEvent::Return {
                index: 11,
                id: ContractId::zeroed(),
                val: 1,
            },
            TraceEvent::ScriptResult {
                index: 12,
                result: ScriptExecutionResult::Success,
                gas_used: 76612,
            },
        ];

        let mut output = Vec::new();
        display_transaction_trace(0, &trace_events, &HashMap::new(), &mut output).unwrap();
        let trace_output = String::from_utf8(output).unwrap();

        let expected_output = r#"
        Traces:
          [Script]
            ├─ [105141] 0x41a231bd983812dd51e5778751fc679461b8f580357515848b3ac9a297c6e8bc::unknown()
            │    ├─ [92530] 0x38bf64bfa5ee78b652a36c70eb89fd97caff5ffb419d0abf1199247c168b730c::unknown()
            │    │    └─ ← ()
            │    ├─ emit 0x00000000000000000000000000000001
            │    ├─ [67314] 0x38bf64bfa5ee78b652a36c70eb89fd97caff5ffb419d0abf1199247c168b730c::unknown()
            │    │    └─ ← 0x00000000000000000000000000000002
            │    ├─ emit 0x00000000000000000000000000000002
            │    ├─ emit 0x00000000000000000000000000000003
            │    ├─ [53729] 0x38bf64bfa5ee78b652a36c70eb89fd97caff5ffb419d0abf1199247c168b730c::unknown()
            │    │    └─ ← ()
            │    └─ ← ()
            └─ ← [Return] val: 1
          [ScriptResult] result: Success, gas_used: 76612

        Transaction successfully executed.
        Gas used: 0
        "#;

        assert_eq!(
            normalize(&trace_output),
            normalize(expected_output),
            "\nExpected:\n{}\n\nActual:\n{}\n",
            expected_output,
            trace_output
        );
    }

    #[test]
    fn test_display_transaction_trace_nested_call_log_revert() {
        let contract1_id = ContractId::from_str(
            "9a7195648cc46c832e490e9bc15ed929fa82801cc0316d1c8e0965bb5e0260a3",
        )
        .unwrap();
        let contract2_id = ContractId::from_str(
            "b56b9921112e2fed854ac85357a4914dab561eed98fed0cbe35c1871971dc129",
        )
        .unwrap();

        let trace_events = vec![
            TraceEvent::Call {
                index: 0,
                method: "unknown".to_string(),
                arguments: None,
                to: contract1_id,
                amount: 0,
                gas: 46590,
            },
            TraceEvent::Call {
                index: 1,
                method: "unknown".to_string(),
                arguments: None,
                to: contract2_id,
                amount: 0,
                gas: 34124,
            },
            TraceEvent::LogData {
                index: 2,
                id: contract2_id,
                value: Some("0x00000000000000000000000000000001".to_string()),
                len: 8,
            },
            TraceEvent::Revert {
                index: 3,
                id: contract2_id,
                ra: 0,
            },
            TraceEvent::ScriptResult {
                index: 4,
                result: ScriptExecutionResult::Revert,
                gas_used: 37531,
            },
        ];

        let mut output = Vec::new();
        display_transaction_trace(0, &trace_events, &HashMap::new(), &mut output).unwrap();
        let trace_output = String::from_utf8(output).unwrap();

        // Expected output with color codes
        let expected_output = r#"
        Traces:
          [Script]
            ├─ [46590] 0x9a7195648cc46c832e490e9bc15ed929fa82801cc0316d1c8e0965bb5e0260a3::unknown()
            │    ├─ [34124] 0xb56b9921112e2fed854ac85357a4914dab561eed98fed0cbe35c1871971dc129::unknown()
            │    │    ├─ emit 0x00000000000000000000000000000001
            │    │    └─ ← [Revert]
          [ScriptResult] result: Revert, gas_used: 37531

        Transaction failed.
        Gas used: 0
        "#;

        assert_eq!(
            normalize(&trace_output),
            normalize(expected_output),
            "\nExpected:\n{}\n\nActual:\n{}\n",
            expected_output,
            trace_output
        );
    }

    #[test]
    fn test_display_transaction_trace_nested_call_log_panic() {
        let contract1_id = ContractId::from_str(
            "b09d73495f6c211ff3586a0542d5fe5fbd45a80e1cd2c1a9a787d6865cc65984",
        )
        .unwrap();
        let contract2_id = ContractId::from_str(
            "75c5015d5243cfd798a7f46eb8cf3338e05197e0a271b43c4703764c82d60080",
        )
        .unwrap();

        let trace_events = vec![
            TraceEvent::Call {
                index: 0,
                method: "unknown".to_string(),
                arguments: None,
                to: contract1_id,
                amount: 0,
                gas: 25156,
            },
            TraceEvent::Call {
                index: 1,
                method: "unknown".to_string(),
                arguments: None,
                to: contract2_id,
                amount: 0,
                gas: 12432,
            },
            TraceEvent::Panic {
                index: 2,
                id: contract2_id,
                reason: "PanicInstruction { reason: MemoryOwnership, instruction: MCP { dst_addr: 0x13, src_addr: 0x14, len: 0x15 } (bytes: 28 4d 45 40) }".to_string(),
                contract_id: None,
            },
            TraceEvent::ScriptResult {
                index: 3,
                result: ScriptExecutionResult::Panic,
                gas_used: 23242,
            },
        ];

        let mut output = Vec::new();
        display_transaction_trace(0, &trace_events, &HashMap::new(), &mut output).unwrap();
        let trace_output = String::from_utf8(output).unwrap();

        let expected_output = r#"
        Traces:
          [Script]
            ├─ [25156] 0xb09d73495f6c211ff3586a0542d5fe5fbd45a80e1cd2c1a9a787d6865cc65984::unknown()
            │    ├─ [12432] 0x75c5015d5243cfd798a7f46eb8cf3338e05197e0a271b43c4703764c82d60080::unknown()
            │    │    └─ ← [Panic] PanicInstruction { reason: MemoryOwnership, instruction: MCP { dst_addr: 0x13, src_addr: 0x14, len: 0x15 } (bytes: 28 4d 45 40) }
          [ScriptResult] result: Panic, gas_used: 23242

        Transaction failed.
        Gas used: 0
        "#;

        assert_eq!(
            normalize(&trace_output),
            normalize(expected_output),
            "\nExpected:\n{}\n\nActual:\n{}\n",
            expected_output,
            trace_output
        );
    }

    #[test]
    fn test_display_transaction_trace_with_labels() {
        let contract_id = ContractId::from_str(
            "2af09151f8276611ba65f14650970657bc42c1503d6502ffbb4d085ec37065dd",
        )
        .unwrap();

        let trace_events = vec![
            TraceEvent::Call {
                index: 0,
                method: "transfer".to_string(),
                arguments: Some(vec!["100".to_string(), "0x123".to_string()]),
                to: contract_id,
                amount: 0,
                gas: 8793,
            },
            TraceEvent::ReturnData {
                index: 1,
                id: contract_id,
                data: "()".to_string(),
            },
            TraceEvent::Return {
                index: 2,
                id: ContractId::zeroed(),
                val: 1,
            },
            TraceEvent::ScriptResult {
                index: 3,
                result: ScriptExecutionResult::Success,
                gas_used: 12400,
            },
        ];

        // Create labels map
        let mut labels = HashMap::new();
        labels.insert(contract_id, "TokenContract".to_string());

        let mut output = Vec::new();
        display_transaction_trace(0, &trace_events, &labels, &mut output).unwrap();
        let trace_output = String::from_utf8(output).unwrap();

        let expected_output = r#"
        Traces:
          [Script]
            ├─ [8793] TokenContract::transfer(100, 0x123)
            │    └─ ← ()
            └─ ← [Return] val: 1
          [ScriptResult] result: Success, gas_used: 12400

        Transaction successfully executed.
        Gas used: 0
        "#;

        assert_eq!(
            normalize(&trace_output),
            normalize(expected_output),
            "\nExpected:\n{}\n\nActual:\n{}\n",
            expected_output,
            trace_output
        );
    }
}
