#[macro_export]
// Let the user format the help and parse it from that string into arguments to create the unit test
macro_rules! cli_examples_v2 {
    ($( [ $($description:ident)* => $command:tt $args:expr ] )*) => {
            #[cfg(test)]
            use $crate::serial_test;
            $(
            $crate::paste::paste! {
                #[cfg(test)]
                #[test]
                #[serial_test::serial]
                fn [<$($description:lower _)*:snake example>] () {
                    let mut proc = std::process::Command::new("cargo");
                    proc.arg("run");
                    proc.arg("--bin");
                    proc.arg(format!("forc-{}", stringify!($command)));
                    proc.arg("--");

                    parse_args($args).into_iter().for_each(|arg| {
                        proc.arg(arg);
                    });

                    let path = std::path::Path::new("tests");
                    if path.is_dir() {
                        proc.current_dir(path);
                    }
                    let output = proc.output().expect(stringify!($command));
                    assert!(output.status.success(), "{}: {:?}", stringify!($($description)*), output);
                }
            }
            )*

        #[cfg(test)]
        fn parse_args(input: &str) -> Vec<String> {
            let mut chars = input.chars().peekable().into_iter();
            let mut args = vec![];

            loop {
                let c = if let Some(c) = chars.next() { c } else { break };

                match c {
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
                        let end_character = c;
                        let mut current_word = String::new();
                        loop {
                            match chars.peek() {
                                Some(c) => {
                                    if *c == end_character {
                                        let _ = chars.next();
                                        args.push(current_word);
                                        break;
                                    } else if *c == '\\' {
                                        let _ = chars.next();
                                        if let Some(c) = chars.next() {
                                            current_word.push(c);
                                        }
                                    } else {
                                        current_word.push(*c);
                                        chars.next();
                                    }
                                }
                                None => {
                                    break;
                                }
                            }
                        }
                    }
                    c => {
                        let mut current_word = c.to_string();
                        loop {
                            match chars.peek() {
                                Some(' ') | Some('\t') | Some('\n') | Some('=') | Some('\'')
                                | Some('"') | None => {
                                    args.push(current_word);
                                    break;
                                }
                                Some(c) => {
                                    current_word.push(*c);
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
                    format!("  #{}\n  forc {} {}\n\n", stringify!($($description)*), stringify!($command), $args )
            },
            )*
            ].concat().into_boxed_str())
        }
    }
}

#[macro_export]
macro_rules! cli_examples {
    ($( [ $($description:ident)* => $command:tt $($arg:expr)* ] )*) => {
            #[cfg(test)]
            use $crate::serial_test;
            $(
            $crate::paste::paste! {
                #[cfg(test)]
                #[test]
                #[serial_test::serial]
                fn [<$($description:lower _)*:snake example>] () {
                    let mut proc = std::process::Command::new("cargo");
                    proc.arg("run");
                    proc.arg("--bin");
                    proc.arg(format!("forc-{}", stringify!($command)));
                    proc.arg("--");
                    $(
                        proc.arg($arg);
                    )*

                    let path = std::path::Path::new("tests");
                    if path.is_dir() {
                        proc.current_dir(path);
                    }
                    let output = proc.output().expect(stringify!($command));
                    assert!(output.status.success(), "{}: {:?}", stringify!($($description)*), output);
                }
            }
            )*

        fn help() -> &'static str {
            Box::leak(format!("EXAMPLES:\n{}", examples()).into_boxed_str())
        }

        fn print_args(args: Vec<String>) -> String {
            let mut result = String::new();
            let mut iter = args.iter().peekable();
            let mut length = 0;
            let mut is_multiline = false;
            let equal = "=".to_owned();
            loop {
                let arg = if let Some(arg) = iter.next() {
                    arg
                } else {
                    break;
                };
                if length + arg.len() > 70 && iter.peek().is_some() && arg.chars().next() != Some('-') {
                    // too long, break it into a new line
                    result.push_str("\\\n     ");
                    is_multiline = true;
                    length = 5;
                }
                if is_multiline && arg.chars().next() == Some('-') {
                    // it a multiline arg and the next arg is a flag, put them in their own line
                    result.push_str("\\\n     ");
                    length = 5;
                }
                result.push_str(arg);
                length += arg.len();
                if arg != &equal && iter.peek() != Some(&&equal) {
                    result.push(' ');
                    length += 1;
                }
            }
            result
        }

        fn quote_str(s: &str) -> String {
            let mut result = String::with_capacity(s.len() + 2); // Initial capacity with room for quotes

            result.push('"');
            for c in s.chars() {
                match c {
                    '\\' | '"' => result.push('\\'), // Escape backslashes and quotes
                    _ => (),
                }
                result.push(c);
            }
            result.push('"');

            result
        }

        fn is_variable(s: &str) -> bool {
            s.chars().all(|x| x.is_uppercase() || x == '_')
        }

        fn format_arguments(input: &str) -> String {
            let mut chars = input.chars().peekable().into_iter();
            let mut args = vec![];

            loop {
                let c = if let Some(c) = chars.next() { c } else { break };

                match c {
                    ' ' | '\t' | '\n' => loop {
                        match chars.peek() {
                            Some(' ') | Some('\t') | Some('\n') => chars.next(),
                            _ => break,
                        };
                    },
                    '=' => {
                        args.push("=".to_string());
                    }
                    '"' | '\'' => {
                        let end_character = c;
                        let mut current_word = String::new();
                        loop {
                            match chars.peek() {
                                Some(c) => {
                                    if *c == end_character {
                                        let _ = chars.next();
                                        args.push(current_word);
                                        break;
                                    } else if *c == '\\' {
                                        let _ = chars.next();
                                        if let Some(c) = chars.next() {
                                            current_word.push(c);
                                        }
                                    } else {
                                        current_word.push(*c);
                                        chars.next();
                                    }
                                }
                                None => {
                                    break;
                                }
                            }
                        }
                    }
                    c => {
                        let mut current_word = c.to_string();
                        loop {
                            match chars.peek() {
                                Some(' ') | Some('\t') | Some('\n') | Some('=') | Some('\'')
                                | Some('"') | None => {
                                    args.push(current_word);
                                    break;
                                }
                                Some(c) => {
                                    current_word.push(*c);
                                    chars.next();
                                }
                            }
                        }
                    }
                }
            }

            print_args(
                args.into_iter()
                    .map(|arg| {
                        if arg.contains(char::is_whitespace) {
                            // arg has spaces, quote the string
                            quote_str(&arg)
                        } else if is_variable(&arg) {
                            // format as a unix variable
                            format!("${}{}{}", "{", arg, "}")
                        } else {
                            // it is fine to show as is
                            arg
                        }
                    })
                    .collect::<Vec<String>>(),
            )
        }


        pub fn examples() -> &'static str {
            Box::leak( [
            $(
            $crate::paste::paste! {
                    format!("  #{}\n  forc {} {}\n\n", stringify!($($description)*), stringify!($command), format_arguments(stringify!($($arg)*)) )
            },
            )*
            ].concat().into_boxed_str())
        }
    };
}
