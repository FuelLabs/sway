use crate::FuelClient;
use rustyline::{
    completion::Completer,
    highlight::{CmdKind, Highlighter},
    hint::Hinter,
    validate::{ValidationContext, ValidationResult, Validator},
    Context, Helper,
};
use std::borrow::Cow;

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

#[derive(Default)]
pub struct DebuggerHelper;

impl Completer for DebuggerHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let commands = vec![
            "n",
            "tx",
            "new_tx",
            "start_tx",
            "reset",
            "c",
            "continue",
            "s",
            "step",
            "b",
            "breakpoint",
            "r",
            "reg",
            "register",
            "registers",
            "m",
            "memory",
            "quit",
            "exit",
        ];

        let word_start = line[..pos].rfind(char::is_whitespace).map_or(0, |i| i + 1);
        let word = &line[word_start..pos];

        let matches: Vec<String> = commands
            .into_iter()
            .filter(|cmd| cmd.starts_with(word))
            .map(String::from)
            .collect();

        Ok((word_start, matches))
    }
}

impl Hinter for DebuggerHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<Self::Hint> {
        None
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

    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        true
    }
}

impl Validator for DebuggerHelper {
    fn validate(&self, _ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }
}

impl Helper for DebuggerHelper {}
