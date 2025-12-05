//! Specific tests for the expression module

use forc_diagnostic::{println_green, println_red};
use paste::paste;
use prettydiff::{basic::DiffOp, diff_lines};
use test_macros::fmt_test_expr;

fmt_test_expr!(  literal "5", extra_whitespace "  5 "
);

fmt_test_expr!(  path_foo_bar            "foo::bar::baz::quux::quuz",
            intermediate_whitespace "foo :: bar :: baz :: quux :: quuz");

fmt_test_expr!(  field_proj_foobar       "foo.bar.baz.quux",
            intermediate_whitespace "foo . bar . baz . quux");

fmt_test_expr!(  abi_cast                "abi(MyAbi, 0x1111111111111111111111111111111111111111111111111111111111111111)",
            intermediate_whitespace " abi (
                  MyAbi
                   ,
                                 0x1111111111111111111111111111111111111111111111111111111111111111
                                  )  "
);

fmt_test_expr!(  basic_func_app          "foo()",
            intermediate_whitespace " foo (

            ) "
);

fmt_test_expr!(  nested_args_func_app    "foo(a_struct { hello: \"hi\" }, a_var, foo.bar.baz.quux)",
            intermediate_whitespace "foo(a_struct {
                    hello  :  \"hi\"
            }, a_var  , foo . bar . baz . quux)"
);

fmt_test_expr!(  multiline_tuple         "(\n    \"reallyreallylongstring\",\n    \"yetanotherreallyreallyreallylongstring\",\n    \"okaynowthatsjustaridiculouslylongstringrightthere\",\n)",
            intermediate_whitespace "(\"reallyreallylongstring\",             \"yetanotherreallyreallyreallylongstring\",
            \"okaynowthatsjustaridiculouslylongstringrightthere\")"
);

fmt_test_expr!(  nested_tuple
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

fmt_test_expr!(  multiline_match_stmt    "match foo {\n    Foo::foo => {}\n    Foo::bar => {}\n}",
            intermediate_whitespace "  match   \n  foo  {   \n\n    Foo :: foo  => {        }\n     Foo :: bar  =>  { }   \n}\n"
);

fmt_test_expr!(  if_else_block           "if foo {\n    foo();\n} else if bar { bar(); } else { baz(); }",
            intermediate_whitespace "   if    foo  {   \n       foo( ) ; \n }    else  if   bar  { \n     bar( ) ; \n }  else  { \n    baz(\n) ; \n }\n\n"
);

fmt_test_expr!(  long_conditional_stmt
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

fmt_test_expr!(  if_else_inline_1    "if foo { break; } else { continue; }",
            intermediate_whitespace "if  foo { \n        break; \n}    else  {\n    continue;    \n}");

fmt_test_expr!(  if_else_inline_2
"if foo { let x = 1; } else { bar(y); }"
 ,
            
            intermediate_whitespace 
"    if foo    {
        let x = 1;
    } else    {
    bar(y)   ;
}    ");

fmt_test_expr!(  if_else_multiline
"if foo {
    let really_long_variable = 1;
} else {
    bar(y);
}",
            
            intermediate_whitespace 
"    if foo    {
    let    really_long_variable = 1;
    } else    {
    bar(y)   ;
}    ");

fmt_test_expr!(  small_if_let "if let Result::Ok(x) = x { 100 } else { 1 }",
            intermediate_whitespace "if    let    Result   ::   Ok( x ) =    x {     100 }   else  {    1 }"
);

fmt_test_expr!(  match_nested_conditional
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

fmt_test_expr!(  match_branch_kind
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

fmt_test_expr!(  match_branch_kind_tuple_long
"match (foo, bar) {
    (
        SomeVeryVeryLongFoo::SomeLongFoo(some_foo),
        SomeVeryVeryLongBar::SomeLongBar(some_bar),
    ) => {
        foo();
    }
    (
        SomeVeryVeryLongFoo::OtherLongFoo(other_foo),
        SomeVeryVeryLongBar::OtherLongBar(other_bar),
    ) => {
        bar();
    }
    _ => {
        revert(0)
    }
}",
            intermediate_whitespace
"match (foo, bar) {
    (
        SomeVeryVeryLongFoo::SomeLongFoo(some_foo), \n  \n SomeVeryVeryLongBar::SomeLongBar(some_bar)) => \n 
    \n{
    \n
        foo();
    }
    (SomeVeryVeryLongFoo::OtherLongFoo(other_foo), SomeVeryVeryLongBar::OtherLongBar(other_bar) \n ) => {
        bar();
    \n
    }
    _ \n=> {
      \n  revert(0)
    }
}"
);

fmt_test_expr!(  basic_array             "[1, 2, 3, 4, 5]",
            intermediate_whitespace " \n [ 1 , 2 , 3 , 4 , 5 ]  \n"
);

fmt_test_expr!(  long_array
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

fmt_test_expr!(  nested_array
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

fmt_test_expr!(basic_while_loop
"while i == true {
    let i = 42;
}",
intermediate_whitespace
"while i==true{
let i = 42;
}");

fmt_test_expr!(scoped_block
"{
    let i = 42;
}",
intermediate_whitespace
"{
let i = 42;
}");

fmt_test_expr!(basic_for_loop
"for iter in [0, 1, 7, 8, 15] {
    let i = 42;
}",
intermediate_whitespace
"for iter in [0, 1, 7, 8, 15]{
let i = 42;
}"
);
