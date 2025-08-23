#[macro_export]
// Let the user format the help and parse it from that string into arguments to create the unit test
macro_rules! cli_examples {
    ($st:path { $( [ $($description:ident)* => $command:stmt ] )* }) => {
        forc_util::cli_examples! {
            {
                $crate::pastey::paste! {
                    use clap::Parser;
                    $st::try_parse_from
                }
            } {
                $( [ $($description)* => $command ] )*
            }
        }
    };
    ( $code:block { $( [ $($description:ident)* => $command:stmt ] )* }) => {
        $crate::pastey::paste! {
        #[cfg(test)]
        mod cli_parsing {
            $(
            #[test]
            fn [<$($description:lower _)*:snake example>] () {

                let cli_parser = $code;
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


        fn help() -> &'static str {
            Box::leak(format!("{}\n{}", forc_util::ansiterm::Colour::Yellow.paint("EXAMPLES:"), examples()).into_boxed_str())
        }

        pub fn examples() -> &'static str {
            Box::leak( [
            $(
            $crate::pastey::paste! {
                format!("    # {}\n    {}\n\n", stringify!($($description)*), $command)
            },
            )*
            ].concat().into_boxed_str())
        }
    }
}
