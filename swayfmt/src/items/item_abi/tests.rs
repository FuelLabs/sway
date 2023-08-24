use forc_tracing::{println_green, println_red};
use paste::paste;
use prettydiff::{basic::DiffOp, diff_lines};
use test_macros::fmt_test_item;

fmt_test_item!(abi_contains_constant
"abi A {
    const ID: u32;
}",
intermediate_whitespace
"abi A {
const ID: u32;
}");

fmt_test_item!(abi_contains_functions
"abi A {
    fn hi() -> bool;
    fn hi2(hello: bool);
    fn hi3(hello: bool) -> u64;
}",
intermediate_whitespace
"abi A {
fn hi() -> bool;
    fn hi2(hello: bool);
        fn hi3(hello: bool)-> u64;
}");

fmt_test_item!(abi_contains_comments
"abi A {
    fn hi() -> bool;
    /// Function 2
    fn hi2(hello: bool);
    fn hi3(hello: bool) -> u64; // here too
}",
intermediate_whitespace
"abi A {
fn hi() -> bool;
/// Function 2
    fn hi2(hello: bool);
        fn hi3(hello: bool)-> u64;// here too
}");
    