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
        let main_commands = vec![
            "n", "tx", "new_tx", "start_tx",
            "reset",
            "c", "continue",
            "s", "step",
            "b", "breakpoint",
            "r", "reg", "register", "registers",
            "m", "memory",
            "quit", "exit",
        ];

        let register_names = vec![
            "zero", "one", "of", "pc", "ssp", "sp", "fp", "hp", 
            "err", "ggas", "cgas", "bal", "is", "ret", "retl", "flag"
        ];

        let words: Vec<&str> = line[..pos].split_whitespace().collect();
        let word_start = line[..pos].rfind(char::is_whitespace).map_or(0, |i| i + 1);
        let word_to_complete = &line[word_start..pos];

        // If we're in a register command context AND there's a space after the command
        if words.get(0).map_or(false, |&cmd| ["r", "reg", "register", "registers"].contains(&cmd)) 
            && line[..word_start].ends_with(' ') 
        {
            let matches: Vec<String> = register_names.into_iter()
                .filter(|name| name.starts_with(word_to_complete))
                .map(String::from)
                .collect();

            return Ok((word_start, matches));
        }
        
        // For all other cases, suggest main commands
        let matches: Vec<String> = main_commands.into_iter()
            .filter(|cmd| cmd.starts_with(word_to_complete))
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
