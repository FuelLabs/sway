use crate::{
    cli::state::{DebuggerHelper, State},
    error::{ArgumentError, Error, Result},
    names::{register_index, register_name},
    ContractId, RunResult, Transaction,
};
use fuel_tx::Receipt;
use fuel_vm::consts::{VM_MAX_RAM, VM_REGISTER_COUNT, WORD_SIZE};
use std::collections::HashSet;
use strsim::levenshtein;

#[derive(Debug, Clone)]
pub struct Command {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub help: &'static str,
}

pub struct Commands {
    pub tx: Command,
    pub reset: Command,
    pub continue_: Command,
    pub step: Command,
    pub breakpoint: Command,
    pub registers: Command,
    pub memory: Command,
    pub quit: Command,
    pub help: Command,
}

impl Commands {
    pub const fn new() -> Self {
        Self {
            tx: Command {
                name: "start_tx",
                aliases: &["n", "tx", "new_tx"],
                help: "Start a new transaction",
            },
            reset: Command {
                name: "reset",
                aliases: &[],
                help: "Reset debugger state",
            },
            continue_: Command {
                name: "continue",
                aliases: &["c"],
                help: "Continue execution",
            },
            step: Command {
                name: "step",
                aliases: &["s"],
                help: "Step execution",
            },
            breakpoint: Command {
                name: "breakpoint",
                aliases: &["b"],
                help: "Set breakpoint",
            },
            registers: Command {
                name: "register",
                aliases: &["r", "reg", "registers"],
                help: "View registers",
            },
            memory: Command {
                name: "memory",
                aliases: &["m", "mem"],
                help: "View memory",
            },
            quit: Command {
                name: "quit",
                aliases: &["exit"],
                help: "Exit debugger",
            },
            help: Command {
                name: "help",
                aliases: &["h", "?"],
                help: "Show help for commands",
            },
        }
    }

    pub fn all_commands(&self) -> Vec<&Command> {
        vec![
            &self.tx,
            &self.reset,
            &self.continue_,
            &self.step,
            &self.breakpoint,
            &self.registers,
            &self.memory,
            &self.quit,
            &self.help,
        ]
    }

    pub fn is_tx_command(&self, cmd: &str) -> bool {
        self.tx.name == cmd || self.tx.aliases.contains(&cmd)
    }

    pub fn is_register_command(&self, cmd: &str) -> bool {
        self.registers.name == cmd || self.registers.aliases.contains(&cmd)
    }

    pub fn is_memory_command(&self, cmd: &str) -> bool {
        self.memory.name == cmd || self.memory.aliases.contains(&cmd)
    }

    pub fn is_breakpoint_command(&self, cmd: &str) -> bool {
        self.breakpoint.name == cmd || self.breakpoint.aliases.contains(&cmd)
    }

    pub fn is_quit_command(&self, cmd: &str) -> bool {
        self.quit.name == cmd || self.quit.aliases.contains(&cmd)
    }

    pub fn is_reset_command(&self, cmd: &str) -> bool {
        self.reset.name == cmd || self.reset.aliases.contains(&cmd)
    }

    pub fn is_continue_command(&self, cmd: &str) -> bool {
        self.continue_.name == cmd || self.continue_.aliases.contains(&cmd)
    }

    pub fn is_step_command(&self, cmd: &str) -> bool {
        self.step.name == cmd || self.step.aliases.contains(&cmd)
    }

    pub fn is_help_command(&self, cmd: &str) -> bool {
        self.help.name == cmd || self.help.aliases.contains(&cmd)
    }

    pub fn find_command(&self, name: &str) -> Option<&Command> {
        self.all_commands()
            .into_iter()
            .find(|cmd| cmd.name == name || cmd.aliases.contains(&name))
    }

    /// Returns a set of all valid command strings including aliases
    pub fn get_all_command_strings(&self) -> HashSet<&'static str> {
        let mut commands = HashSet::new();
        for cmd in self.all_commands() {
            commands.insert(cmd.name);
            commands.extend(cmd.aliases);
        }
        commands
    }

    /// Suggests a similar command
    pub fn find_closest(&self, unknown_cmd: &str) -> Option<&Command> {
        self.all_commands()
            .into_iter()
            .flat_map(|cmd| {
                std::iter::once((cmd, cmd.name))
                    .chain(cmd.aliases.iter().map(move |&alias| (cmd, alias)))
            })
            .map(|(cmd, name)| (cmd, levenshtein(unknown_cmd, name)))
            .filter(|&(_, distance)| distance <= 2)
            .min_by_key(|&(_, distance)| distance)
            .map(|(cmd, _)| cmd)
    }
}

pub async fn cmd_start_tx(state: &mut State, mut args: Vec<String>) -> Result<()> {
    args.remove(0); // Remove the command name
    ArgumentError::ensure_arg_count(&args, 1, 1)?; // Ensure exactly one argument

    let path_to_tx_json = args.pop().unwrap(); // Safe due to arg count check

    // Read and parse the transaction JSON
    let tx_json = std::fs::read(&path_to_tx_json).map_err(Error::IoError)?;
    let tx: Transaction = serde_json::from_slice(&tx_json).map_err(Error::JsonError)?;

    // Start the transaction
    let status = state
        .client
        .start_tx(&state.session_id, &tx)
        .await
        .map_err(|e| Error::FuelClientError(e.to_string()))?;

    pretty_print_run_result(&status);
    Ok(())
}

pub async fn cmd_reset(state: &mut State, mut args: Vec<String>) -> Result<()> {
    args.remove(0); // Remove the command name
    ArgumentError::ensure_arg_count(&args, 0, 0)?; // Ensure no extra arguments

    // Reset the session
    state
        .client
        .reset(&state.session_id)
        .await
        .map_err(|e| Error::FuelClientError(e.to_string()))?;

    Ok(())
}

pub async fn cmd_continue(state: &mut State, mut args: Vec<String>) -> Result<()> {
    args.remove(0); // Remove the command name
    ArgumentError::ensure_arg_count(&args, 0, 0)?; // Ensure no extra arguments

    // Continue the transaction
    let status = state
        .client
        .continue_tx(&state.session_id)
        .await
        .map_err(|e| Error::FuelClientError(e.to_string()))?;

    pretty_print_run_result(&status);
    Ok(())
}

pub async fn cmd_step(state: &mut State, mut args: Vec<String>) -> Result<()> {
    args.remove(0); // Remove the command name
    ArgumentError::ensure_arg_count(&args, 0, 1)?; // Ensure the argument count is at most 1

    // Determine whether to enable or disable single stepping
    let enable = args
        .first()
        .map_or(true, |v| !["off", "no", "disable"].contains(&v.as_str()));

    // Call the client
    state
        .client
        .set_single_stepping(&state.session_id, enable)
        .await
        .map_err(|e| Error::FuelClientError(e.to_string()))?;

    Ok(())
}

pub async fn cmd_breakpoint(state: &mut State, mut args: Vec<String>) -> Result<()> {
    args.remove(0); // Remove command name
    ArgumentError::ensure_arg_count(&args, 1, 2)?;

    let offset_str = args.pop().unwrap(); // Safe due to arg count check
    let offset = parse_int(&offset_str).ok_or(ArgumentError::InvalidNumber(offset_str))?;

    let contract = if let Some(contract_id) = args.pop() {
        contract_id
            .parse::<ContractId>()
            .map_err(|_| ArgumentError::Invalid(format!("Invalid contract ID: {}", contract_id)))?
    } else {
        ContractId::zeroed()
    };

    // Call client
    state
        .client
        .set_breakpoint(&state.session_id, contract, offset as u64)
        .await
        .map_err(|e| Error::FuelClientError(e.to_string()))?;

    Ok(())
}

pub async fn cmd_registers(state: &mut State, mut args: Vec<String>) -> Result<()> {
    args.remove(0); // Remove the command name

    if args.is_empty() {
        // Print all registers
        for r in 0..VM_REGISTER_COUNT {
            let value = state
                .client
                .register(&state.session_id, r as u32)
                .await
                .map_err(|e| Error::FuelClientError(e.to_string()))?;
            println!("reg[{:#x}] = {:<8} # {}", r, value, register_name(r));
        }
    } else {
        // Process specific registers provided as arguments
        for arg in &args {
            if let Some(v) = parse_int(arg) {
                if v < VM_REGISTER_COUNT {
                    let value = state
                        .client
                        .register(&state.session_id, v as u32)
                        .await
                        .map_err(|e| Error::FuelClientError(e.to_string()))?;
                    println!("reg[{:#02x}] = {:<8} # {}", v, value, register_name(v));
                } else {
                    return Err(ArgumentError::InvalidNumber(format!(
                        "Register index too large: {v}"
                    ))
                    .into());
                }
            } else if let Some(index) = register_index(arg) {
                let value = state
                    .client
                    .register(&state.session_id, index as u32)
                    .await
                    .map_err(|e| Error::FuelClientError(e.to_string()))?;
                println!("reg[{index:#02x}] = {value:<8} # {arg}");
            } else {
                return Err(ArgumentError::Invalid(format!("Unknown register name: {arg}")).into());
            }
        }
    }
    Ok(())
}

pub async fn cmd_memory(state: &mut State, mut args: Vec<String>) -> Result<()> {
    args.remove(0); // Remove the command name

    // Parse limit argument or use the default
    let limit = args
        .pop()
        .map(|a| parse_int(&a).ok_or(ArgumentError::InvalidNumber(a)))
        .transpose()?
        .unwrap_or(WORD_SIZE * (VM_MAX_RAM as usize));

    // Parse offset argument or use the default
    let offset = args
        .pop()
        .map(|a| parse_int(&a).ok_or(ArgumentError::InvalidNumber(a)))
        .transpose()?
        .unwrap_or(0);

    // Ensure the argument count is at most 2
    ArgumentError::ensure_arg_count(&args, 0, 2)?;

    // Fetch memory from the client
    let mem = state
        .client
        .memory(&state.session_id, offset as u32, limit as u32)
        .await
        .map_err(|e| Error::FuelClientError(e.to_string()))?;

    // Print memory contents
    for (i, chunk) in mem.chunks(WORD_SIZE).enumerate() {
        print!(" {:06x}:", offset + i * WORD_SIZE);
        for byte in chunk {
            print!(" {byte:02x}");
        }
        println!();
    }
    Ok(())
}

// Add help command implementation:
pub async fn cmd_help(helper: &DebuggerHelper, args: &[String]) -> Result<()> {
    if args.len() > 1 {
        // Help for specific command
        if let Some(cmd) = helper.commands.find_command(&args[1]) {
            println!("{} - {}", cmd.name, cmd.help);
            if !cmd.aliases.is_empty() {
                println!("Aliases: {}", cmd.aliases.join(", "));
            }
            return Ok(());
        }
        println!("Unknown command: '{}'", args[1]);
    }

    println!("Available commands:");
    for cmd in helper.commands.all_commands() {
        println!("  {:<12} - {}", cmd.name, cmd.help);
        if !cmd.aliases.is_empty() {
            println!("    aliases: {}", cmd.aliases.join(", "));
        }
    }
    Ok(())
}

/// Pretty-prints the result of a run, including receipts and breakpoint information.
///
/// Outputs each receipt in the `RunResult` and details about the breakpoint if present.
/// If the execution terminated without hitting a breakpoint, it prints "Terminated".
fn pretty_print_run_result(rr: &RunResult) {
    for receipt in rr.receipts() {
        println!("Receipt: {receipt:#?}");

        if let Receipt::LogData {
            rb,
            data: Some(data),
            ..
        } = receipt
        {
            let decoded_log_data = forc_test::decode_log_data(
                &rb.to_string(),
                &data,
                &rr.contract_abi.unwrap(),
            )
            .unwrap();
            println!(
                "Decoded log value: {}, log rb: {}",
                decoded_log_data.value, rb
            );
        }
    }
    if let Some(bp) = &rr.breakpoint {
        println!(
            "Stopped on breakpoint at address {} of contract {}",
            bp.pc.0, bp.contract
        );
    } else {
        println!("Terminated");
    }
}

/// Parses a string representing a number and returns it as a `usize`.
///
/// The input string can be in decimal or hexadecimal format:
/// - Decimal numbers are parsed normally (e.g., `"123"`).
/// - Hexadecimal numbers must be prefixed with `"0x"` (e.g., `"0x7B"`).
/// - Underscores can be used as visual separators (e.g., `"1_000"` or `"0x1_F4"`).
///
/// If the input string is not a valid number in the specified format, `None` is returned.
///
/// # Examples
///
/// ```
/// use forc_debug::cli::parse_int;
/// /// Use underscores as separators in decimal and hexadecimal numbers
/// assert_eq!(parse_int("123"), Some(123));
/// assert_eq!(parse_int("1_000"), Some(1000));
///
/// /// Parse hexadecimal numbers with "0x" prefix
/// assert_eq!(parse_int("0x7B"), Some(123));
/// assert_eq!(parse_int("0x1_F4"), Some(500));
///
/// /// Handle invalid inputs gracefully
/// assert_eq!(parse_int("abc"), None);
/// assert_eq!(parse_int("0xZZZ"), None);
/// assert_eq!(parse_int(""), None);
/// ```
///
/// # Errors
///
/// Returns `None` if the input string contains invalid characters,
/// is not properly formatted, or cannot be parsed into a `usize`.
pub fn parse_int(s: &str) -> Option<usize> {
    let (s, radix) = s.strip_prefix("0x").map_or((s, 10), |s| (s, 16));
    usize::from_str_radix(&s.replace('_', ""), radix).ok()
}
