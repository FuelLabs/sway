use forc_tracing::{println_green, println_red};
use pastey::paste;
use prettydiff::{basic::DiffOp, diff_lines};
use test_macros::fmt_test_item;

fmt_test_item!(  long_fn_name            "pub fn hello_this_is_a_really_long_fn_name_wow_so_long_ridiculous(\n    self,\n    foo: Foo,\n    bar: Bar,\n    baz: Baz,\n) {}",
            intermediate_whitespace "pub fn hello_this_is_a_really_long_fn_name_wow_so_long_ridiculous    ( self   , foo   : Foo    , bar : Bar  ,    baz:    Baz) {\n    }"
);

fmt_test_item!(  long_fn_args            "fn foo(\n    mut self,\n    this_is_a_really_long_variable: Foo,\n    hello_im_really_long: Bar,\n) -> String {}",
            intermediate_whitespace "  fn  foo( \n        mut self , \n     this_is_a_really_long_variable : Foo ,\n    hello_im_really_long: Bar , \n ) ->    String { \n }     "
);

fmt_test_item!(  non_self_fn
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

fmt_test_item!(  fn_with_nested_items
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

fmt_test_item!(  fn_nested_if_lets
"fn has_nested_if_let() {
    let result_1 = if let Result::Ok(x) = x { 100 } else { 1 };
    let result_2 = if let Result::Err(x) = x { 3 } else { 43 };
}",
            intermediate_whitespace
"fn has_nested_if_let() {
    let result_1 = if let Result::Ok(x) = x {
        100
    } else {
        1
    };
    let result_2 = if let Result::Err(x) = x {
        3
    } else {
        43
    };
}"
);

fmt_test_item!(  fn_conditional_with_comment
"fn conditional_with_comment() {
    if true {
        // comment here
    }
}",
intermediate_whitespace
"fn conditional_with_comment() {
    if true {
        // comment here
    }
}"
);

fmt_test_item!(  fn_conditional_with_comment_and_else
"fn conditional_with_comment() {
    if true {
        // if
    } else {
        // else
    }
}",
intermediate_whitespace
"fn conditional_with_comment() {
    if true {
        // if
    } else {
        // else
    }
}"
);

fmt_test_item!(fn_comments_special_chars
"fn comments_special_chars() {
    // this ↓↓↓↓↓   
    let val = 1; // this is a normal comment
}",
intermediate_whitespace
"fn comments_special_chars() {
    // this ↓↓↓↓↓   
    let val = 1;      // this is a normal comment
}"
);
