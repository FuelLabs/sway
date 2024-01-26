#[macro_export]
// Let the user format the help and parse it from that string into arguments to create the unit test
macro_rules! cli_examples {
    ($( [ $($description:ident)* => $command:tt $args:expr $( => $output:expr )? ] )*) => {
            #[cfg(test)]
            mod cli_examples {
            use $crate::serial_test;
            $(
            $crate::paste::paste! {
                #[test]
                #[serial_test::serial]
                #[allow(unreachable_code)]
                fn [<$($description:lower _)*:snake example>] () {
                    let mut proc = std::process::Command::new("cargo");
                    proc.env("CLI_TEST", "true");
                    proc.arg("run");
                    proc.arg("--bin");
                    proc.arg(if stringify!($command) == "forc" {
                        "forc".to_owned()
                    } else {
                        format!("forc-{}", stringify!($command))
                    });
                    proc.arg("--");

                    super::parse_args($args).into_iter().for_each(|arg| {
                        proc.arg(arg);
                    });

                    let path = std::path::Path::new("tests");
                    if path.is_dir() {
                        // a tests folder exists, move the cwd of the process to
                        // be executed there. In that folder all files needed to
                        // run the cmd should be stored
                        proc.current_dir(path);
                    }
                    let output = proc.output().expect(stringify!($command));

                    $(
                        let expected_output = $crate::Regex::new($output).expect("valid regex");
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);

                        assert!(
                            expected_output.is_match(&stdout) ||
                            expected_output.is_match(&stderr),
                            "expected_output: {}\nStdOut:\n{}\nStdErr:\n{}\n",
                            expected_output,
                            stdout,
                            stderr,
                        );
                        return;
                    )?
                    // We don't know what to get or how to parse the output, all
                    // we care is to get a valid exit code
                    assert!(output.status.success(), "{}: {:?}", stringify!($($description)*), output);
                }
            }
            )*
        }

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

        fn help() -> &'static str {
            Box::leak(format!("EXAMPLES:\n{}", examples()).into_boxed_str())
        }

        pub fn examples() -> &'static str {
            Box::leak( [
            $(
            $crate::paste::paste! {
                if stringify!($command) == "forc" {
                    format!("  #{}\n  forc {}\n\n", stringify!($($description)*), $args )
                } else {
                    format!("  #{}\n  forc {} {}\n\n", stringify!($($description)*), stringify!($command), $args )
                }
            },
            )*
            ].concat().into_boxed_str())
        }
    }
}
