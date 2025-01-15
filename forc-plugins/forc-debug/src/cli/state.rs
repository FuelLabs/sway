use crate::{cli::commands::Commands, FuelClient};
use rustyline::{
    completion::Completer,
    highlight::{CmdKind, Highlighter},
    hint::Hinter,
    validate::{ValidationContext, ValidationResult, Validator},
    Context, Helper,
};
use serde_json::Value;
use std::{borrow::Cow, fs, path::Path};

pub struct State {
    pub client: FuelClient,
    pub session_id: String,
}

impl State {
    pub fn new(client: FuelClient) -> Self {
        Self {
            client,
            session_id: String::new(),
        }
    }
}

pub struct DebuggerHelper {
    pub commands: Commands,
}

impl DebuggerHelper {
    pub fn new() -> Self {
        Self {
            commands: Commands::new(),
        }
    }
}

impl Completer for DebuggerHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let words: Vec<&str> = line[..pos].split_whitespace().collect();
        let word_start = line[..pos].rfind(char::is_whitespace).map_or(0, |i| i + 1);
        let word_to_complete = &line[word_start..pos];

        // Transaction command context
        if let Some(first_word) = words.first() {
            if self.commands.is_tx_command(first_word) && line[..word_start].ends_with(' ') {
                return Ok((word_start, get_transaction_files(word_to_complete)));
            }

            // Register command context
            if self.commands.is_register_command(first_word) && line[..word_start].ends_with(' ') {
                let register_names = vec![
                    "zero", "one", "of", "pc", "ssp", "sp", "fp", "hp", "err", "ggas", "cgas",
                    "bal", "is", "ret", "retl", "flag",
                ];

                let matches: Vec<String> = register_names
                    .into_iter()
                    .filter(|name| name.starts_with(word_to_complete))
                    .map(String::from)
                    .collect();

                return Ok((word_start, matches));
            }
        }

        // Main command completion
        let matches: Vec<String> = self
            .commands
            .get_all_command_strings()
            .into_iter()
            .filter(|cmd| cmd.starts_with(word_to_complete))
            .map(String::from)
            .collect();

        Ok((word_start, matches))
    }
}

impl Hinter for DebuggerHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        let cmd = line[..pos].split_whitespace().next()?;
        let command = self.commands.find_command(cmd)?;

        if line[..pos].split_whitespace().count() == 1 {
            return Some(format!(" - {}", command.help));
        }

        if self.commands.is_help_command(cmd) {
            Some(" [command] - show help for a command".into())
        } else {
            None
        }
    }
}

impl Highlighter for DebuggerHelper {
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Borrowed(hint)
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        _completion: rustyline::CompletionType,
    ) -> Cow<'c, str> {
        Cow::Borrowed(candidate)
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Borrowed(line)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _kind: CmdKind) -> bool {
        true
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            // Using RGB values: 4, 234, 130 | fuel green :)
            Cow::Owned("\x1b[38;2;4;234;130m>>\x1b[0m ".to_owned())
        } else {
            Cow::Borrowed(prompt)
        }
    }
}

impl Validator for DebuggerHelper {
    fn validate(&self, _ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }
}

impl Helper for DebuggerHelper {}

/// Returns valid transaction JSON files from current directory and subdirectories.
/// Files must contain one of: Script, Create, Mint, Upgrade, Upload, or Blob keys.
fn get_transaction_files(current_word: &str) -> Vec<String> {
    fn is_valid_transaction_json(path: &Path) -> bool {
        // Read and parse the JSON file
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => return false,
        };
        let json: Value = match serde_json::from_str(&content) {
            Ok(json) => json,
            Err(_) => return false,
        };

        // Check if it's a valid transaction JSON
        if let Value::Object(obj) = json {
            // Check for transaction type
            let has_valid_type = obj.keys().any(|key| {
                matches!(
                    key.as_str(),
                    "Script" | "Create" | "Mint" | "Upgrade" | "Upload" | "Blob"
                )
            });
            return has_valid_type;
        }
        false
    }

    let mut matches = Vec::new();

    // Create the walker and iterate through entries
    let walker = walkdir::WalkDir::new(".").follow_links(true);
    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();

            // Check if it's a .json file and starts with the current word
            if let Some(filename) = path
                .to_string_lossy()
                .strip_prefix("./")
                .map(|f| f.to_string())
            {
                if filename.ends_with(".json")
                    && filename.starts_with(current_word)
                    && is_valid_transaction_json(path)
                {
                    matches.push(filename);
                }
            }
        }
    }
    matches
}
