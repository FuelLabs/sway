use clap::Parser;
use forc_debug::{
    names::{register_index, register_name},
    server::DapServer,
    ContractId, FuelClient, RunResult, Transaction,
};
use fuel_vm::consts::{VM_MAX_RAM, VM_REGISTER_COUNT, WORD_SIZE};
use shellfish::{async_fn, Command as ShCommand, Shell};
use std::error::Error;

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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Opt::parse();

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

    macro_rules! command {
        ($f:ident, $help:literal, $names:expr) => {
            for c in $names {
                shell.commands.insert(
                    c,
                    ShCommand::new_async($help.to_string(), async_fn!(State, $f)),
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

#[derive(Debug, thiserror::Error)]
enum ArgError {
    #[error("Invalid argument")]
    Invalid,
    #[error("Not enough arguments")]
    NotEnough,
    #[error("Too many arguments")]
    TooMany,
}

fn pretty_print_run_result(rr: &RunResult) {
    for receipt in rr.receipts() {
        println!("Receipt: {:?}", receipt);
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

async fn cmd_start_tx(state: &mut State, mut args: Vec<String>) -> Result<(), Box<dyn Error>> {
    args.remove(0);
    let path_to_tx_json = args.pop().ok_or_else(|| Box::new(ArgError::NotEnough))?;
    if !args.is_empty() {
        return Err(Box::new(ArgError::TooMany));
    }

    let tx_json = std::fs::read(path_to_tx_json)?;
    let tx: Transaction = serde_json::from_slice(&tx_json).unwrap();
    let status = state.client.start_tx(&state.session_id, &tx).await?;
    pretty_print_run_result(&status);

    Ok(())
}

async fn cmd_reset(state: &mut State, mut args: Vec<String>) -> Result<(), Box<dyn Error>> {
    args.remove(0);
    if !args.is_empty() {
        return Err(Box::new(ArgError::TooMany));
    }

    let _ = state.client.reset(&state.session_id).await?;

    Ok(())
}

async fn cmd_continue(state: &mut State, mut args: Vec<String>) -> Result<(), Box<dyn Error>> {
    args.remove(0);
    if !args.is_empty() {
        return Err(Box::new(ArgError::TooMany));
    }

    let status = state.client.continue_tx(&state.session_id).await?;
    pretty_print_run_result(&status);

    Ok(())
}

async fn cmd_step(state: &mut State, mut args: Vec<String>) -> Result<(), Box<dyn Error>> {
    args.remove(0);
    if args.len() > 1 {
        return Err(Box::new(ArgError::TooMany));
    }

    state
        .client
        .set_single_stepping(
            &state.session_id,
            args.first()
                .map(|v| !["off", "no", "disable"].contains(&v.as_str()))
                .unwrap_or(true),
        )
        .await?;
    Ok(())
}

async fn cmd_breakpoint(state: &mut State, mut args: Vec<String>) -> Result<(), Box<dyn Error>> {
    args.remove(0);
    let offset = args.pop().ok_or_else(|| Box::new(ArgError::NotEnough))?;
    let contract_id = args.pop();

    if !args.is_empty() {
        return Err(Box::new(ArgError::TooMany));
    }

    let offset = if let Some(offset) = parse_int(&offset) {
        offset as u64
    } else {
        return Err(Box::new(ArgError::Invalid));
    };

    let contract = if let Some(contract_id) = contract_id {
        if let Ok(contract_id) = contract_id.parse::<ContractId>() {
            contract_id
        } else {
            return Err(Box::new(ArgError::Invalid));
        }
    } else {
        ContractId::zeroed() // Current script
    };

    state
        .client
        .set_breakpoint(&state.session_id, contract, offset)
        .await?;

    Ok(())
}

async fn cmd_registers(state: &mut State, mut args: Vec<String>) -> Result<(), Box<dyn Error>> {
    args.remove(0);

    if args.is_empty() {
        for r in 0..VM_REGISTER_COUNT {
            let value = state.client.register(&state.session_id, r as u32).await?;
            println!("reg[{:#x}] = {:<8} # {}", r, value, register_name(r));
        }
    } else {
        for arg in &args {
            if let Some(v) = parse_int(arg) {
                if v < VM_REGISTER_COUNT {
                    let value = state.client.register(&state.session_id, v as u32).await?;
                    println!("reg[{:#02x}] = {:<8} # {}", v, value, register_name(v));
                } else {
                    println!("Register index too large {}", v);
                    return Ok(());
                }
            } else if let Some(index) = register_index(arg) {
                let value = state
                    .client
                    .register(&state.session_id, index as u32)
                    .await?;
                println!("reg[{:#02x}] = {:<8} # {}", index, value, arg);
            } else {
                println!("Unknown register name {}", arg);
                return Ok(());
            }
        }
    }

    Ok(())
}

async fn cmd_memory(state: &mut State, mut args: Vec<String>) -> Result<(), Box<dyn Error>> {
    args.remove(0);

    let limit = args
        .pop()
        .map(|a| parse_int(&a).ok_or(ArgError::Invalid))
        .transpose()?
        .unwrap_or(WORD_SIZE * (VM_MAX_RAM as usize));

    let offset = args
        .pop()
        .map(|a| parse_int(&a).ok_or(ArgError::Invalid))
        .transpose()?
        .unwrap_or(0);

    if !args.is_empty() {
        return Err(Box::new(ArgError::TooMany));
    }

    let mem = state
        .client
        .memory(&state.session_id, offset as u32, limit as u32)
        .await?;

    for (i, chunk) in mem.chunks(WORD_SIZE).enumerate() {
        print!(" {:06x}:", offset + i * WORD_SIZE);
        for byte in chunk {
            print!(" {:02x}", byte);
        }
        println!();
    }

    Ok(())
}

fn parse_int(s: &str) -> Option<usize> {
    let (s, radix) = if let Some(stripped) = s.strip_prefix("0x") {
        (stripped, 16)
    } else {
        (s, 10)
    };

    let s = s.replace('_', "");

    usize::from_str_radix(&s, radix).ok()
}
