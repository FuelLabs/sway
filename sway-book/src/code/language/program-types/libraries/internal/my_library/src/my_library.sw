// ANCHOR: library
// ANCHOR: module
library my_library;
// ANCHOR_END: module

// Cannot import because the `pub` keyword is missing
fn foo() {}

// Can import everything below because they are using the `pub` keyword
pub const ONE = "1";

pub struct MyStruct {}

pub enum MyEnum {
    Variant: ()
}

pub fn bar() {}
// ANCHOR_END: library
