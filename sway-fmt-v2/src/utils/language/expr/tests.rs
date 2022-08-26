//! Specific tests for the expression module

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
/// convenience macro for generating test cases
/// provide a known good, and then some named test cases that should evaluate to
/// that known good. e.g.:
/// ```
///       // test suite name          known good
///fmt_test!(field_proj_foobar       "foo.bar.baz.quux",
///       // test case name           should format to known good
///          intermediate_whitespace "foo . bar . baz . quux");
/// ```
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
                println!("{formatted_code}");
                assert_eq!(&formatted_code, $desired_output)
            }
        }
    )+
}
}

fmt_test!(  literal "5", extra_whitespace "  5 "
);

fmt_test!(  path_foo_bar            "foo::bar::baz::quux::quuz",
            intermediate_whitespace "foo :: bar :: baz :: quux :: quuz");

fmt_test!(  field_proj_foobar       "foo.bar.baz.quux",
            intermediate_whitespace "foo . bar . baz . quux");

fmt_test!(  abi_cast                "abi(MyAbi, 0x1111111111111111111111111111111111111111111111111111111111111111)",
            intermediate_whitespace " abi (
                  MyAbi
                   ,
                                 0x1111111111111111111111111111111111111111111111111111111111111111
                                  )  "
);

fmt_test!(  basic_func_app          "foo()",
            intermediate_whitespace " foo (

            ) "
);

fmt_test!(  nested_args_func_app    "foo(a_struct { hello: \"hi\" }, a_var, foo.bar.baz.quux)",
            intermediate_whitespace "foo(a_struct {
                    hello  :  \"hi\"
            }, a_var  , foo . bar . baz . quux)"
);

fmt_test!(  multiline_tuple         "(\n    \"reallyreallylongstring\",\n    \"yetanotherreallyreallyreallylongstring\",\n    \"okaynowthatsjustaridiculouslylongstringrightthere\",\n)",
            intermediate_whitespace "(\"reallyreallylongstring\",             \"yetanotherreallyreallyreallylongstring\",
            \"okaynowthatsjustaridiculouslylongstringrightthere\")"
);

fmt_test!(  nested_tuple
"(
    (
        0x0000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000,
    ),
    (
        0x0000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000,
    ),
)",
            intermediate_whitespace
"   (
        (
            0x0000000000000000000000000000000000000000000000000000000000 ,
            0x0000000000000000000000000000000000000000000000000000000000 ,
        ) ,
(
            0x0000000000000000000000000000000000000000000000000000000000 ,
        0x0000000000000000000000000000000000000000000000000000000000 ,
 ) ,
)"
);

fmt_test!(  multiline_match_stmt    "match foo {\n    Foo::foo => {}\n    Foo::bar => {}\n}",
            intermediate_whitespace "  match   \n  foo  {   \n\n    Foo :: foo  => {        }\n     Foo :: bar  =>  { }   \n}\n"
);

fmt_test!(  if_else_block           "if foo {\n    foo();\n} else if bar { bar(); } else { baz(); }",
            intermediate_whitespace "   if    foo  {   \n       foo( ) ; \n }    else  if   bar  { \n     bar( ) ; \n }  else  { \n    baz(\n) ; \n }\n\n"
);

fmt_test!(  long_conditional_stmt
"if really_long_var_name > other_really_long_var
    || really_long_var_name <= 0
    && other_really_long_var != 0
{
    foo();
} else {
    bar();
}",
            intermediate_whitespace
"   if really_long_var_name  >    
other_really_long_var  
||  really_long_var_name  <=    
0  &&   other_really_long_var    !=    0 {  foo();  }else{bar();}"
);

fmt_test!(  if_else_control_flow    "if foo { break; } else { continue; }",
            intermediate_whitespace "if  foo { \n        break; \n}    else  {\n    continue;    \n}");

fmt_test!(  small_if_let "if let Result::Ok(x) = x { 100 } else { 1 }",
            intermediate_whitespace "if let Result::Ok(x) = x { 100 } else { 1 }"
);

fmt_test!(  match_nested_conditional
"match foo {
    Foo::foo => {
        if really_long_var > other_really_long_var {
            foo();
        } else if really_really_long_var_name > really_really_really_really_long_var_name111111111111
        {
            bar();
        } else {
            baz();
        }
    }
}",
            intermediate_whitespace
"     match foo {
        Foo::foo   =>    {
          if      really_long_var   >     other_really_long_var {
    foo();
    }     else if really_really_long_var_name        > really_really_really_really_long_var_name111111111111
        {   
                bar();
     }    else      {    
            baz()   ;  
            }
    } 
}"
);

fmt_test!(  match_branch_kind
"match foo {
    Foo::foo => {
        foo();
        bar();
    }
    Foo::bar => {
        baz();
        quux();
    }
}",
            intermediate_whitespace
"match     foo
            
\n{\n\n    Foo::foo    
     => {\n        foo() 
        ;     \n        bar(
         ); \n    } \n    Foo::\nbar => 
         {\n        baz();\n        
quux();\n    }\n\n\n}"
);

fmt_test!(  basic_array             "[1, 2, 3, 4, 5]",
            intermediate_whitespace " \n [ 1 , 2 , 3 , 4 , 5 ]  \n"
);

fmt_test!(  long_array
"[
    \"hello_there_this_is_a_very_long_string\",
    \"and_yet_another_very_long_string_just_because\",
    \"would_you_look_at_that_another_long_string\",
]",
intermediate_whitespace
"    [
       \"hello_there_this_is_a_very_long_string\",
     \"and_yet_another_very_long_string_just_because\"\n,
         \"would_you_look_at_that_another_long_string\",
 ]    \n"
);

fmt_test!(  nested_array
"[
    [
        0x0000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000,
    ],
    [
        0x0000000000000000000000000000000000000000000000000000000000,
        0x0000000000000000000000000000000000000000000000000000000000,
    ],
]",
            intermediate_whitespace
"   [
      [
         0x0000000000000000000000000000000000000000000000000000000000 ,
         0x0000000000000000000000000000000000000000000000000000000000 ,
     ] ,
[
         0x0000000000000000000000000000000000000000000000000000000000 ,
        0x0000000000000000000000000000000000000000000000000000000000 ,
     ] ,
  ]"
);
