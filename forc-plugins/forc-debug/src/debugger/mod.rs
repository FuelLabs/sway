pub mod commands;

pub use commands::{AbiMapping, BreakpointHit, DebugCommand, DebugResponse, RegisterValue};

use crate::{
    error::{Error, Result},
    names::register_name,
    types::AbiMap,
    ContractId, FuelClient, RunResult, Transaction,
};
use fuel_tx::Receipt;
use fuel_vm::consts::{VM_REGISTER_COUNT, WORD_SIZE};
use sway_core::asm_generation::ProgramABI;

pub struct Debugger {
    client: FuelClient,
    session_id: String,
    contract_abis: AbiMap,
}

impl Debugger {
    /// Create a debugger instance connected to the given API URL
    pub async fn new(api_url: &str) -> Result<Self> {
        let client = FuelClient::new(api_url).map_err(|e| Error::FuelClientError(e.to_string()))?;
        Self::from_client(client).await
    }

    /// Create a debugger instance from FuelClient
    pub async fn from_client(client: FuelClient) -> Result<Self> {
        let session_id = client
            .start_session()
            .await
            .map_err(|e| Error::FuelClientError(e.to_string()))?;

        Ok(Self {
            client,
            session_id,
            contract_abis: AbiMap::default(),
        })
    }

    /// Execute a debugger command from CLI arguments
    pub async fn execute_from_args<W: std::io::Write>(
        &mut self,
        args: Vec<String>,
        writer: &mut W,
    ) -> Result<()> {
        let command = DebugCommand::from_cli_args(&args)?;
        let response = self.execute(command).await?;
        match response {
            DebugResponse::RunResult {
                receipts,
                breakpoint,
            } => {
                // Process receipts with ABI decoding
                let decoded_receipts = self.process_receipts(&receipts);
                for decoded in decoded_receipts {
                    match decoded {
                        DecodedReceipt::Regular(receipt) => {
                            writeln!(writer, "Receipt: {receipt:?}")?;
                        }
                        DecodedReceipt::LogData {
                            receipt,
                            decoded_value,
                            contract_id,
                        } => {
                            writeln!(writer, "Receipt: {receipt:?}")?;
                            if let Some(value) = decoded_value {
                                writeln!(
                                    writer,
                                    "Decoded log value: {value}, from contract: {contract_id}"
                                )?;
                            }
                        }
                    }
                }
                // Print breakpoint info
                if let Some(bp) = breakpoint {
                    writeln!(
                        writer,
                        "Stopped on breakpoint at address {} of contract 0x{}",
                        bp.pc, bp.contract
                    )?;
                } else {
                    writeln!(writer, "Terminated")?;
                }
            }
            DebugResponse::Success => {
                // Command completed successfully, no output needed
            }
            DebugResponse::Registers(registers) => {
                for reg in registers {
                    writeln!(
                        writer,
                        "reg[{:#02x}] = {:<8} # {}",
                        reg.index, reg.value, reg.name
                    )?;
                }
            }
            DebugResponse::Memory(mem) => {
                for (i, chunk) in mem.chunks(WORD_SIZE).enumerate() {
                    write!(writer, " {:06x}:", i * WORD_SIZE)?;
                    for byte in chunk {
                        write!(writer, " {byte:02x}")?;
                    }
                    writeln!(writer)?;
                }
            }
            DebugResponse::Error(err) => {
                writeln!(writer, "Error: {err}")?;
            }
        }
        Ok(())
    }

    pub async fn execute(&mut self, command: DebugCommand) -> Result<DebugResponse> {
        match command {
            DebugCommand::StartTransaction {
                tx_path,
                abi_mappings,
            } => self.start_transaction(tx_path, abi_mappings).await,
            DebugCommand::Reset => self.reset().await,
            DebugCommand::Continue => self.continue_execution().await,
            DebugCommand::SetSingleStepping { enable } => self.set_single_stepping(enable).await,
            DebugCommand::SetBreakpoint {
                contract_id,
                offset,
            } => self.set_breakpoint(contract_id, offset).await,
            DebugCommand::GetRegisters { indices } => self.get_registers(indices).await,
            DebugCommand::GetMemory { offset, limit } => self.get_memory(offset, limit).await,
            DebugCommand::Quit => Ok(DebugResponse::Success),
        }
    }

    /// Start a new transaction with optional ABI support
    async fn start_transaction(
        &mut self,
        tx_path: String,
        abi_mappings: Vec<AbiMapping>,
    ) -> Result<DebugResponse> {
        let load_and_parse_abi = |abi_path: &str| -> Result<ProgramABI> {
            let abi_content = std::fs::read_to_string(abi_path)?;
            let fuel_abi =
                serde_json::from_str::<fuel_abi_types::abi::program::ProgramABI>(&abi_content)
                    .map_err(Error::JsonError)?;
            Ok(ProgramABI::Fuel(fuel_abi))
        };

        // Process ABI mappings
        for mapping in abi_mappings {
            match mapping {
                AbiMapping::Local { abi_path } => {
                    let abi = load_and_parse_abi(&abi_path)?;
                    self.contract_abis.register_abi(ContractId::zeroed(), abi);
                }
                AbiMapping::Contract {
                    contract_id,
                    abi_path,
                } => {
                    let abi = load_and_parse_abi(&abi_path)?;
                    self.contract_abis.register_abi(contract_id, abi);
                }
            }
        }

        // Load and start transaction
        let tx_json = std::fs::read(&tx_path)?;
        let tx: Transaction = serde_json::from_slice(&tx_json).map_err(Error::JsonError)?;

        let status = self
            .client
            .start_tx(&self.session_id, &tx)
            .await
            .map_err(|e| Error::FuelClientError(e.to_string()))?;

        Ok(self.create_run_result_response(&status))
    }

    async fn reset(&mut self) -> Result<DebugResponse> {
        self.client
            .reset(&self.session_id)
            .await
            .map_err(|e| Error::FuelClientError(e.to_string()))?;
        Ok(DebugResponse::Success)
    }

    async fn continue_execution(&mut self) -> Result<DebugResponse> {
        let status = self
            .client
            .continue_tx(&self.session_id)
            .await
            .map_err(|e| Error::FuelClientError(e.to_string()))?;
        Ok(self.create_run_result_response(&status))
    }

    async fn set_single_stepping(&mut self, enable: bool) -> Result<DebugResponse> {
        self.client
            .set_single_stepping(&self.session_id, enable)
            .await
            .map_err(|e| Error::FuelClientError(e.to_string()))?;
        Ok(DebugResponse::Success)
    }

    async fn set_breakpoint(
        &mut self,
        contract_id: ContractId,
        offset: u64,
    ) -> Result<DebugResponse> {
        self.client
            .set_breakpoint(&self.session_id, contract_id, offset)
            .await
            .map_err(|e| Error::FuelClientError(e.to_string()))?;
        Ok(DebugResponse::Success)
    }

    async fn get_registers(&mut self, indices: Vec<u32>) -> Result<DebugResponse> {
        let indices = if indices.is_empty() {
            (0..VM_REGISTER_COUNT as u32).collect()
        } else {
            indices
        };

        let mut values = Vec::new();
        for index in indices {
            if index >= VM_REGISTER_COUNT as u32 {
                return Err(Error::ArgumentError(crate::error::ArgumentError::Invalid(
                    format!("Register index too large: {index}"),
                )));
            }
            let value = self
                .client
                .register(&self.session_id, index)
                .await
                .map_err(|e| Error::FuelClientError(e.to_string()))?;
            values.push(RegisterValue {
                index,
                value,
                name: register_name(index as usize).to_string(),
            });
        }
        Ok(DebugResponse::Registers(values))
    }

    async fn get_memory(&mut self, offset: u32, limit: u32) -> Result<DebugResponse> {
        let mem = self
            .client
            .memory(&self.session_id, offset, limit)
            .await
            .map_err(|e| Error::FuelClientError(e.to_string()))?;
        Ok(DebugResponse::Memory(mem))
    }

    /// Convert RunResult to DebugResponse
    fn create_run_result_response(&self, run_result: &RunResult) -> DebugResponse {
        let receipts: Vec<Receipt> = run_result.receipts().collect();
        let breakpoint = run_result.breakpoint.as_ref().map(|bp| BreakpointHit {
            contract: bp.contract.clone().into(),
            pc: bp.pc.0,
        });
        DebugResponse::RunResult {
            receipts,
            breakpoint,
        }
    }

    /// Process receipts with ABI decoding (used for pretty printing in CLI)
    pub fn process_receipts(&mut self, receipts: &[Receipt]) -> Vec<DecodedReceipt> {
        receipts
            .iter()
            .map(|receipt| {
                if let Receipt::LogData {
                    id,
                    rb,
                    data: Some(data),
                    ..
                } = receipt
                {
                    self.contract_abis
                        .get_or_fetch_abi(id)
                        .and_then(|abi| {
                            forc_util::tx_utils::decode_log_data(&rb.to_string(), data, abi).ok()
                        })
                        .map(|decoded_log| DecodedReceipt::LogData {
                            receipt: receipt.clone(),
                            decoded_value: Some(decoded_log.value),
                            contract_id: *id,
                        })
                        .unwrap_or_else(|| DecodedReceipt::Regular(receipt.clone()))
                } else {
                    DecodedReceipt::Regular(receipt.clone())
                }
            })
            .collect()
    }

    /// Get the current session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

/// Decoded receipt for pretty printing
#[derive(Debug, Clone)]
pub enum DecodedReceipt {
    Regular(Receipt),
    LogData {
        receipt: Receipt,
        decoded_value: Option<String>,
        contract_id: ContractId,
    },
}
