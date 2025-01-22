use crate::{
    cli::commands::Commands,
    error::{Error, Result},
    names, FuelClient,
};
use fuel_types::ContractId;
use rustyline::{
    completion::Completer,
    highlight::{CmdKind, Highlighter},
    hint::Hinter,
    validate::{ValidationContext, ValidationResult, Validator},
    Context, Helper,
};
use serde_json::Value;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fs,
};
use sway_core::asm_generation::ProgramABI;

pub struct State {
    pub client: FuelClient,
    pub session_id: String,
    pub contract_abis: HashMap<ContractId, ProgramABI>,
}

impl State {
    pub fn new(client: FuelClient) -> Self {
        Self {
            client,
            session_id: String::new(),
            contract_abis: HashMap::new(),
        }
    }

    /// Registers the given ABI for the given contract ID.
    pub fn register_abi(&mut self, contract_id: ContractId, abi: ProgramABI) {
        self.contract_abis.insert(contract_id, abi);
    }

    /// Either fetches the ABI from the Sway ABI Registry or returns it from the cache if it's already known.
    pub fn get_or_fetch_abi(&mut self, contract_id: &ContractId) -> Option<&ProgramABI> {
        // If we already have it, return it
        if self.contract_abis.contains_key(contract_id) {
            return self.contract_abis.get(contract_id);
        }

        // Try to fetch from ABI Registry
        match fetch_abi_from_api(contract_id) {
            Ok(abi) => {
                self.register_abi(*contract_id, abi);
                self.contract_abis.get(contract_id)
            }
            Err(_) => None,
        }
    }
}

/// Fetches the ABI for the given contract ID from the Sway ABI Registry.
fn fetch_abi_from_api(_contract_id: &ContractId) -> Result<ProgramABI> {
    // TODO: Implement this once the Sway ABI Registry is available
    Err(Error::AbiError("Not implemented yet".to_string()))
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
            if self.commands.is_tx_command(first_word) {
                match words.len() {
                    1 => {
                        // First argument is transaction file
                        return Ok((word_start, get_transaction_files(word_to_complete)));
                    }
                    2 => {
                        // Second argument could be either:
                        // 1. a local ABI file (ends in .json)
                        // 2. the --abi flag
                        if word_to_complete.is_empty() || word_to_complete.starts_with('-') {
                            return Ok((word_start, vec!["--abi".to_string()]));
                        } else {
                            return Ok((word_start, get_abi_files(word_to_complete)));
                        }
                    }
                    _ => {
                        // If previous word was --abi, we expect contract_id:abi.json
                        if words[words.len() - 2] == "--abi" {
                            // Here we could potentially provide completion for known contract IDs
                            // followed by a colon and the ABI file
                            let abi_files = get_abi_files("");
                            let completions: Vec<String> = abi_files
                                .into_iter()
                                .map(|abi| format!("contract_id:{}", abi))
                                .collect();
                            return Ok((word_start, completions));
                        }
                    }
                }
            }

            // Register command context
            if self.commands.is_register_command(first_word) && line[..word_start].ends_with(' ') {
                let matches: Vec<String> = names::REGISTERS
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

/// Get valid ABI files matching the current word
fn get_abi_files(current_word: &str) -> Vec<String> {
    find_valid_json_files(current_word, is_valid_abi)
}

/// Returns valid transaction JSON files from current directory and subdirectories.
/// Files must contain one of: Script, Create, Mint, Upgrade, Upload, or Blob keys.
fn get_transaction_files(current_word: &str) -> Vec<String> {
    find_valid_json_files(current_word, is_valid_transaction)
}

/// Generic function to find and validate JSON files
fn find_valid_json_files<F>(current_word: &str, is_valid: F) -> Vec<String>
where
    F: Fn(&Value) -> bool,
{
    let mut matches = Vec::new();
    let walker = walkdir::WalkDir::new(".").follow_links(true);

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(filename) = entry
                .path()
                .to_string_lossy()
                .strip_prefix("./")
                .map(|f| f.to_string())
            {
                if filename.ends_with(".json") && filename.starts_with(current_word) {
                    // Try to read and parse the file
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if let Ok(json) = serde_json::from_str::<Value>(&content) {
                            if is_valid(&json) {
                                matches.push(filename);
                            }
                        }
                    }
                }
            }
        }
    }
    matches
}

/// Checks if a JSON value represents a valid transaction
fn is_valid_transaction(json: &Value) -> bool {
    if let Value::Object(obj) = json {
        // Check for transaction type
        obj.keys().any(|key| {
            matches!(
                key.as_str(),
                "Script" | "Create" | "Mint" | "Upgrade" | "Upload" | "Blob"
            )
        })
    } else {
        false
    }
}

/// Checks if a JSON value represents a valid ABI
fn is_valid_abi(json: &Value) -> bool {
    if let Value::Object(obj) = json {
        // Required fields for an ABI
        let required_fields: HashSet<_> = [
            "programType",
            "functions",
            "concreteTypes",
            "encodingVersion",
        ]
        .iter()
        .collect();

        // Check that all required fields exist and have the correct type
        if !required_fields
            .iter()
            .all(|&field| obj.contains_key(*field))
        {
            return false;
        }

        // Validate functions array
        if let Some(Value::Array(functions)) = obj.get("functions") {
            // Every function should have a name and inputs field
            functions.iter().all(|f| {
                matches!(f, Value::Object(f_obj) if f_obj.contains_key("name") && f_obj.contains_key("inputs"))
            })
        } else {
            false
        }
    } else {
        false
    }
}
