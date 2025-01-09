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
                aliases: &["m"],
                help: "View memory",
            },
            quit: Command {
                name: "quit",
                aliases: &["exit"],
                help: "Exit debugger",
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
        ]
    }

    pub fn is_tx_command(&self, cmd: &str) -> bool {
        self.tx.name == cmd || self.tx.aliases.contains(&cmd)
    }

    pub fn is_register_command(&self, cmd: &str) -> bool {
        self.registers.name == cmd || self.registers.aliases.contains(&cmd)
    }

    pub fn is_memory_command(&self, cmd: &str) -> bool {
        self.memory.name == cmd || self.memory.aliases.contains(&cmd)
    }

    pub fn is_breakpoint_command(&self, cmd: &str) -> bool {
        self.breakpoint.name == cmd || self.breakpoint.aliases.contains(&cmd)
    }

    pub fn is_quit_command(&self, cmd: &str) -> bool {
        self.quit.name == cmd || self.quit.aliases.contains(&cmd)
    }

    pub fn is_reset_command(&self, cmd: &str) -> bool {
        self.reset.name == cmd || self.reset.aliases.contains(&cmd)
    }

    pub fn is_continue_command(&self, cmd: &str) -> bool {
        self.continue_.name == cmd || self.continue_.aliases.contains(&cmd)
    }

    pub fn is_step_command(&self, cmd: &str) -> bool {
        self.step.name == cmd || self.step.aliases.contains(&cmd)
    }

    pub fn get_all_command_strings(&self) -> HashSet<&'static str> {
        let mut commands = HashSet::new();
        for cmd in self.all_commands() {
            commands.insert(cmd.name);
            commands.extend(cmd.aliases);
        }
        commands
    }

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
