use clap::{ArgAction, Command};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub long_help: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub subcommands: Vec<CommandInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub args: Vec<ArgInfo>,
}

impl CommandInfo {
    pub fn new(cmd: &Command) -> Self {
        CommandInfo {
            name: cmd.get_name().to_owned(),
            long_help: cmd.get_after_long_help().map(|s| s.to_string()),
            description: cmd.get_about().map(|s| s.to_string()),
            subcommands: Self::get_subcommands(cmd),
            args: Self::get_args(cmd),
        }
    }

    pub fn to_clap(&self) -> clap::App<'_> {
        let mut cmd = Command::new(self.name.as_str());
        if let Some(desc) = &self.description {
            cmd = cmd.about(desc.as_str());
        }
        if let Some(long_help) = &self.long_help {
            cmd = cmd.after_long_help(long_help.as_str());
        }
        for subcommand in &self.subcommands {
            cmd = cmd.subcommand(subcommand.to_clap());
        }
        for arg in &self.args {
            cmd = cmd.arg(arg.to_clap());
        }
        cmd
    }

    fn get_subcommands(cmd: &Command) -> Vec<CommandInfo> {
        cmd.get_subcommands()
            .map(|subcommand| CommandInfo::new(subcommand))
            .collect::<Vec<_>>()
    }

    fn arg_conflicts(cmd: &Command, arg: &clap::Arg) -> Vec<String> {
        let mut res = vec![];

        for conflict in cmd.get_arg_conflicts_with(arg) {
            if let Some(s) = conflict.get_short() {
                res.push(format!("-{}", s));
            }

            if let Some(l) = conflict.get_long() {
                res.push(format!("--{}", l));
            }
        }

        res
    }

    fn get_args(cmd: &Command) -> Vec<ArgInfo> {
        cmd.get_arguments()
            .map(|opt| ArgInfo {
                name: opt.get_name().to_string(),
                short: opt.get_short_and_visible_aliases(),
                aliases: opt
                    .get_long_and_visible_aliases()
                    .map(|c| c.iter().map(|x| x.to_string()).collect::<Vec<_>>())
                    .unwrap_or_default(),
                help: opt.get_help().map(|s| s.to_string()),
                long_help: opt.get_long_help().map(|s| s.to_string()),
                conflicts: Self::arg_conflicts(cmd, opt),
                is_repeatable: matches!(
                    opt.get_action(),
                    ArgAction::Set | ArgAction::Append | ArgAction::Count,
                ),
            })
            .collect::<Vec<_>>()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArgInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub short: Option<Vec<char>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub help: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub long_help: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub conflicts: Vec<String>,
    pub is_repeatable: bool,
}

impl ArgInfo {
    pub fn to_clap(&self) -> clap::Arg<'_> {
        let mut arg = clap::Arg::with_name(self.name.as_str());
        if let Some(short) = &self.short {
            arg = arg.short(short[0]);
        }
        if let Some(help) = &self.help {
            arg = arg.help(help.as_str());
        }
        if let Some(long_help) = &self.long_help {
            arg = arg.long_help(long_help.as_str());
        }
        if self.is_repeatable {
            arg = arg.multiple(true);
        }
        arg
    }
}

#[macro_export]
// Let the user format the help and parse it from that string into arguments to create the unit test
macro_rules! cli_examples {
    ($st:path { $( [ $($description:ident)* => $command:stmt ] )* }) => {
        forc_util::cli_examples! {
            {
                $crate::paste::paste! {
                    use clap::IntoApp;
                    Some($st::into_app())
                }
            } {
                $crate::paste::paste! {
                    use clap::Parser;
                    $st::try_parse_from
                }
            } {
                $( [ $($description)* => $command ] )*
            }
        }
    };
    ( $into_app:block $parser:block { $( [ $($description:ident)* => $command:stmt ] )* }) => {
        $crate::paste::paste! {
        #[cfg(test)]
        mod cli_parsing {
            $(
            #[test]
            fn [<$($description:lower _)*:snake example>] () {
                let cli_parser = $parser;
                let mut args = parse_args($command);
                if cli_parser(args.clone()).is_err() {
                    // Failed to parse, it maybe a plugin. To execute a plugin the first argument needs to be removed, `forc`.
                    args.remove(0);
                    cli_parser(args).expect("valid subcommand");
                }
            }

            )*

            #[cfg(test)]
            fn parse_args(input: &str) -> Vec<String> {
                let mut chars = input.chars().peekable().into_iter();
                let mut args = vec![];

                loop {
                    let character = if let Some(c) = chars.next() { c } else { break };

                    match character {
                        ' ' | '\\' | '\t' | '\n' => loop {
                            match chars.peek() {
                                Some(' ') | Some('\t') | Some('\n') => chars.next(),
                                _ => break,
                            };
                        },
                        '=' => {
                            args.push("=".to_string());
                        }
                        '"' | '\'' => {
                            let end_character = character;
                            let mut current_word = String::new();
                            loop {
                                match chars.peek() {
                                    Some(character) => {
                                        if *character == end_character {
                                            let _ = chars.next();
                                            args.push(current_word);
                                            break;
                                        } else if *character == '\\' {
                                            let _ = chars.next();
                                            if let Some(character) = chars.next() {
                                                current_word.push(character);
                                            }
                                        } else {
                                            current_word.push(*character);
                                            chars.next();
                                        }
                                    }
                                    None => {
                                        break;
                                    }
                                }
                            }
                        }
                        character => {
                            let mut current_word = character.to_string();
                            loop {
                                match chars.peek() {
                                    Some(' ') | Some('\t') | Some('\n') | Some('=') | Some('\'')
                                    | Some('"') | None => {
                                        args.push(current_word);
                                        break;
                                    }
                                    Some(character) => {
                                        current_word.push(*character);
                                        chars.next();
                                    }
                                }
                            }
                        }
                    }
                }

                args
            }

        }
        }

        mod cli_definition {
            /// Dump the CLI definition to the stdout
            pub(crate) fn dump() {
                std::env::set_var("CLI_DUMP_DEFINITION", "");

                if let Some(mut cmd) = $into_app {
                    forc_util::serde_json::to_writer_pretty(
                        std::io::stdout(),
                        &forc_util::cli::CommandInfo::new(&cmd)
                    ).unwrap();
                    std::process::exit(0);
                }
            }
        }

        /// Show the long help for the current app
        ///
        /// This function is being called automatically, so if CLI_DUMP_DEFINITION is set to 1, it
        /// will dump the definition of the CLI. Otherwise, it would have to be manually invoked by
        /// the developer
        fn help() -> &'static str {
            if std::env::var("CLI_DUMP_DEFINITION") == Ok("1".to_string()) {
                cli_definition::dump();
            }
            Box::leak(format!("{}\n{}", forc_util::ansi_term::Colour::Yellow.paint("EXAMPLES:"), examples()).into_boxed_str())
        }

        /// Returns the examples for the command
        pub fn examples() -> &'static str {
            Box::leak( [
            $(
            $crate::paste::paste! {
                format!("    # {}\n    {}\n\n", stringify!($($description)*), $command)
            },
            )*
            ].concat().into_boxed_str())
        }
    }
}
