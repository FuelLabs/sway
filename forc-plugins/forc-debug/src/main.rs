use clap::Parser;
use forc_debug::{
    error::{self, ArgumentError, ForcDebugError},
    names::{register_index, register_name},
    server::DapServer,
    ContractId, FuelClient, RunResult, Transaction,
};
use forc_tracing::{init_tracing_subscriber, println_error, TracingSubscriberOptions};
use fuel_vm::consts::{VM_MAX_RAM, VM_REGISTER_COUNT, WORD_SIZE};
use shellfish::{Command as ShCommand, Shell};

#[derive(Parser, Debug)]
#[clap(name = "forc-debug", version)]
/// Forc plugin for the Sway DAP (Debug Adapter Protocol) implementation.
pub struct Opt {
    /// The URL of the Fuel Client GraphQL API
    #[clap(default_value = "http://127.0.0.1:4000/graphql")]
    pub api_url: String,
    /// Start the DAP server
    #[clap(short, long)]
    pub serve: bool,
}

#[tokio::main]
async fn main() {
    init_tracing_subscriber(TracingSubscriberOptions::default());
    let config = Opt::parse();

    if let Err(err) = run(&config).await {
        println_error(&format!("{err}"));
        std::process::exit(1);
    }
}

async fn run(config: &Opt) -> Result<(), Box<dyn std::error::Error>> {
    if config.serve {
        return DapServer::default().start();
    }

    let mut shell = Shell::new_async(
        State {
            client: FuelClient::new(&config.api_url)?,
            session_id: String::new(), // Placeholder
        },
        ">> ",
    );

    // Registers an async command by wrapping the handler function `$f`,
    // converting its error type into `Box<dyn std::error::Error>`, and
    // associating it with the provided command names.
    macro_rules! command {
        ($f:ident, $help:literal, $names:expr) => {
            for c in $names {
                shell.commands.insert(
                    c,
                    ShCommand::new_async($help.to_string(), |state, args| {
                        Box::pin(async move {
                            $f(state, args)
                                .await
                                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
                        })
                    }),
                );
            }
        };
    }

    command!(
        cmd_start_tx,
        "path/to/tx.json -- start a new transaction",
        ["n", "tx", "new_tx", "start_tx"]
    );
    command!(
        cmd_reset,
        "-- reset, removing breakpoints and other state",
        ["reset"]
    );
    command!(
        cmd_continue,
        "-- run until next breakpoint or termination",
        ["c", "continue"]
    );
    command!(
        cmd_step,
        "[on|off] -- turn single-stepping on or off",
        ["s", "step"]
    );
    command!(
        cmd_breakpoint,
        "[contract_id] offset -- set a breakpoint",
        ["b", "breakpoint"]
    );
    command!(
        cmd_registers,
        "[regname ...] -- dump registers",
        ["r", "reg", "register", "registers"]
    );
    command!(cmd_memory, "[offset] limit -- dump memory", ["m", "memory"]);

    let session_id = shell.state.client.start_session().await?;
    shell.state.session_id.clone_from(&session_id);
    shell.run_async().await?;
    shell.state.client.end_session(&session_id).await?;
    Ok(())
}

struct State {
    client: FuelClient,
    session_id: String,
}

async fn cmd_start_tx(state: &mut State, mut args: Vec<String>) -> error::Result<()> {
    args.remove(0); // Remove the command name
    ArgumentError::ensure_arg_count(&args, 1, 1)?; // Ensure exactly one argument

    let path_to_tx_json = args.pop().unwrap(); // Safe due to arg count check

    // Read and parse the transaction JSON
    let tx_json = std::fs::read(&path_to_tx_json).map_err(ForcDebugError::IoError)?;
    let tx: Transaction = serde_json::from_slice(&tx_json).map_err(ForcDebugError::JsonError)?;

    // Start the transaction
    let status = state
        .client
        .start_tx(&state.session_id, &tx)
        .await
        .map_err(|e| ForcDebugError::FuelClientError(e.to_string()))?;

    pretty_print_run_result(&status);
    Ok(())
}

async fn cmd_reset(state: &mut State, mut args: Vec<String>) -> error::Result<()> {
    args.remove(0); // Remove the command name
    ArgumentError::ensure_arg_count(&args, 0, 0)?; // Ensure no extra arguments

    // Reset the session
    state
        .client
        .reset(&state.session_id)
        .await
        .map_err(|e| ForcDebugError::FuelClientError(e.to_string()))?;

    Ok(())
}

async fn cmd_continue(state: &mut State, mut args: Vec<String>) -> error::Result<()> {
    args.remove(0); // Remove the command name
    ArgumentError::ensure_arg_count(&args, 0, 0)?; // Ensure no extra arguments

    // Continue the transaction
    let status = state
        .client
        .continue_tx(&state.session_id)
        .await
        .map_err(|e| ForcDebugError::FuelClientError(e.to_string()))?;

    pretty_print_run_result(&status);
    Ok(())
}

async fn cmd_step(state: &mut State, mut args: Vec<String>) -> error::Result<()> {
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
        .map_err(|e| ForcDebugError::FuelClientError(e.to_string()))?;

    Ok(())
}

async fn cmd_breakpoint(state: &mut State, mut args: Vec<String>) -> error::Result<()> {
    args.remove(0); // Remove command name
    ArgumentError::ensure_arg_count(&args, 1, 2)?;

    let offset_str = args.pop().unwrap(); // Safe due to arg count check
    let offset = parse_int(&offset_str).ok_or_else(|| ArgumentError::InvalidNumber(offset_str))?;

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
        .map_err(|e| ForcDebugError::FuelClientError(e.to_string()))?;

    Ok(())
}

async fn cmd_registers(state: &mut State, mut args: Vec<String>) -> error::Result<()> {
    args.remove(0); // Remove the command name

    if args.is_empty() {
        // Print all registers
        for r in 0..VM_REGISTER_COUNT {
            let value = state
                .client
                .register(&state.session_id, r as u32)
                .await
                .map_err(|e| ForcDebugError::FuelClientError(e.to_string()))?;
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
                        .map_err(|e| ForcDebugError::FuelClientError(e.to_string()))?;
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
                    .map_err(|e| ForcDebugError::FuelClientError(e.to_string()))?;
                println!("reg[{index:#02x}] = {value:<8} # {arg}");
            } else {
                return Err(ArgumentError::Invalid(format!("Unknown register name: {arg}")).into());
            }
        }
    }
    Ok(())
}

async fn cmd_memory(state: &mut State, mut args: Vec<String>) -> error::Result<()> {
    args.remove(0); // Remove the command name

    // Parse limit argument or use the default
    let limit = args
        .pop()
        .map(|a| parse_int(&a).ok_or_else(|| ArgumentError::InvalidNumber(a)))
        .transpose()?
        .unwrap_or(WORD_SIZE * (VM_MAX_RAM as usize));

    // Parse offset argument or use the default
    let offset = args
        .pop()
        .map(|a| parse_int(&a).ok_or_else(|| ArgumentError::InvalidNumber(a)))
        .transpose()?
        .unwrap_or(0);

    // Ensure no extra arguments
    ArgumentError::ensure_arg_count(&args, 0, 0)?;

    // Fetch memory from the client
    let mem = state
        .client
        .memory(&state.session_id, offset as u32, limit as u32)
        .await
        .map_err(|e| ForcDebugError::FuelClientError(e.to_string()))?;

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

/// Pretty-prints the result of a run, including receipts and breakpoint information.
///
/// Outputs each receipt in the `RunResult` and details about the breakpoint if present.
/// If the execution terminated without hitting a breakpoint, it prints "Terminated".
fn pretty_print_run_result(rr: &RunResult) {
    for receipt in rr.receipts() {
        println!("Receipt: {receipt:?}");
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
fn parse_int(s: &str) -> Option<usize> {
    let (s, radix) = s.strip_prefix("0x").map_or((s, 10), |s| (s, 16));
    usize::from_str_radix(&s.replace('_', ""), radix).ok()
}
