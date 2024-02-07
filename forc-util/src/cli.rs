#[derive(Debug, serde::Deserialize)]
struct BuildMessage {
    reason: String,
    target: BuildTarget,
    executable: String,
}

#[derive(Debug, serde::Deserialize)]
struct BuildTarget {
    kind: Vec<String>,
}

static BUILD_PATH: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// Gives the current working directory for the each unit-test. Each unit-test have their own CWD,
/// this is done to let the unit tests run in parallel
pub fn get_cwd() -> String {
    format!("/tmp/forc-cli/build-{}", thread_id::get())
}

/// Builds the binaries from a rust project *once* to reuse the compiled binaries. The binaries
/// won't change until the process is restarted.
///
/// The compilation passes the special CLI_TEST env variable so the code may take a default beviour
/// when user input is required
pub fn build_project(bin_name: &str) -> String {
    let mut build_path = BUILD_PATH.lock().unwrap();

    if let Some(build_path) = build_path.as_ref() {
        format!("{}/{}", build_path, bin_name)
    } else {
        let new_build_path = std::process::Command::new("cargo")
            .env("CLI_TEST", "true")
            .args(["build", "--message-format=json"])
            .output()
            .and_then(|output| {
                let stdout_str = String::from_utf8_lossy(&output.stdout);
                let build_messages: Vec<_> = stdout_str
                    .lines()
                    .filter_map(|line| serde_json::from_str::<BuildMessage>(line).ok())
                    .collect();

                let binary_paths: Vec<_> = build_messages
                    .iter()
                    .filter(|message| {
                        message.reason == "compiler-artifact"
                            && message.target.kind.iter().any(|kind| kind == "bin")
                    })
                    .map(|message| message.executable.clone())
                    .collect();

                binary_paths
                    .first()
                    .cloned()
                    .map(|p| {
                        std::path::PathBuf::from(p)
                            .parent()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_owned()
                    })
                    .ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::Other, "Binary path not found")
                    })
            })
            .unwrap();

        *build_path = Some(new_build_path.clone());
        format!("{}/{}", new_build_path, bin_name)
    }
}

#[macro_export]
/// Run an arbitrary code block if running in the CLI_TEST environment
macro_rules! if_cli_test {
    ($($code:tt)*) => {
        if option_env!("CLI_TEST").is_some() {
            $($code)*
        }
    };
}

#[macro_export]
/// Let the user format the help and parse it from that string into arguments to create the unit
/// test.
///
/// The list of examples is a list of tuples where the first element is the description of the test
/// (in plain English) followed by the command to be executed and the arguments to be passed to the
/// command. Optionally, the expected output of the command can be passed as well. These examples
/// are part of the help message of the CLI.
///
/// Each example is also converted into a unit test. The test invokes the CLI command externally
/// (there is no `#[cfg(test)]` since the command is an external process and unaware of the test
/// context). The `option_env!("CLI_TEST").is_some()` expression can be used to detect if the
/// command is being executed from the CLI_TEST environment and take a different path (for instance
/// to mock a user given input response).
///
/// This macro also takes a list of examples and a setup block. The setup code block is executed
/// once *before* and it is responsible to set the state of the system to the initial state
/// that is expected for the CLI command to be executed.
macro_rules! cli_examples {
    ($( [ $($description:ident)* => $command:tt $args:expr $( => $output:expr )? ] )* $( setup { $($setup:tt)* } )?) => {
            #[cfg(test)]
            mod cli_examples {
            use $crate::serial_test;

            fn test_setup() {
                $(
                    {
                        $($setup)*
                    }
                )?
            }

            $(
            $crate::paste::paste! {
                #[test]
                #[allow(unreachable_code)]
                fn [<$($description:lower _)*:snake example>] () {
                    let bin = if stringify!($command) == "forc" {
                        "forc".to_owned()
                    } else {
                        format!("forc-{}", stringify!($command))
                    };

                    let tmp_dir = forc_util::cli::get_cwd();
                    let mut proc = std::process::Command::new(&forc_util::cli::build_project(&bin));
                    super::parse_args($args).into_iter().for_each(|arg| {
                        proc.arg(arg.replace("{path}", &tmp_dir));
                    });

                    let _ = std::fs::remove_dir_all(&tmp_dir);
                    std::fs::create_dir_all(&tmp_dir).unwrap();
                    test_setup();
                    proc.current_dir(&tmp_dir);
                    let output = proc.output();
                    let _ = std::fs::remove_dir_all(&tmp_dir);
                    let output = output.expect("failed to run command");

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
