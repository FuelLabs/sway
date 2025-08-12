use crate::{cli::state::DebuggerHelper, error::Result};
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

    pub fn is_quit_command(&self, cmd: &str) -> bool {
        self.quit.name == cmd || self.quit.aliases.contains(&cmd)
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
