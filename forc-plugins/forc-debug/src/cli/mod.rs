mod commands;
mod state;

use crate::{
    error::{Error, Result},
    FuelClient,
};
use forc_tracing::println_red_err;
use rustyline::{CompletionType, Config, EditMode, Editor};
use state::{DebuggerHelper, State};

/// Start the CLI debug interface
pub async fn start_cli(api_url: &str) -> Result<()> {
    // Initialize editor with config
    let mut editor = Editor::with_config(
        Config::builder()
            .auto_add_history(true)
            .history_ignore_space(true)
            .completion_type(CompletionType::Circular)
            .edit_mode(EditMode::Vi)
            .build(),
    )?;

    // Set up helper
    let helper = DebuggerHelper::new();
    editor.set_helper(Some(helper));

    // Load history
    let _ = editor.load_history("debug_history.txt");

    // Create state
    let client = FuelClient::new(api_url).map_err(|e| Error::FuelClientError(e.to_string()))?;
    let mut state = State::new(client);

    // Start session
    state.session_id = state
        .client
        .start_session()
        .await
        .map_err(|e| Error::FuelClientError(e.to_string()))?;

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
                        cmd if helper.commands.is_tx_command(cmd) => {
                            if let Err(e) = commands::cmd_start_tx(&mut state, args).await {
                                println_red_err(&format!("Error: {}", e));
                            }
                        },
                        cmd if helper.commands.is_register_command(cmd) => {
                            if let Err(e) = commands::cmd_registers(&mut state, args).await {
                                println_red_err(&format!("Error: {}", e));
                            }
                        },
                        cmd if helper.commands.is_breakpoint_command(cmd) => {
                            if let Err(e) = commands::cmd_breakpoint(&mut state, args).await {
                                println_red_err(&format!("Error: {}", e));
                            }
                        },
                        cmd if helper.commands.is_memory_command(cmd) => {
                            if let Err(e) = commands::cmd_memory(&mut state, args).await {
                                println_red_err(&format!("Error: {}", e));
                            }
                        },
                        cmd if helper.commands.is_quit_command(cmd) => {
                            break;
                        },
                        cmd if helper.commands.is_reset_command(cmd) => {
                            if let Err(e) = commands::cmd_reset(&mut state, args).await {
                                println_red_err(&format!("Error: {}", e));
                            }
                        },
                        cmd if helper.commands.is_continue_command(cmd) => {
                            if let Err(e) = commands::cmd_continue(&mut state, args).await {
                                println_red_err(&format!("Error: {}", e));
                            }
                        },
                        cmd if helper.commands.is_step_command(cmd) => {
                            if let Err(e) = commands::cmd_step(&mut state, args).await {
                                println_red_err(&format!("Error: {}", e));
                            }
                        },
                        unknown_cmd => {
                            if let Some(suggestion) = helper.commands.find_closest(unknown_cmd) {
                                println!("Unknown command: '{}'. Did you mean '{}'?", unknown_cmd, suggestion.name);
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
    if let Err(e) = editor.save_history("debug_history.txt") {
        println!("Failed to save history: {}", e);
    }

    // End session
    state
        .client
        .end_session(&state.session_id)
        .await
        .map_err(|e| Error::FuelClientError(e.to_string()))?;

    Ok(())
}
