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

        pub fn examples() -> &'static str {
            Box::leak( [
            $(
            $crate::paste::paste! {
                    format!("\t#{}\n\tforc {} {}\n\n", stringify!($($description)*), stringify!($command), stringify!($($arg)*) )
            },
            )*
            ].concat().into_boxed_str())
        }
    };
}
