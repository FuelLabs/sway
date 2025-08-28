use crate::{error::ArgumentError, ContractId};
use fuel_tx::Receipt;
use serde::{Deserialize, Serialize};

/// Commands representing all debug operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugCommand {
    /// Start a new transaction with optional ABI information
    StartTransaction {
        /// Path to the transaction JSON file
        tx_path: String,
        /// Optional ABI mappings - either a single ABI path for local dev
        /// or contract_id:abi_path pairs for contract-specific ABIs
        abi_mappings: Vec<AbiMapping>,
    },
    /// Reset the debugger state
    Reset,
    /// Continue execution until next breakpoint or termination
    Continue,
    /// Set single stepping mode
    SetSingleStepping {
        /// Whether to enable single stepping
        enable: bool,
    },
    /// Set a breakpoint at the specified location
    SetBreakpoint {
        /// Contract ID (zeroed for script breakpoints)
        contract_id: ContractId,
        /// Instruction offset
        offset: u64,
    },
    /// Get register value(s)
    GetRegisters {
        /// Optional specific register indices. If empty, returns all registers
        indices: Vec<u32>,
    },
    /// Get memory contents
    GetMemory {
        /// Starting offset in memory
        offset: u32,
        /// Number of bytes to read
        limit: u32,
    },
    /// Exit the debugger
    Quit,
}

/// ABI mapping for contract debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbiMapping {
    /// Local development ABI (no specific contract ID)
    Local { abi_path: String },
    /// Contract-specific ABI mapping
    Contract {
        contract_id: ContractId,
        abi_path: String,
    },
}

/// Response types for debug commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugResponse {
    /// Transaction started or continued with execution result
    RunResult {
        receipts: Vec<Receipt>,
        breakpoint: Option<BreakpointHit>,
    },
    /// Command completed successfully with no data
    Success,
    /// Register values
    Registers(Vec<RegisterValue>),
    /// Memory contents
    Memory(Vec<u8>),
    /// Error occurred
    Error(String),
}

/// Information about a breakpoint hit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointHit {
    pub contract: ContractId,
    pub pc: u64,
}

/// Register value with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterValue {
    pub index: u32,
    pub value: u64,
    pub name: String,
}

impl DebugCommand {
    /// Parse a command from CLI arguments
    pub fn from_cli_args(args: &[String]) -> Result<Self, ArgumentError> {
        if args.is_empty() {
            return Err(ArgumentError::NotEnough {
                expected: 1,
                got: 0,
            });
        }

        let cmd = &args[0];
        let args = &args[1..];

        match cmd.as_str() {
            "start_tx" | "n" | "tx" | "new_tx" => {
                Self::parse_start_tx(args).map_err(ArgumentError::Invalid)
            }
            "reset" => {
                if !args.is_empty() {
                    return Err(ArgumentError::Invalid(
                        "reset command takes no arguments".to_string(),
                    ));
                }
                Ok(DebugCommand::Reset)
            }
            "continue" | "c" => {
                if !args.is_empty() {
                    return Err(ArgumentError::Invalid(
                        "continue command takes no arguments".to_string(),
                    ));
                }
                Ok(DebugCommand::Continue)
            }
            "step" | "s" => Self::parse_step(args).map_err(ArgumentError::Invalid),
            "breakpoint" | "bp" | "b" => {
                Self::parse_breakpoint(args).map_err(ArgumentError::Invalid)
            }
            "register" | "r" | "reg" | "registers" => {
                Self::parse_registers(args).map_err(ArgumentError::Invalid)
            }
            "memory" | "m" | "mem" => Self::parse_memory(args).map_err(ArgumentError::Invalid),
            "quit" | "q" | "exit" => Ok(DebugCommand::Quit),
            _ => Err(ArgumentError::UnknownCommand(cmd.to_string())),
        }
    }

    /// Parse a start_tx command from CLI arguments
    ///
    /// Handles two distinct modes of operation:
    /// 1. Local Development: `tx transaction.json abi.json`
    /// 2. Contract-specific: `tx transaction.json --abi <contract_id>:<abi_file.json>`
    fn parse_start_tx(args: &[String]) -> Result<Self, String> {
        if args.is_empty() {
            return Err("Transaction file path required".to_string());
        }

        let tx_path = args[0].clone();
        let mut abi_mappings = Vec::new();
        let mut i = 1;

        while i < args.len() {
            if args[i] == "--abi" {
                if i + 1 >= args.len() {
                    return Err("Missing argument for --abi".to_string());
                }
                let abi_arg = &args[i + 1];
                if let Some((contract_id, abi_path)) = abi_arg.split_once(':') {
                    let contract_id = contract_id
                        .parse::<ContractId>()
                        .map_err(|_| format!("Invalid contract ID: {contract_id}"))?;
                    abi_mappings.push(AbiMapping::Contract {
                        contract_id,
                        abi_path: abi_path.to_string(),
                    });
                } else {
                    return Err(format!("Invalid --abi argument: {abi_arg}"));
                }
                i += 2;
            } else if args[i].ends_with(".json") {
                // Local development ABI
                abi_mappings.push(AbiMapping::Local {
                    abi_path: args[i].clone(),
                });
                i += 1;
            } else {
                return Err(format!("Unexpected argument: {}", args[i]));
            }
        }

        Ok(DebugCommand::StartTransaction {
            tx_path,
            abi_mappings,
        })
    }

    fn parse_step(args: &[String]) -> Result<Self, String> {
        let enable = args
            .first()
            .is_none_or(|v| !["off", "no", "disable"].contains(&v.as_str()));

        Ok(DebugCommand::SetSingleStepping { enable })
    }

    fn parse_breakpoint(args: &[String]) -> Result<Self, String> {
        if args.is_empty() {
            return Err("Breakpoint offset required".to_string());
        }

        let (contract_id, offset_str) = if args.len() == 2 {
            // Contract ID provided
            let contract_id = args[0]
                .parse::<ContractId>()
                .map_err(|_| format!("Invalid contract ID: {}", args[0]))?;
            (contract_id, &args[1])
        } else {
            // No contract ID, use zeroed
            (ContractId::zeroed(), &args[0])
        };

        let offset = crate::cli::parse_int(offset_str)
            .ok_or_else(|| format!("Invalid offset: {offset_str}"))? as u64;

        Ok(DebugCommand::SetBreakpoint {
            contract_id,
            offset,
        })
    }

    fn parse_registers(args: &[String]) -> Result<Self, String> {
        let mut indices = Vec::new();
        for arg in args {
            if let Some(v) = crate::cli::parse_int(arg) {
                indices.push(v as u32);
            } else if let Some(index) = crate::names::register_index(arg) {
                indices.push(index as u32);
            } else {
                return Err(format!("Unknown register: {arg}"));
            }
        }
        Ok(DebugCommand::GetRegisters { indices })
    }

    fn parse_memory(args: &[String]) -> Result<Self, String> {
        use fuel_vm::consts::{VM_MAX_RAM, WORD_SIZE};

        let offset = args
            .first()
            .map(|a| crate::cli::parse_int(a).ok_or_else(|| format!("Invalid offset: {a}")))
            .transpose()?
            .unwrap_or(0) as u32;

        let limit = args
            .get(1)
            .map(|a| crate::cli::parse_int(a).ok_or_else(|| format!("Invalid limit: {a}")))
            .transpose()?
            .unwrap_or(WORD_SIZE * (VM_MAX_RAM as usize)) as u32;

        Ok(DebugCommand::GetMemory { offset, limit })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_tx_command() {
        let args = vec!["start_tx".to_string(), "test.json".to_string()];
        let result = DebugCommand::from_cli_args(&args).unwrap();

        assert!(matches!(
            result,
            DebugCommand::StartTransaction { ref tx_path, ref abi_mappings }
            if tx_path == "test.json" && abi_mappings.is_empty()
        ));

        // Test alias
        let args = vec!["n".to_string(), "test.json".to_string()];
        let result = DebugCommand::from_cli_args(&args).unwrap();
        assert!(matches!(result, DebugCommand::StartTransaction { .. }));
    }

    #[test]
    fn test_reset_command() {
        let args = vec!["reset".to_string()];
        let result = DebugCommand::from_cli_args(&args).unwrap();
        assert!(matches!(result, DebugCommand::Reset));
    }

    #[test]
    fn test_continue_command() {
        let args = vec!["continue".to_string()];
        let result = DebugCommand::from_cli_args(&args).unwrap();
        assert!(matches!(result, DebugCommand::Continue));

        // Test alias
        let args = vec!["c".to_string()];
        let result = DebugCommand::from_cli_args(&args).unwrap();
        assert!(matches!(result, DebugCommand::Continue));
    }

    #[test]
    fn test_step_command() {
        let args = vec!["step".to_string()];
        let result = DebugCommand::from_cli_args(&args).unwrap();
        assert!(matches!(
            result,
            DebugCommand::SetSingleStepping { enable: true }
        ));

        let args = vec!["step".to_string(), "off".to_string()];
        let result = DebugCommand::from_cli_args(&args).unwrap();
        assert!(matches!(
            result,
            DebugCommand::SetSingleStepping { enable: false }
        ));

        // Test alias
        let args = vec!["s".to_string()];
        let result = DebugCommand::from_cli_args(&args).unwrap();
        assert!(matches!(
            result,
            DebugCommand::SetSingleStepping { enable: true }
        ));
    }

    #[test]
    fn test_breakpoint_command() {
        let args = vec!["breakpoint".to_string(), "100".to_string()];
        let result = DebugCommand::from_cli_args(&args).unwrap();
        assert!(matches!(
            result,
            DebugCommand::SetBreakpoint { contract_id, offset: 100 }
            if contract_id == ContractId::zeroed()
        ));

        // Test alias
        let args = vec!["bp".to_string(), "50".to_string()];
        let result = DebugCommand::from_cli_args(&args).unwrap();
        assert!(matches!(
            result,
            DebugCommand::SetBreakpoint { offset: 50, .. }
        ));
    }

    #[test]
    fn test_register_command() {
        let args = vec!["register".to_string()];
        let result = DebugCommand::from_cli_args(&args).unwrap();
        assert!(matches!(
            result,
            DebugCommand::GetRegisters { ref indices }
            if indices.is_empty()
        ));

        let args = vec!["reg".to_string(), "0".to_string()];
        let result = DebugCommand::from_cli_args(&args).unwrap();
        assert!(matches!(
            result,
            DebugCommand::GetRegisters { ref indices }
            if indices == &vec![0]
        ));
    }

    #[test]
    fn test_memory_command() {
        let args = vec!["memory".to_string()];
        let result = DebugCommand::from_cli_args(&args).unwrap();
        assert!(matches!(
            result,
            DebugCommand::GetMemory {
                offset: 0,
                limit: _
            }
        ));

        let args = vec!["memory".to_string(), "100".to_string(), "200".to_string()];
        let result = DebugCommand::from_cli_args(&args).unwrap();
        assert!(matches!(
            result,
            DebugCommand::GetMemory {
                offset: 100,
                limit: 200
            }
        ));

        // Test alias
        let args = vec!["m".to_string(), "50".to_string()];
        let result = DebugCommand::from_cli_args(&args).unwrap();
        assert!(matches!(result, DebugCommand::GetMemory { offset: 50, .. }));
    }

    #[test]
    fn test_quit_command() {
        let args = vec!["quit".to_string()];
        let result = DebugCommand::from_cli_args(&args).unwrap();
        assert!(matches!(result, DebugCommand::Quit));

        // Test aliases
        let args = vec!["q".to_string()];
        let result = DebugCommand::from_cli_args(&args).unwrap();
        assert!(matches!(result, DebugCommand::Quit));
    }

    #[test]
    fn test_error_cases() {
        // Empty args
        let args = vec![];
        let result = DebugCommand::from_cli_args(&args);
        assert!(matches!(
            result,
            Err(ArgumentError::NotEnough {
                expected: 1,
                got: 0
            })
        ));

        // Unknown command
        let args = vec!["unknown".to_string()];
        let result = DebugCommand::from_cli_args(&args);
        assert!(matches!(result, Err(ArgumentError::UnknownCommand(_))));

        // Missing arguments
        let args = vec!["start_tx".to_string()];
        let result = DebugCommand::from_cli_args(&args);
        assert!(result.is_err());

        let args = vec!["breakpoint".to_string()];
        let result = DebugCommand::from_cli_args(&args);
        assert!(result.is_err());
    }
}
