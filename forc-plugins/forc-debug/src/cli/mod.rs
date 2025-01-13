mod commands;
mod state;

use crate::{
    error::{Error, Result},
    FuelClient,
};
use rustyline::{CompletionType, Config, EditMode, Editor};
use state::{DebuggerHelper, State};
use std::path::PathBuf;

/// Start the CLI debug interface
pub async fn start_cli(api_url: &str) -> Result<()> {
    // Initialize editor with config
    let config = Config::builder()
        .auto_add_history(true)
        .history_ignore_space(true)
        .completion_type(CompletionType::Circular)
        .edit_mode(EditMode::Vi)
        .max_history_size(100)?
        .build();

    let mut editor = Editor::with_config(config)?;

    // Set up helper
    let helper = DebuggerHelper::new();
    editor.set_helper(Some(helper));

    // Load history from .forc/.debug/history
    let history_path = get_history_file_path()?;
    let _ = editor.load_history(&history_path);

    // Create state
    let client = FuelClient::new(api_url).map_err(|e| Error::FuelClientError(e.to_string()))?;
    let mut state = State::new(client);

    // Start session
    state.session_id = state
        .client
        .start_session()
        .await
        .map_err(|e| Error::FuelClientError(e.to_string()))?;

    println!("Welcome to the Sway Debugger! Type \"help\" for a list of commands.");

    // Main REPL loop
    loop {
        let readline = editor.readline(">> ");
        match readline {
            Ok(line) => {
                let args: Vec<String> = line.split_whitespace().map(String::from).collect();

                if args.is_empty() {
                    continue;
                }

                if let Some(helper) = editor.helper() {
                    match args[0].as_str() {
                        cmd if helper.commands.is_help_command(cmd) => {
                            if let Err(e) = commands::cmd_help(&helper, &args).await {
                                println!("Error: {}", e);
                            }
                        }
                        cmd if helper.commands.is_tx_command(cmd) => {
                            if let Err(e) = commands::cmd_start_tx(&mut state, args).await {
                                println!("Error: {}", e);
                            }
                        }
                        cmd if helper.commands.is_register_command(cmd) => {
                            if let Err(e) = commands::cmd_registers(&mut state, args).await {
                                println!("Error: {}", e);
                            }
                        }
                        cmd if helper.commands.is_breakpoint_command(cmd) => {
                            if let Err(e) = commands::cmd_breakpoint(&mut state, args).await {
                                println!("Error: {}", e);
                            }
                        }
                        cmd if helper.commands.is_memory_command(cmd) => {
                            if let Err(e) = commands::cmd_memory(&mut state, args).await {
                                println!("Error: {}", e);
                            }
                        }
                        cmd if helper.commands.is_quit_command(cmd) => {
                            break;
                        }
                        cmd if helper.commands.is_reset_command(cmd) => {
                            if let Err(e) = commands::cmd_reset(&mut state, args).await {
                                println!("Error: {}", e);
                            }
                        }
                        cmd if helper.commands.is_continue_command(cmd) => {
                            if let Err(e) = commands::cmd_continue(&mut state, args).await {
                                println!("Error: {}", e);
                            }
                        }
                        cmd if helper.commands.is_step_command(cmd) => {
                            if let Err(e) = commands::cmd_step(&mut state, args).await {
                                println!("Error: {}", e);
                            }
                        }
                        unknown_cmd => {
                            if let Some(suggestion) = helper.commands.find_closest(unknown_cmd) {
                                println!(
                                    "Unknown command: '{}'. Did you mean '{}'?",
                                    unknown_cmd, suggestion.name
                                );
                            } else {
                                println!("Unknown command: '{}'", unknown_cmd);
                            }
                        }
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }

    // Save history
    let _ = editor.save_history(&history_path);

    // End session
    state
        .client
        .end_session(&state.session_id)
        .await
        .map_err(|e| Error::FuelClientError(e.to_string()))?;

    Ok(())
}

fn get_history_file_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| {
        Error::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find home directory",
        ))
    })?;
    let debug_dir = home.join(".forc").join(".debug");
    std::fs::create_dir_all(&debug_dir).map_err(Error::IoError)?;
    Ok(debug_dir.join("history"))
}
