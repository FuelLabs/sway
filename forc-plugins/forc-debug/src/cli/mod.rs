mod commands;
mod state;

pub use commands::parse_int;

use crate::{
    debugger::Debugger,
    error::{ArgumentError, Error, Result},
};
use rustyline::{CompletionType, Config, EditMode, Editor};
use state::DebuggerHelper;
use std::path::PathBuf;

/// Start the CLI debug interface
pub async fn start_cli(api_url: &str) -> Result<()> {
    let mut cli = Cli::new()?;
    let mut debugger = Debugger::new(api_url).await?;
    cli.run(&mut debugger, None).await
}

pub struct Cli {
    editor: Editor<DebuggerHelper, rustyline::history::FileHistory>,
    history_path: PathBuf,
}

impl Drop for Cli {
    fn drop(&mut self) {
        // Save the terminal history
        let _ = self.editor.save_history(&self.history_path);
    }
}

impl Cli {
    pub fn new() -> Result<Self> {
        // Initialize editor with config
        let config = Config::builder()
            .auto_add_history(true)
            .history_ignore_space(true)
            .completion_type(CompletionType::Circular)
            .edit_mode(EditMode::Vi)
            .max_history_size(100)?
            .build();

        let mut editor = Editor::with_config(config)?;
        let helper = DebuggerHelper::new();
        editor.set_helper(Some(helper));

        // Load history from .forc/.debug/history
        let history_path = get_history_file_path()?;
        let _ = editor.load_history(&history_path);

        Ok(Self {
            editor,
            history_path,
        })
    }

    /// Main CLI entry point with optional initial input
    pub async fn run(
        &mut self,
        debugger: &mut Debugger,
        initial_input: Option<String>,
    ) -> Result<()> {
        println!("Welcome to the Sway Debugger! Type \"help\" for a list of commands.");

        let mut prefill_next = initial_input;

        // Main REPL loop
        loop {
            let readline = if let Some(prefill) = prefill_next.take() {
                self.editor.readline_with_initial(">> ", (&prefill, ""))
            } else {
                self.editor.readline(">> ")
            };

            match readline {
                Ok(line) => {
                    let args: Vec<String> = line.split_whitespace().map(String::from).collect();
                    if args.is_empty() {
                        continue;
                    }

                    if let Some(helper) = self.editor.helper() {
                        match args[0].as_str() {
                            cmd if helper.commands.is_help_command(cmd) => {
                                if let Err(e) = commands::cmd_help(helper, &args).await {
                                    println!("Error: {}", e);
                                }
                            }
                            cmd if helper.commands.is_quit_command(cmd) => {
                                break Ok(());
                            }
                            _ => {
                                // Execute the command using debugger
                                if let Err(e) = debugger
                                    .execute_from_args(args.clone(), &mut std::io::stdout())
                                    .await
                                {
                                    if let Error::ArgumentError(ArgumentError::UnknownCommand(
                                        cmd,
                                    )) = &e
                                    {
                                        // Check if this is an unknown command error and provide suggestions
                                        if let Some(suggestion) = helper.commands.find_closest(cmd)
                                        {
                                            println!(
                                                "Unknown command: '{}'. Did you mean '{}'?",
                                                cmd, suggestion.name
                                            );
                                        } else {
                                            println!("Error: {}", e);
                                        }
                                    } else {
                                        println!("Error: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break Ok(());
                }
                Err(rustyline::error::ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break Ok(());
                }
                Err(err) => {
                    println!("Error: {}", err);
                    break Ok(());
                }
            }
        }
    }
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
