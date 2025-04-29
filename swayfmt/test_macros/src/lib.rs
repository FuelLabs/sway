/// Convenience macro for generating test cases for a parsed Item of ItemKind.
/// This macro is a wrapper around the fmt_test! macro and simply passes the AST type
/// to it.
///
/// Provide a known good, and then some named test cases that should evaluate to
/// that known good. e.g.:
/// ```
/// # use paste::paste;
/// # use prettydiff::{basic::DiffOp, diff_lines};
/// # use test_macros::fmt_test_item; fn main() {
///               // test suite name       known good
///fmt_test_item!(field_proj_foobar       "foo.bar.baz.quux",
///               // test case name        should format to known good
///               intermediate_whitespace  "foo . bar . baz . quux");
/// # }
/// ```
#[macro_export]
macro_rules! fmt_test_item {
    ($scope:ident $desired_output:expr, $($name:ident $y:expr),+) =>{
        $crate::fmt_test!(sway_ast::ItemKind, $scope $desired_output,
                    $($name $y)+);
            };
}

/// Convenience macro for generating test cases for a parsed Expr.
/// This macro is a wrapper around the fmt_test! macro and simply passes the AST type
/// to it.
///
/// Provide a known good, and then some named test cases that should evaluate to
/// that known good. e.g.:
/// ```
/// # use paste::paste;
/// # use prettydiff::{basic::DiffOp, diff_lines};
/// # use test_macros::fmt_test_expr; fn main() {
///               // test suite name       known good
///fmt_test_expr!(field_proj_foobar       "foo.bar.baz.quux",
///               // test case name        should format to known good
///               intermediate_whitespace  "foo . bar . baz . quux");
/// # }
/// ```
#[macro_export]
macro_rules! fmt_test_expr {
    ($scope:ident $desired_output:expr, $($name:ident $y:expr),+) =>{
        $crate::fmt_test!(sway_ast::Expr, $scope $desired_output,
                    $($name $y)+);
            };
}

/// Convenience macro for generating test cases.
///
/// This macro should be wrapped by another macro, eg. `fmt_test_expr!` that passes
/// in a Sway AST type, eg. sway_ast::Expr, and is not meant to be used directly.
#[macro_export]
macro_rules! fmt_test {
    ($ty:expr, $scope:ident $desired_output:expr, $($name:ident $y:expr),+) => {
        $crate::fmt_test_inner!($ty, $scope $desired_output,
                                $($name $y)+
                                ,
                                remove_trailing_whitespace format!("{} \n\n\t ", $desired_output).as_str(),
                                remove_beginning_whitespace format!("  \n\t{}", $desired_output).as_str(),
                                identity $desired_output, /* test return is valid */
                                remove_beginning_and_trailing_whitespace format!("  \n\t  {} \n\t   ", $desired_output).as_str()
                       );
    };
}

/// Inner macro for fmt_test! that does the actual formatting and presents the diffs.
///
/// This macro is not meant to be called directly, but through fmt_test!.
#[allow(clippy::crate_in_macro_def)] // Allow external parse crate
#[macro_export]
macro_rules! fmt_test_inner {
    ($ty:expr, $scope:ident $desired_output:expr, $($name:ident $y:expr),+) => {
        $(
        paste! {
            #[test]
            fn [<$scope _ $name>] () {
                let formatted_code = crate::parse::parse_format::<$ty>($y, sway_features::ExperimentalFeatures::default()).unwrap();
                let changeset = diff_lines(&formatted_code, $desired_output);
                let count_of_updates = changeset.diff().len();
                if count_of_updates != 0 {
                    println!("FAILED: {count_of_updates} diff items.");
                }
                for diff in changeset.diff() {
                    match diff {
                        DiffOp::Equal(old) => {
                            for o in old {
                                println!("{}", o)
                            }
                        }
                        DiffOp::Insert(new) => {
                            for n in new {
                                println_green(&format!("+{}", n));
                            }
                        }
                        DiffOp::Remove(old) => {
                            for o in old {
                                println_red(&format!("-{}", o));
                            }
                        }
                        DiffOp::Replace(old, new) => {
                            for o in old {
                                println_red(&format!("-{}", o));
                            }
                            for n in new {
                                println_green(&format!("+{}", n));
                            }
                        }
                    }
                }
                println!("{formatted_code}");
                assert_eq!(&formatted_code, $desired_output)
            }
        }
    )+
}
}

#[macro_export]
macro_rules! assert_eq_pretty {
    ($got:expr, $expected:expr) => {
        let got = &$got[..];
        let expected = &$expected[..];

        if got != expected {
            use similar::TextDiff;

            let diff = TextDiff::from_lines(expected, got);
            for op in diff.ops() {
                for change in diff.iter_changes(op) {
                    match change.tag() {
                        similar::ChangeTag::Equal => print!("{}", change),
                        similar::ChangeTag::Insert => print!("\x1b[32m+{}\x1b[0m", change), // Green for additions
                        similar::ChangeTag::Delete => print!("\x1b[31m-{}\x1b[0m", change), // Red for deletions
                    }
                }
            }
            println!();
            panic!("printed outputs differ!");
        }
    };
}
