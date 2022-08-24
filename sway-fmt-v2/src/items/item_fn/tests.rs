use crate::{Format, Formatter};
use forc_util::{println_green, println_red};
use paste::paste;
use prettydiff::{basic::DiffOp, diff_lines};
use sway_ast::ItemFn;
use sway_parse::{handler::Handler, *};

fn format_code(input: &str) -> String {
    let mut formatter: Formatter = Default::default();
    let input_arc = std::sync::Arc::from(input);
    let token_stream = lex(&input_arc, 0, input.len(), None).unwrap();
    let handler = Handler::default();
    let mut parser = Parser::new(&token_stream, &handler);
    let expression: ItemFn = parser.parse().unwrap();

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
                let diff = changeset.diff();
                let count_of_updates = diff.len();
                if count_of_updates != 0 {
                    println!("FAILED: {count_of_updates} diff items.");
                }
                for diff in diff {
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

fmt_test!(  long_fn_name            "pub fn hello_this_is_a_really_long_fn_name_wow_so_long_ridiculous(\n    self,\n    foo: Foo,\n    bar: Bar,\n    baz: Baz,\n) {\n}",
            intermediate_whitespace "pub fn hello_this_is_a_really_long_fn_name_wow_so_long_ridiculous    ( self   , foo   : Foo    , bar : Bar  ,    baz:    Baz) {\n    }"
);

fmt_test!(  long_fn_args            "fn foo(\n    mut self,\n    this_is_a_really_long_variable: Foo,\n    hello_im_really_long: Bar,\n) -> String {\n}",
            intermediate_whitespace "  fn  foo( \n        mut self , \n     this_is_a_really_long_variable : Foo ,\n    hello_im_really_long: Bar , \n ) ->    String { \n }     "
);

fmt_test!(  non_self_fn
"fn test_function(
    helloitsverylong: String,
    whatisgoingonthisistoolong: String,
    yetanotherlongboy: String,
) -> bool {
    match foo {
        Foo::foo => true,
        _ => false,
    }
}",
            intermediate_whitespace
"fn   test_function   (\n\n
    helloitsverylong : String ,
    whatisgoingonthisistoolong   : String    ,
    yetanotherlongboy   : String   ,\n
) -> bool {
    match  foo  {
        Foo :: foo => true ,
         _    => false  ,
        }
}"
);

fmt_test!(  fn_with_nested_items
"fn returns_msg_sender(expected_id: ContractId) -> bool {
    let result: Result<Identity, AuthError> = msg_sender();
    let mut ret = false;
    if result.is_err() {
        ret = false;
    }
    let unwrapped = result.unwrap();
    match unwrapped {
        Identity::ContractId(v) => {
            ret = true
        }
        _ => {
            ret = false
        }
    }
    ret
}",
            intermediate_whitespace
"fn returns_msg_sender(expected_id: ContractId) -> bool {
    let result: Result<Identity, AuthError> = msg_sender();
    let mut ret = false;
    if result.is_err() {
        ret = false;
    }
    let unwrapped = result.unwrap();
    match unwrapped {
        Identity::ContractId(v) => {ret = true}
        _ => {ret = false}
    }
    ret
}"
);
