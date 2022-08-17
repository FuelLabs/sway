use crate::{Format, Formatter};
use forc_util::{println_green, println_red};
use paste::paste;
use prettydiff::{basic::DiffOp, diff_lines};
use sway_ast::Expr;
use sway_parse::{handler::Handler, *};

fn format_code(input: &str) -> String {
    let mut formatter: Formatter = Default::default();
    let input_arc = std::sync::Arc::from(input);
    let token_stream = lex(&input_arc, 0, input.len(), None).unwrap();
    let handler = Handler::default();
    let mut parser = Parser::new(&token_stream, &handler);
    let expression: Expr = parser.parse().unwrap();

    let mut buf = Default::default();
    expression.format(&mut buf, &mut formatter).unwrap();

    buf
}

macro_rules! fmt_test {
    ($scope:ident $desired_output:expr, $($name:ident $y:expr),+) => {
        fmt_test_inner!($scope $desired_output,
                                $($name $y)+
                                ,
                                remove_trailing_whitespace format!("{} \n\n\t ", $desired_output).as_str(),
                                remove_beginning_whitespace format!("  \n\t{}", $desired_output).as_str(),
                                identity $desired_output, /* test return is valid */
                                remove_beginning_and_trailing_whitespace format!("  \n\t  {} \n\t   ", $desired_output).as_str()
                       );
    };
}

macro_rules! fmt_test_inner {
    ($scope:ident $desired_output:expr, $($name:ident $y:expr),+) => {
        $(
        paste! {
            #[test]
            fn [<$scope _ $name>] () {
                let formatted_code = format_code($y);
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
                assert_eq!(&formatted_code, $desired_output)
            }
        }
    )+
}
}

fmt_test!(literal "5", extra_whitespace "  5 "
);

fmt_test!(  path_foo_bar "foo::bar::baz::quux::quuz",
            intermediate_whitespace "foo :: bar :: baz :: quux :: quuz");

fmt_test!(  field_proj_foobar "foo.bar.baz.quux",
            intermediate_whitespace "foo . bar . baz . quux");

fmt_test!(  abi_cast "abi(MyAbi, 0x1111111111111111111111111111111111111111111111111111111111111111)",
            intermediate_whitespace " abi (
                  MyAbi
                   ,
                                 0x1111111111111111111111111111111111111111111111111111111111111111
                                  )  "
);

fmt_test!(  basic_func_app "foo()",
            intermediate_whitespace " foo (

            ) "
);

fmt_test!(  nested_args_func_app "foo(a_struct { hello: \"hi\" }, a_var, foo.bar.baz.quux)",
            intermediate_whitespace "foo(a_struct {
                    hello  :  \"hi\"
            }, a_var  , foo . bar . baz . quux)"
);

fmt_test!(  multiline_tuple "(\n    \"reallyreallylongstring\",\n    \"yetanotherreallyreallyreallylongstring\",\n    \"okaynowthatsjustaridiculouslylongstringrightthere\",\n)",
            intermediate_whitespace "(\"reallyreallylongstring\",             \"yetanotherreallyreallyreallylongstring\",
            \"okaynowthatsjustaridiculouslylongstringrightthere\")"
);

fmt_test!(  multiline_match_stmt "match foo {\n    Foo::foo => {}\n    Foo::bar => {}\n}",
            intermediate_whitespace "match foo {\n    Foo::foo => {}\n    Foo::bar => {}\n}"
);
