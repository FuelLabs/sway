#[macro_export]
macro_rules! fmt_test_expr {
    ($scope:ident $desired_output:expr, $($name:ident $y:expr),+) =>{
        fmt_test!(sway_ast::Expr, $scope $desired_output,
                    $($name $y)+);
            };
}

#[macro_export]
macro_rules! fmt_test_impl {
    ($scope:ident $desired_output:expr, $($name:ident $y:expr),+) =>{
        fmt_test!(sway_ast::ItemImpl, $scope $desired_output,
                    $($name $y)+);
            };
}

/// convenience macro for generating test cases
/// provide a known good, and then some named test cases that should evaluate to
/// that known good. e.g.:
/// ```
///       // test suite name          known good
///fmt_test!(field_proj_foobar       "foo.bar.baz.quux",
///       // test case name           should format to known good
///          intermediate_whitespace "foo . bar . baz . quux");
/// ```
#[macro_export]
macro_rules! fmt_test {
    ($ty:expr, $scope:ident $desired_output:expr, $($name:ident $y:expr),+) => {
        fmt_test_inner!($ty, $scope $desired_output,
                                $($name $y)+
                                ,
                                remove_trailing_whitespace format!("{} \n\n\t ", $desired_output).as_str(),
                                remove_beginning_whitespace format!("  \n\t{}", $desired_output).as_str(),
                                identity $desired_output, /* test return is valid */
                                remove_beginning_and_trailing_whitespace format!("  \n\t  {} \n\t   ", $desired_output).as_str()
                       );
    };
}

#[macro_export]
macro_rules! fmt_test_inner {
    ($ty:expr, $scope:ident $desired_output:expr, $($name:ident $y:expr),+) => {
        $(
        paste! {
            #[test]
            fn [<$scope _ $name>] () {
                let formatted_code = crate::parse::parse_format::<$ty>($y);
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
        let got = &$got;
        let expected = &$expected;
        if got != expected {
            panic!(
                "
printed outputs differ!
expected:
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
{}
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
got:
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
{}
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
",
                expected, got
            );
        }
    };
}
